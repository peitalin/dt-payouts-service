use actix::{Actor};
use actix_web::{
    HttpRequest,
    HttpResponse,
    web::Json,
    web::Query,
    Error,
};
use futures::{
    future::Either,
    Future,
};

use crate::AppState;
use crate::models::{
    PaypalError, DbError, ErrJson,
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
    PaypalRefreshResponse,
};

use crate::db;
use crate::db::{ GetPool };


pub mod orders;
pub mod payouts;
pub use orders::*;
pub use payouts::*;



pub async fn test_paypal_token_refresh(
    req: HttpRequest,
) -> Result<HttpResponse, Error> {

    let json_res = AppState::paypalActor(&req)
                        .send(PaypalAuthRefresh::RefreshToken)
                        .await??;

    let refresh_res = serde_json::from_value::<PaypalRefreshResponse>(json_res)
        .map_err(Error::from)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(refresh_res))
}


pub async fn test_paypal_read_invoice(
    req: HttpRequest,
) -> Result<HttpResponse, Error> {

    let url = PaypalRequest::Get::<serde_json::Value>(
        String::from("/v1/invoicing/invoices?page=3&page_size=4&total_count_required=true")
    );

    let response = AppState::paypalActor(&req)
                    .send(url)
                    .await?;

    debug!("Response: {:?}", response);

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("{:#?}", response.unwrap())))
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalRefundQuery {
    refund_id: String
}

pub async fn test_paypal_refund(
    req: HttpRequest,
    query: Query<PaypalRefundQuery>
) -> Result<HttpResponse, Error> {

    let url = PaypalRequest::Get::<serde_json::Value>(
        format!("/v2/payments/refunds/{}", query.refund_id)
    );

    let response = AppState::paypalActor(&req)
                    .send(url)
                    .await?;

    println!("Response: {:?}", response);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("{:#?}", response.unwrap())))
}
