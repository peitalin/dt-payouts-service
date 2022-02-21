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
};
use std::str::FromStr;
use gm::utils::dates::from_datetimestr_to_naivedatetime;
use crate::pricing::PaymentFees;
use crate::rpc::rpc_save_payment_method;

use crate::AppState;
use crate::models::{
    PaypalResponse,
    PaypalError,
    Transaction,
    TransactionAggregates,
    TransactionId,
    PayoutPeriod,
    ErrJson,
    StripeError,
    Currency,
    PayoutSplit,
    to_payout_items,
    TxQuery,
    PayoutItem,
    OrderItemRpc,
};
use crate::models::paypal::{
    PaypalPurchaseUnits,
    PaypalCaptures,
};
use crate::models::connection::{
    ConnectionQuery,
    Edge,
    Connection,
    PageInfo,
};
use crate::db;
use crate::db::{ GetPool };
use crate::rest::affiliates::convert_buyer_affiliate;


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadManyTransactionsBody {
    transaction_ids: Vec<String>,
}


pub async fn read_many_transactions_by_ids(
    req: HttpRequest,
    json: Json<ReadManyTransactionsBody>,
) -> Result<HttpResponse, Error> {

    let transaction_ids = json.into_inner().transaction_ids;
    debug!("retrieving transaction_ids: {:?}", &transaction_ids);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;


    let tx_result = db::read_many_transactions_by_ids(
        &conn,
        transaction_ids
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(tx_result))
}



#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadRecentTransactions {
    count: i64,
}

pub async fn read_recent_transactions(
    req: HttpRequest,
    json: Json<ReadRecentTransactions>,
) -> Result<HttpResponse, Error> {

    let count = json.into_inner().count;
    debug!("retrieving recent transactions... count: {:?}", &count);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let tx_result = db::read_recent_transactions(
        &conn,
        count
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(tx_result))
}


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadTransactionConnectionBody {
    month: i32,
    year: i32,
    query: ConnectionQuery,
}

pub async fn read_transaction_connection(
    req: HttpRequest,
    json: Json<ReadTransactionConnectionBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let sort_ascending = body.query.sortAscending.clone();

    let payout_period: PayoutPeriod = PayoutPeriod::new(
        body.year,
        body.month
    ).map_err(Error::from)?;

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let (vecTx, numPages, isLastPage) = db::read_transactions_paginate_cursor(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        body.query
    ).map_err(Error::from)?;

    let agg: TransactionAggregates = db::read_transaction_aggregates(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        sort_ascending.unwrap_or(false),
    );

    let endCursor = match agg.count {
        0 => None,
        _ => Some(base64::encode(&format!("created_at:{}", agg.created_at)))
    };

    let connection = Connection::<Transaction> {
        pageInfo: PageInfo {
            endCursor: endCursor,
            isLastPage: isLastPage,
            totalPages: Some(numPages),
        },
        totalCount: Some(agg.count),
        totalAmount: Some(agg.subtotal_sum),
        totalFees: Some(agg.fees_total),
        edges: vecTx.into_iter().map(|tx| {
            let edgeCursor = format!("created_at:{:?}", &tx.created_at);
            Edge {
                cursor: Some(base64::encode(&edgeCursor)),
                node: tx
            }
        }).collect::<Vec<Edge<Transaction>>>()
    };

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(connection))
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderPaymentParams {
    pub order_id: String,
    pub currency: String,
    pub subtotal: i32,
    pub taxes: i32,
    pub total: i32,
    pub payment_processor_data: String,
    pub order_items_rpc: Vec<OrderItemRpc>
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize)]
struct StripePaymentProcessorData {
    pub payment_method: String,
    pub save_payment_method: bool,
    pub customer_id: Option<String>,
}


#[serde(rename_all = "camelCase")]
#[derive(Deserialize)]
struct PaypalPaymentProcessorData {
    pub order_id: String,
    pub payer_id: Option<String>,
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub created_at: chrono::NaiveDateTime
}


