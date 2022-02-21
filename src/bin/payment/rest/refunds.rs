use actix_web::{
    HttpRequest, HttpResponse,
    web, web::Query, web::Json,
    Error,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use futures::{
    future,
    future::Either,
    Future,
    Stream,
};
use std::str::FromStr;
use serde::Serialize;
use std::marker::Send;

use crate::db;
use crate::db::GetPool;
use crate::{AppState};
use crate::models::{
    StripeError, DbError, ErrJson,
    PaypalResponse,
    PaypalError,
    Transaction,
    RefundReason,
    Refund,
    PaypalRefundResponse,
    PaypalRefundDetails,
    PaypalErrorResponse,
    Currency,
    PayoutItem,
    PayeeType,
    PayoutStatus,
};
use crate::rest::PaymentProcessor;
use crate::payment_clients::PaypalRequest;
use crate::rpc::rpc_update_orders_for_refunds;
// import stripe traits to enable Requests
use gm::models::stripe;
use gm::models::stripe::{
    PaymentIntent,
    RefundCreateParams,
    List,
};
use crate::payment_clients::{
    PaymentIntentMsg,
    RefundMsg,
};



/// total = subtotal + taxes + payment_processing_fee
/// subtotal = seller_payment + platform_fee
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RefundOrderBody {
    order_id: String,
    refund_order_item_ids: Vec<String>,
    charge_id: String,
    taxes: i32,
    reason: Option<String>,
    reason_details: Option<String>,
    payment_intent_id: Option<String>, // only for Stripe
    paypal_invoice_number: Option<String>, // only for Paypal
    payment_processor: PaymentProcessor,
}

// #[serde(rename_all = "camelCase")]
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct RefundOrderItem {
//     order_item_id: String,
//     disable_item: bool,
//     refund_payout_items: Vec<RefundPayoutItem>,
// }
// #[serde(rename_all = "camelCase")]
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct RefundPayoutItem {
//     payee_id: String, // storeId or affiliateId
//     payee_type: PayeeType,
//     amount: i32,
//     payment_processing_fee: Option<i32>,
// }





pub async fn refund_endpoint(
    req: HttpRequest,
    json: Json<RefundOrderBody>
) -> Result<HttpResponse, Error> {
    match json.payment_processor {
        PaymentProcessor::Stripe => refund_stripe(req, json).await,
        PaymentProcessor::Paypal => refund_paypal(req, json).await,
    }
}


/////////////////////////////////
/// Stripe Refund
/////////////////////////////////


pub async fn refund_stripe(
    req: HttpRequest,
    json: Json<RefundOrderBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();

    let payment_intent_id = match body.payment_intent_id.clone() {
        None => return Err(Error::from(
            StripeError::IdPrefix(ErrJson::new("paymentIntentId wrong prefix!"))
        )),
        Some(id) => id,
    };

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    // read all payoutItems associated with orderItemIds
    let payout_items = db::read_payout_items_by_order_item_ids(
        &conn,
        &body.refund_order_item_ids,
    ).map_err(Error::from)?;
    // Guard against refunding an already paid-out payoutItem
    // accept only UNPAID payoutitems
    if payout_items.clone().into_iter().any(|p: PayoutItem| {
        p.payout_status != PayoutStatus::UNPAID
    }) {
        return Err(StripeError::Refund(errJson!(
            format!("PayoutItems already refunded: {:?}", &payout_items)
        ))).map_err(Error::from)
    }

    // sum payout amounts for each entity to refund
    let summed_payouts_for_entities = sum_payouts_for_all_payees(
        &payout_items,
    );
    let spfe = summed_payouts_for_entities;

    // Total amount to refund
    let total_amount = spfe.total_seller_payment
        + spfe.seller_payment_processing_fees
        + spfe.total_platform_fee
        + spfe.total_buyer_affiliate_fee
        + spfe.total_seller_affiliate_fee
        + body.taxes;

    // 1. dispatch a Stripe refund
    let stripe_refund_response: stripe::Refund = AppState::stripeActor(&req)
        .send(RefundMsg::Create(
            RefundCreateParams {
                amount: Some(total_amount as i64),
                charge: None, // deprecated for stripe. for paypal only
                payment_intent: Some(payment_intent_id.clone()),
                reason: body.reason.clone(),
                metadata: Default::default(),
                refund_application_fee: Default::default(),
                reverse_transfer: Default::default(),
            }
        ))
        .await??;

    // 1b. Lookup Stripe payment intent details
    let stripe_payment_intent: PaymentIntent = AppState::stripeActor(&req)
        .send(PaymentIntentMsg::Retrieve(payment_intent_id.clone()))
        .await??;


    debug!("stripe_refund_response: {:#?}", &stripe_refund_response);
    let r = stripe_refund_response.clone();

    let created_at = chrono::NaiveDateTime::from_timestamp(r.created, 0);
    let refund_currency = r.currency.to_string();

    // 2. create refund, transaction structs
    let refund = Refund {
        id: stripe_refund_response.id.to_string(),
        transaction_id: stripe_refund_response
            .balance_transaction.clone()
            .expect("balance_transaction missing in refund response")
            .id().to_string(),
        order_id: body.order_id,
        order_item_ids: Some(body.refund_order_item_ids.clone()),
        created_at: created_at,
        reason: body.reason.clone(),
        reason_details: body.reason_details,
    };

    let tx = Transaction {
        id: refund.id.clone(), // txn_xxxxxx
        subtotal: -sum_subtotal(
            spfe.total_seller_payment + spfe.seller_payment_processing_fees,
            spfe.total_platform_fee,
            spfe.total_buyer_affiliate_fee,
            spfe.total_seller_affiliate_fee,
        ),
        taxes: -body.taxes,
        payment_processing_fee: -spfe.seller_payment_processing_fees,
        created_at: created_at,
        currency: Currency::from_str(&refund_currency).ok(),
        customer_id: stripe_payment_intent.customer,
        order_id: Some(refund.order_id.clone()),
        charge_id: Some(payment_intent_id),
        payment_processor: Some(String::from("Stripe")),
        payment_method_id: stripe_payment_intent.payment_method,
        payment_intent_id: Some(stripe_payment_intent.id.to_string()),
        refund_id: Some(r.id.to_string()), // ref_xxxxxxx
        details: None,
    };

    let refund_items: Vec<PayoutItem> = create_refund_payout_items(
        &payout_items,
        &created_at,
        &tx.id,
    );

    // 3. write a refund_items and transaction
    let (tx, refund, _ritems) = db::write_transaction_and_refund_and_refund_items(
        &conn,
        &tx,
        &refund,
        &refund_items,
    ).map_err(Error::from)?;

    // 4. Update Order, OrderSnapshots, OrderItem statuses
    let _order_db = rpc_update_orders_for_refunds(
        AppState::httpClient(&req),
        tx.clone(),
        refund.clone(),
    ).await?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(json!({
        "refund": refund,
        "transaction": tx,
    })))
}


/////////////////////////////////
/// Paypal Refund
/////////////////////////////////


pub async fn refund_paypal(
    req: HttpRequest,
    json: Json<RefundOrderBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    // read all payoutItems associated with orderItemIds
    let payout_items = db::read_payout_items_by_order_item_ids(
        &conn,
        &body.refund_order_item_ids,
    ).map_err(Error::from)?;
    // Guard against refunding an already paid-out payoutItem
    // accept only UNPAID payoutitems
    if payout_items.clone().into_iter().any(|p: PayoutItem| {
        p.payout_status != PayoutStatus::UNPAID
    }) {
        return Err(StripeError::Refund(errJson!(
            format!("PayoutItems already refunded: {:?}", &payout_items)
        ))).map_err(Error::from)
    }

    // sum payout amounts for each entity to refund
    let summed_payouts_for_entities = sum_payouts_for_all_payees(
        &payout_items,
    );
    let spfe = summed_payouts_for_entities;

    // Total amount to refund
    let total_amount = spfe.total_seller_payment
        + spfe.seller_payment_processing_fees
        + spfe.total_platform_fee
        + spfe.total_buyer_affiliate_fee
        + spfe.total_seller_affiliate_fee
        + body.taxes;

    // 1. dispatch a Paypal refund
    let paypal_refund_response = AppState::paypalActor(&req)
        .send(PaypalRequest::PostBody(
            format!("/v1/payments/sale/{}/refund", body.charge_id),
            json!({
                "amount": {
                     "total": ((total_amount as f64) / 100.0).to_string(),
                     "currency": "USD"
                },
                "invoice_number": body.paypal_invoice_number,
                "description": body.reason_details,
            })
        ))
        .await??;

    // PAYPAL_PAYMENTS_API = 'https://api.sandbox.paypal.com/v2/payments/captures/';
    // format!("/v2/payments/captures/{}/refund", body.charge_id),

    let r = paypal_refund_response;
    debug!("paypal_refund_response: {:#?}", &r);

    let paypal_refund = match serde_json::from_str::<PaypalRefundResponse>(&r) {
        Ok(res) => res,
        Err(_e) => match serde_json::from_str::<PaypalErrorResponse>(&r) {
            Err(_e) => {
                // unexpected error, return raw string response as Error
                return Err(PaypalError::InternalError(errJson!(r.clone())))
                            .map_err(Error::from)
            },
            Ok(e2) => match e2.name {
                Some(dup) => return Err(PaypalError::DuplicateTransaction(
                    errJson!(dup)
                )).map_err(Error::from),
                None => return Err(PaypalError::DuplicateTransaction(
                    errJson!("DUPLICATE_TRANSACTION")
                )).map_err(Error::from),
            }
        }
    };

    // 2. get Paypal refund details
    let refund_details_response = AppState::paypalActor(&req)
        .send(PaypalRequest::Get::<serde_json::Value>(
            format!("/v2/payments/refunds/{}", paypal_refund.id)
        ))
        .await
        .map_err(|e| Error::from(PaypalError::InternalError(errJson!(e))))?;

    let refund_details = serde_json::from_str::<PaypalRefundDetails>(
            &refund_details_response.expect("refund_details_response")
        ).map_err(Error::from)?;

    debug!("refund_details: {:#?}", &refund_details);

    let tx_id = match refund_details.invoice_id.starts_with("txn_") {
        true => refund_details.invoice_id.clone(),
        false => format!("txn_{}", refund_details.invoice_id),
    };
    let refund_id = match refund_details.id.starts_with("re_") {
        true => refund_details.id.clone(),
        false => format!("re_{}", refund_details.id),
    };

    let created_at = refund_details.create_time.unwrap_or(
        chrono::NaiveDateTime::from_timestamp(
            chrono::Utc::now().timestamp(),
            0
        )
    );

    let refund_currency = refund_details.amount.currency_code.clone()
        .unwrap_or(String::from("USD"));

    // 3. create refund + transaction structs to write to DB
    let refund = Refund {
        id: refund_id.clone(),
        transaction_id: tx_id,
        order_id: body.order_id,
        order_item_ids: Some(body.refund_order_item_ids),
        created_at: created_at,
        reason: body.reason.map(|r| r.as_str().to_string()),
        reason_details: body.reason_details,
    };

    let (
        payment_method_id,
        customer_id,
    ) = db::read_payment_method_id_for_order(
        &conn,
        &refund.order_id
    );

    let tx = Transaction {
        id: refund.id.clone(), // txn_xxxxxx
        subtotal: -sum_subtotal(
            spfe.total_seller_payment + spfe.seller_payment_processing_fees,
            spfe.total_platform_fee,
            spfe.total_buyer_affiliate_fee,
            spfe.total_seller_affiliate_fee,
        ),
        taxes: -body.taxes,
        payment_processing_fee: -spfe.seller_payment_processing_fees,
        created_at: created_at,
        currency: Currency::from_str(&refund_currency).ok(),
        customer_id: customer_id,
        order_id: Some(refund.order_id.clone()),
        charge_id: Some(refund_details.id.clone()),
        payment_processor: Some(String::from("Paypal")),
        payment_method_id: payment_method_id,
        payment_intent_id: None,
        refund_id: Some(refund.id.clone()), // re_xxxxxxx
        details: None,
    };

    let refund_payout_items: Vec<PayoutItem> = create_refund_payout_items(
        &payout_items,
        &created_at,
        &tx.id,
    );

    debug!("refund: {:?}", refund);
    debug!("transaction: {:?}", tx);

    // 4. write a refund and transaction
    let (tx, refund, _pitems) = db::write_transaction_and_refund_and_refund_items(
        &conn,
        &tx,
        &refund,
        &refund_payout_items,
    ).map_err(Error::from)?;

    // 5. Update Order, OrderSnapshots, OrderItem statuses
    let _order_db = rpc_update_orders_for_refunds(
        AppState::httpClient(&req),
        tx.clone(),
        refund.clone(),
    ).await?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(json!({
        "refund": refund,
        "transaction": tx,
    })))
}

///////////// Helpers ///////////

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadRefundsByIdsBody {
    refund_ids: Vec<String>,
}

pub async fn read_refunds_by_ids(
    req: HttpRequest,
    json: Json<ReadRefundsByIdsBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let refund_ids = body.refund_ids;

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let refunds = db::read_many_refunds(
        &conn,
        refund_ids,
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(refunds))
}


// pub fn sum_payouts_by_payee_type(
//     refund_order_items: &Vec<RefundOrderItem>,
//     payee_type: PayeeType
// ) -> i32 {
//     refund_order_items
//         .iter()
//         .map(|oitem: &RefundOrderItem| {
//             oitem.refund_payout_items
//                 .iter()
//                 .filter(|pitem: &&RefundPayoutItem| pitem.payee_type == payee_type)
//                 .fold(0, |acc: i32, pitem: &RefundPayoutItem| {
//                     acc + pitem.amount
//                 })
//         })
//         .fold(0, |acc: i32, seller_payment: i32| acc + seller_payment)
// }

pub fn sum_payment_processing_fees(
    payout_items: &Vec<PayoutItem>,
    payee_type: PayeeType
) -> i32 {
    payout_items
        .iter()
        .filter(|pitem: &&PayoutItem| pitem.payee_type == payee_type)
        .fold(0, |acc: i32, pitem: &PayoutItem| {
            acc + pitem.payment_processing_fee
        })
}

pub fn sum_payouts_by_payee_type(
    payout_items: &Vec<PayoutItem>,
    payee_type: PayeeType
) -> i32 {
    payout_items
        .iter()
        .filter(|pitem: &&PayoutItem| pitem.payee_type == payee_type)
        .fold(0, |acc: i32, pitem: &PayoutItem| {
            acc + pitem.amount
        })
}

pub fn sum_payouts_for_all_payees(
    payout_items: &Vec<PayoutItem>,
) -> SumPayoutsByPayeeType {

    let total_seller_payment = sum_payouts_by_payee_type(
        payout_items,
        PayeeType::STORE
    );

    let total_platform_fee = sum_payouts_by_payee_type(
        payout_items,
        PayeeType::PLATFORM
    );

    let total_buyer_affiliate_fee = sum_payouts_by_payee_type(
        payout_items,
        PayeeType::BUYER_AFFILIATE
    );

    let total_seller_affiliate_fee = sum_payouts_by_payee_type(
        payout_items,
        PayeeType::SELLER_AFFILIATE
    );

    let seller_payment_processing_fees = sum_payment_processing_fees(
        payout_items,
        PayeeType::STORE
    );

    SumPayoutsByPayeeType {
        total_seller_payment: total_seller_payment,
        total_platform_fee: total_platform_fee,
        total_buyer_affiliate_fee: total_buyer_affiliate_fee,
        total_seller_affiliate_fee: total_seller_affiliate_fee,
        seller_payment_processing_fees: seller_payment_processing_fees,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SumPayoutsByPayeeType {
    total_seller_payment: i32,
    total_platform_fee: i32,
    total_buyer_affiliate_fee: i32,
    total_seller_affiliate_fee: i32,
    seller_payment_processing_fees: i32,
}

// pub fn create_refund_payout_items(
//     refund_order_items: &Vec<RefundOrderItem>,
//     created_at: &chrono::NaiveDateTime,
//     refund_currency: &str,
//     tx_id: &str,
// ) -> Vec<PayoutItem> {
//     // create m PayoutItems from n RefundOrderItems
//     // where m >= n
//     refund_order_items.into_iter()
//         .map(move |oitem: &RefundOrderItem| {
//             oitem.refund_payout_items.iter()
//                 .filter(|p| p.amount != 0)
//                 .map(move |pitem: &RefundPayoutItem| {
//                     PayoutItem::new(
//                         oitem.order_item_id.clone(),
//                         pitem.payee_id.clone(),
//                         -pitem.amount,
//                         -pitem.payment_processing_fee.unwrap_or(0),
//                         created_at.clone(),
//                         refund_currency.to_string(),
//                         tx_id.to_string(),
//                     ).set_payout_status(PayoutStatus::REFUNDING)
//                 })
//         })
//         .flatten()
//         .collect::<Vec<PayoutItem>>()
// }


pub fn create_refund_payout_items(
    payout_items: &Vec<PayoutItem>,
    created_at: &chrono::NaiveDateTime,
    tx_id: &str,
) -> Vec<PayoutItem> {
    // create m RefundPayoutItems from m PayoutItems
    // RefundPayoutItem is the same as PayoutItem, just with new ids and time
    payout_items.into_iter()
        .filter(|p| p.amount != 0)
        .map(move |pitem: &PayoutItem| {
            pitem.to_refund(
                created_at.clone(),
                tx_id.to_string(), // create a next txn_id for refund
            )
        })
        .collect::<Vec<PayoutItem>>()
}


fn sum_subtotal(
    seller_payment: i32,
    platform_fee: i32,
    buyer_affiliate_fee: i32,
    seller_affiliate_fee: i32,
) -> i32 {
    seller_payment + platform_fee + buyer_affiliate_fee + seller_affiliate_fee
}
