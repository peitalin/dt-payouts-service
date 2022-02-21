use actix::{Actor, Addr};
use actix_web::{
    client::ClientResponse,
    HttpRequest, HttpResponse,
    web::Json,
    web::Query,
    Error,
};
use futures::{
    future,
    future::Either,
    Future,
    Stream,
};
use std::str::FromStr;
use serde::Serialize;
use std::marker::Send;

use crate::AppState;
use crate::models::{
    PaypalResponse, PaypalError,
    Transaction, TransactionId,
    DbError, ErrJson,
    CartRpc,
    TxQuery,
    OrderDb,
    PayoutItem,
    Currency,
    to_payout_items,
};
use crate::models::paypal::{
    PaypalPayoutParams,
    PaypalSenderBatchHeader,
    PaypalPayout,
    PaypalPayoutResponse,
};
// message actions
use crate::payment_clients::{
    PaypalRequest,
    PaypalAuthRefresh,
};

use crate::rpc::{
    rpc_get_cart,
    rpc_create_order,
    rpc_cart_clear,
    rpc_confirm_order,
};
use crate::rpc;
use crate::db;
use crate::db::{ GetPool };




/// This is the PaypalRequest from the front-end client
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalCreateOrderParams {
    order_id: String,
    payer_id: String,
    subtotal: i32,
    taxes: i32,
    payment_processing_fee: i32,
    total: i32,
    currency: Currency,
}

pub async fn paypal_create_order(
    req: HttpRequest,
    json: Json<PaypalCreateOrderParams>,
    query: Query<TxQuery>
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let query = query.into_inner();

    // 1. create transaction to send to Orders
    // 1b. Await cart response before proceeding
    let cart = rpc_get_cart(
        &AppState::httpClient(&req),
        &query.cart_id
    ).await?;

    // 2. Check Cart subtotals
    if cart.total != body.total {
        return Err(Error::from(PaypalError::ValidationError(errJson!(format!(
                "total {:?} does not equal submitted amount: {:?}",
                &cart.total,
                &body.total,
            )))))
    } else {
        info!("cart total: {:?} === tx total: {:?}", &cart.total, &body.total);
    }

    let created_at = chrono::NaiveDateTime::from_timestamp(
        chrono::Utc::now().timestamp(), 0);

    // 2b. Create pending Transaction details to send to shopping orders
    let mut tx = Transaction {
        id: format!("txn_pending_{}", body.order_id),
        subtotal: cart.subtotal,
        taxes: cart.taxes,
        payment_processing_fee: cart.payment_processing_fee,
        created_at: created_at,
        currency: Some(body.currency),
        charge_id: None,
        customer_id: None,
        order_id: None,
        payment_processor: Some(String::from("Paypal")),
        payment_method_id: None,
        payment_intent_id: None,
        refund_id: None,
        details: query.details,
    };

    // 3. Create order in DB via gm-shopping
    debug!("Writing order for user: {:?}, cart: {:?}", &cart.user_id, &cart.id);

    let create_order = rpc_create_order(
        AppState::httpClient(&req),
        &cart.id,
        tx.clone()
    ).await?;

    // update Transaction with orderId generated from shoppingservice
    tx.update_order_id(create_order.id.clone());

    /// 4. then return to front-end to finish Paypal transaction
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "order": create_order,
            "tx_receipt": tx
        })))
}




#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalConfirmOrderBody {
    paypal_response: PaypalResponse,
    transaction: Transaction,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalConfirmOrderQuery {
    cart_id: String,
}

pub async fn paypal_confirm_order(
    req: HttpRequest,
    query: Query<PaypalConfirmOrderQuery>,
    json: Json<PaypalConfirmOrderBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let paypal_response = body.paypal_response;
    let mut tx = body.transaction;
    let order_id = match tx.order_id.clone() {
        Some(oid) => oid,
        None => return Err(Error::from(
            PaypalError::BadInput(ErrJson::new("No order_id on transaction"))
        ))
    };
    debug!("received pending transaction: {:?}", &tx.id);
    debug!("received paypal_response: {:?}", &paypal_response.id);
    debug!("Confirming order: {:?}", &order_id);

    // 1. await all 3 futures to return concurrently, then send response
    let (
        conn,
        cart_clear,
        confirm_order,
    ) = match futures::join!(
        // f1: Retreive DB pool thread concurrently
        AppState::databaseActor(&req).send(GetPool::Postgres),
        // f2: Clear cart concurrently
        rpc_cart_clear(AppState::httpClient(&req), &query.cart_id),
        // f3: Confirm and update OrderStatus concurrently
        rpc_confirm_order(AppState::httpClient(&req), &order_id, &tx.id
        )
    ) {
        (f1, f2, f3) => (f1??, f2?, f3?) // unwrap Results
    };

    // 2. update tx with orderId and Paypal response
    tx.update_order_id(confirm_order.id.clone());
    tx.update_with_paypal_response(paypal_response);

    // 3. create payout_items
    let payout_items: Vec<PayoutItem> = to_payout_items(
        &conn,
        confirm_order.payout_items.clone().expect("missing payout_items on OrderDb in rpc_confirm_order()"),
        &tx.id,
        &tx.created_at,
        None
    );

    // 4. write both transaction and payout_items to DB in single transaction
    // for double-entry accounting.
    let (tx_result, pitems_result) = db::write_transaction_and_payout_items(
        &conn,
        &tx,
        &payout_items
    ).map_err(Error::from)?;

    debug!("tx: {:?}", &tx_result);
    debug!("pitems: {:?}", &pitems_result);
    debug!("cart was cleared: {:?}", cart_clear);

    // 6. Return http response with tx, order, and paypal_response
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "order": confirm_order,
            "tx_receipt": tx_result,
        })))

}