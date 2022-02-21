/// Remote Network Calls with REST
use crate::AppState;
use crate::endpoints::Endpoint;
use crate::models::errors::{
    RpcError,
    ErrJson,
};
use crate::models::{
    CartRpc,
    OrderDb,
    Refund,
    Transaction,
    UserId,
    UpdateUserProfile,
    UpdateStripeCustomerId, };
use futures::future::Future;
use actix_web::{HttpResponse, HttpRequest, Error, web::Query};



pub async fn rpc_get_cart(
    client: &actix_web::client::Client,
    cart_id: &str
) -> Result<CartRpc, Error> {

    let route = format!("/carts/total?cart_id={}", cart_id);
    debug!("requesting endpoint: {}", route);

    let mut response = client.get(Endpoint::Shopping(&route).as_url())
                    .send()
                    .await?;

    let bytes = response.body().await?;

    serde_json::from_slice::<CartRpc>(&bytes)
        .map_err(|e| Error::from(RpcError::Shopping(errJson!(e))))

}


pub async fn rpc_create_order(
    client: &actix_web::client::Client,
    cart_id: &str,
    transaction: Transaction,
) -> Result<OrderDb, Error> {

    // NOTE: avoid nested impl Futures, use Box<Future>. Rustc compiler bugs.
    // Maybe be fixed since rust v1.36.
    let route = format!("/orders/create?cart_id={}", cart_id);
    debug!("sending transaction to endpoint: {}", route);
    debug!("transaction: {:?}", transaction);

    let mut response = client
                    .post(Endpoint::Shopping(&route).as_url())
                    .send_json(&json!({
                        "transaction": transaction
                    }))
                    .await?;

    let bytes = response.body().await?;

    serde_json::from_slice::<OrderDb>(&bytes)
        .map_err(|e| Error::from(RpcError::Shopping(errJson!(e))))

}

pub async fn rpc_confirm_order(
    client: &actix_web::client::Client,
    order_id: &str,
    transaction_id: &str,
) -> Result<OrderDb, Error> {

    let route = format!(
        "/orders/confirm?order_id={}&transaction_id={}",
        order_id,
        transaction_id,
    );
    debug!("requesting endpoint: {}", route);

    let mut response = client.post(Endpoint::Shopping(&route).as_url())
                    .send()
                    .await?;

    let bytes = response.body().await?;

    // order item ids: Vec<String>
    serde_json::from_slice::<OrderDb>(&bytes)
        .map_err(|e| Error::from(RpcError::Shopping(errJson!(e))))

}


pub async fn rpc_cart_clear(
    client: &actix_web::client::Client,
    cart_id: &str
) -> Result<CartRpc, Error> {

    let route = format!("/carts/clear?cart_id={}", cart_id);
    debug!("requesting endpoint: {}", route);

    let mut response = client.delete(Endpoint::Shopping(&route).as_url())
                    .send()
                    .await?;

    let bytes = response.body().await?;

    serde_json::from_slice::<CartRpc>(&bytes)
        .map_err(|e| Error::from(RpcError::Shopping(errJson!(e))))
}


// POST /orders/refund/update
// body: { order_id: ID, transaction: Transaction, refund: Refund }
pub async fn rpc_update_orders_for_refunds(
    client: &actix_web::client::Client,
    transaction: Transaction,
    refund: Refund,
) -> Result<OrderDb, Error> {

    let route = format!("/orders/refund/update");
    debug!("requesting endpoint: {}", route);

    let mut response = client.post(Endpoint::Shopping(&route).as_url())
                    .send_json(&json!({
                        "order_id": &transaction.order_id,
                        "transaction": transaction,
                        "refund": refund
                    }))
                    .await?;

    let bytes = response.body().await?;

    serde_json::from_slice::<OrderDb>(&bytes)
        .map_err(|e| Error::from(RpcError::Shopping(errJson!(e))))
}