pub async fn record_frontend_tx(
    req: HttpRequest,
    json: Json<OrderPaymentParams>
) -> Result<HttpResponse, Error> {

    // Parse the payment processor data
    let params = json.into_inner();
    debug!("incoming params: {:?}", params);

    let paypal_response: PaypalResponse = serde_json::from_str(
        &params.payment_processor_data
    ).map_err(|e| PaypalError::DeserializationError(errJson!(e)))?;

    debug!("1. paypal_response: {:?}", paypal_response);

    let purchase_unit: PaypalPurchaseUnits = paypal_response.purchase_units
        .clone()
        .expect("purchase_units field missing")
        .into_iter().next()
        .expect("purchase_units[0] missing!");

    let captures: PaypalCaptures = purchase_unit.payments.captures
        .clone()
        .expect("puchase_unit.payments.captures missing!")
        .into_iter().next()
        .expect("purchase_unit.payments.captures[0] missing!");

    let tx_id = format!("txn_{}", paypal_response.id.clone());

    let created_at = paypal_response.create_time.unwrap_or(
        chrono::NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0)
    );

    // convert buyer-affiliate and retrieve userId if there is one.
    let buyer_affiliate_user_id = convert_buyer_affiliate(&req, params.order_id.clone()).await;
    debug!("buyer_affiliate_user_id: {:?}", &buyer_affiliate_user_id);

    // DB pool
    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    // Create payout_items first, before transaction
    let payout_items: Vec<PayoutItem> = to_payout_items(
        &conn,
        params.order_items_rpc.clone(),
        &tx_id,
        &created_at,
        buyer_affiliate_user_id
    );

    // seller pays payment_processing_fee for each orderItem:
    // this figure is for tx reference only, not payoutItems
    // sum over transaction fees on the PayoutItems
    let payment_proc_fee = payout_items
        .iter()
        .fold(0, |acc: i32, pitem: &PayoutItem| {
            acc + pitem.payment_processing_fee
        });

    // Create Transaction details
    let tx = Transaction {
        id: tx_id,
        subtotal: params.subtotal,
        taxes: params.taxes,
        payment_processing_fee: payment_proc_fee,
        created_at: created_at,
        currency: Some(Currency::USD),
        // currency: Currency::from_str(&purchase_unit.amount.currency_code).ok(),
        charge_id: Some(captures.id),
        customer_id: paypal_response.payer.clone().map(|y| y.payer_id),
        order_id: Some(params.order_id),
        payment_processor: Some("Paypal".to_string()),
        payment_method_id: paypal_response.payer.map(|y| y.payer_id),
        payment_intent_id: None,
        refund_id: None,
        details: None,
    };

    debug!("2. transaction object: {:?}", tx);

    // Write both transaction and payout_items to DB in single transaction
    // for double-entry accounting.
    let (_tx_result, _pitems_result) = db::write_transaction_and_payout_items(
        &conn,
        &tx,
        &payout_items
    ).map_err(Error::from)?;

    // Return http response with Transaction object
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "transaction": tx,
    })))
}


#[test]
fn deserializes_confirm_paypal_transaction() {

    let test_str = r#"
    {
        "create_time":"2020-05-07T08:05:58Z",
        "update_time":"2020-05-07T08:06:16Z",
        "id":"58N39849K48981646",
        "intent":"CAPTURE",
        "status":"COMPLETED",
        "payer":{
            "email_address":"nadia@gm.com",
            "payer_id":"ATSY6LZTPHUPS",
            "address":{
                "country_code":"US"
            },
            "name":{
                "given_name":"Nadia",
                "surname":"Supernova"
            }
        },
        "purchase_units":[
            {
                "reference_id":"default",
                "amount":{
                    "value":"8.11",
                    "currency_code":"USD"
                },
                "payee":{
                    "email_address":"s4143868-facilitator@gmail.com",
                    "merchant_id":"EKBZYVBPSZENE"
                },
                "shipping":{
                    "name":{
                        "full_name":"Nadia Supernova"
                    },
                    "address":{
                        "address_line_1":"1 Main St",
                        "admin_area_2":"San Jose",
                        "admin_area_1":"CA",
                        "postal_code":"95131",
                        "country_code":"US"
                    }
                },
                "payments":{
                    "captures":[
                        {
                            "status":"COMPLETED",
                            "id":"217957193E7519037",
                            "final_capture":true,
                            "create_time":"2020-05-07T08:06:16Z",
                            "update_time":"2020-05-07T08:06:16Z",
                            "amount":{
                                "value":"8.11",
                                "currency_code":"USD"
                            },
                            "seller_protection":{
                                "status":"ELIGIBLE",
                                "dispute_categories":[
                                    "ITEM_NOT_RECEIVED",
                                    "UNAUTHORIZED_TRANSACTION"
                                ]
                            },
                            "links":[
                                {
                                    "href":"https://api.sandbox.paypal.com/v2/payments/captures/217957193E7519037",
                                    "rel":"self",
                                    "method":"GET",
                                    "title":"GET"
                                },
                                {
                                    "href":"https://api.sandbox.paypal.com/v2/payments/captures/217957193E7519037/refund",
                                    "rel":"refund",
                                    "method":"POST",
                                    "title":"POST"
                                },
                                {
                                    "href":"https://api.sandbox.paypal.com/v2/checkout/orders/58N39849K48981646",
                                    "rel":"up",
                                    "method":"GET",
                                    "title":"GET"
                                }
                            ]
                        }
                    ]
                }
            }
        ],
        "links":[
            {
                "href":"https://api.sandbox.paypal.com/v2/checkout/orders/58N39849K48981646",
                "rel":"self",
                "method":"GET",
                "title":"GET"
            }
        ]
    }
    "#;

    let ppd = serde_json::from_str::<PaypalResponse>(test_str);

    match ppd {
        Ok(p) => assert_eq!(
            p.id,
            String::from("58N39849K48981646")
        ),
        Err(e) => panic!(e.to_string()),
    }
}
