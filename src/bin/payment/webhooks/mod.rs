pub mod stripe_refunds;
pub mod paypal_refunds;

pub use stripe_refunds::*;
pub use paypal_refunds::*;

use actix_web::{HttpResponse, HttpRequest, Error, web::Json};
use futures::future::Future;
// diesel
use diesel::prelude::*; // need for table_name proc macro
use gm::db::schema::refunds;
use std::f64;
use crate::models::Refund;

use crate::AppState;
use crate::db;
use crate::db::GetPool;




pub fn handle_stripe_refund_webhook(
    req: HttpRequest,
    json: Json<StripeRefundResponse>,
) -> HttpResponse {

    debug!("req: {:?}", req);
    debug!("headers: {:?}", req.headers());
    debug!("uri: {:?}", req.uri());
    let stripe_refund = json.into_inner();
    debug!("stripe_refund: {:?}", &stripe_refund);

    HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "refund_result": stripe_refund,
        }))
}


pub fn handle_paypal_refund_webhook(
    req: HttpRequest,
    json: Json<PaypalRefundResponse>,
) -> HttpResponse {

    debug!("req: {:?}", req);
    debug!("headers: {:?}", req.headers());
    debug!("uri: {:?}", req.uri());
    let paypal_refund = json.into_inner();
    debug!("paypal_refund: {:?}", &paypal_refund);

    HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "paypal_refund": paypal_refund,
        }))
}

