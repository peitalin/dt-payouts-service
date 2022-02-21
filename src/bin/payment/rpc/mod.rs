/// Remote Network Calls with REST
mod shopping;
mod user;
mod affiliate;

pub use shopping::*;
pub use user::*;
pub use affiliate::*;

use crate::AppState;
use crate::endpoints::Endpoint;
use crate::models::errors::{
    RpcError,
    ErrJson,
};
use crate::models::{
    CartRpc,
    TxQuery,
    Transaction,
    OrderDb,
    Currency,
};
use crate::rpc;
use futures::future::Future;
use actix_web::{HttpResponse, HttpRequest, Error, web::Query};



pub async fn rpc_test_handler(
    req: HttpRequest
) -> Result<HttpResponse, Error> {

    debug!("req: {:?}", req);
    debug!("headers: {:?}", req.headers());
    debug!("uri: {:?}", req.uri());
    debug!("requesting endpoint: {}", Endpoint::Shopping("/test"));

    let mut response = AppState::from(&req).http_client
                    .get(Endpoint::Shopping("/test").as_url())
                    .send()
                    .await?;

    let bytes = response.body().await?;

    let shopping_msg = std::str::from_utf8(&bytes)
        .map(|s| String::from(s))
        .map_err(|e| Error::from(RpcError::Shopping(errJson!(e))))?;

    Ok(HttpResponse::Ok()
        .content_type("application_json")
        .json(json!({
            "status": "OK",
            "message": "Test response for gm-payment service",
            "message2": &shopping_msg,
        })))
}



/////////////////// TESTS ///////////////////

pub async fn rpc_test_create_order(
    req: HttpRequest,
    query: Query<TxQuery>
) -> Result<HttpResponse, Error> {

    let cart_id = query.into_inner().cart_id;
    let route = format!("/orders/confirm?cart_id={}", cart_id);
    debug!("requesting endpoint: {}", route);

    let test_tx = Transaction {
       id: String::from("pi_1ExzOVKqy1M9WH1_TEST_TX"),
       subtotal: 668,
       taxes: 10,
       payment_processing_fee: 8,
       currency: Some(Currency::USD),
       customer_id: Some(String::from("test_customer_id")),
       order_id: Some(String::from("test_order_id")),
       created_at: chrono::NaiveDateTime::from_timestamp(1_500_000_000, 0),
       payment_processor: Some(String::from("Stripe")),
       payment_method_id: Some(String::from("pm_1ExzOUKqy1M9WH1Dh3JS108d")),
       payment_intent_id: Some(String::from("pi_123123123123123123123")),
       charge_id: None,
       refund_id: None,
       details: None,
    };

    let mut response = AppState::from(&req).http_client
                    .post(Endpoint::Shopping(&route).as_url())
                    .send_json(&json!({
                        "tx": test_tx
                    }))
                    .await?;

    let bytes = response.body().await?;


    let order_db = serde_json::from_slice::<OrderDb>(&bytes)
        .map_err(|e| Error::from(RpcError::Shopping(errJson!(e))))?;

    Ok(HttpResponse::Ok()
        .content_type("application_json")
        .json(json!({
            "status": "OK",
            "message": "Test response for gm-payment service",
            "order": &order_db,
        })))
}


pub async fn rpc_test_cart_handler(
    req: HttpRequest,
    query: Query<TxQuery>
) -> Result<HttpResponse, Error> {

    debug!("req: {:?}", req);
    debug!("headers: {:?}", req.headers());
    debug!("uri: {:?}", req.uri());

    let cart_id = query.into_inner().cart_id;
    let route = format!("/carts/total?cart_id={}", cart_id);
    debug!("requesting endpoint: {}", route);

    let mut response = AppState::from(&req).http_client
                    .get(Endpoint::Shopping(&route).as_url())
                    .send()
                    .await?;


    let bytes = response.body().await?;

    let cart_db = serde_json::from_slice::<CartRpc>(&bytes)
        .map_err(|e| Error::from(RpcError::Shopping(errJson!(e))))?;

    Ok(HttpResponse::Ok()
        .content_type("application_json")
        .json(json!({
            "status": "OK",
            "message": "Test response for gm-payment service",
            "cart": &cart_db,
        })))
}


pub async fn rpc_test_id(
    req: HttpRequest
) -> Result<HttpResponse, Error> {

    debug!("req: {:?}", req);
    debug!("headers: {:?}", req.headers());
    debug!("uri: {:?}", req.uri());
    debug!("requesting endpoint: {}", Endpoint::User("/id"));

    let auth_info = rpc::rpc_get_auth_info(
                        &AppState::from(&req).http_client,
                        &req,
                    ).await?;

    Ok(HttpResponse::Ok()
        .content_type("application_json")
        .json(json!({
            "status": "OK",
            "message": "Test response for gm-user service",
            "auth_info": auth_info,
        })))
}

