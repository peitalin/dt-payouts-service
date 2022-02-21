use actix_web::{
    HttpRequest, HttpResponse,
    web, web::Query, web::Json,
    Error,
};
use futures::{
    future,
    future::Either,
    Future,
};
use std::str::FromStr;

use crate::db;
use crate::db::GetPool;
use crate::models::{
    StripeError, DbError, ErrJson,
    Transaction,
    CartRpc,
    OrderDb,
    TxQuery,
    PayoutItem,
    Currency,
    PayoutSplit,
};
use crate::{AppState};
use crate::rpc::{
    rpc_get_cart,
    rpc_create_order,
    rpc_cart_clear,
    rpc_confirm_order,
};

// import trait for stripe requests
use gm::models::stripe;
use gm::models::stripe::{
    PaymentIntent,
    PaymentIntentCreateParams,
    PaymentIntentUpdateParams,
    PaymentIntentConfirmParams,
    PaymentIntentCaptureParams,
    PaymentIntentCancelParams,
    PaymentIntentListParams,
    List,
};
use crate::payment_clients::{
    SetupIntentMsg,
    PaymentIntentMsg,
    PaymentMethodMsg,
    ListMsg,
};

// POST /payment_intent
pub async fn create_payment_intent(
    req: HttpRequest,
    json: Json<PaymentIntentCreateParams>,
) -> Result<HttpResponse, Error> {

    let params = json.into_inner();

    let res: PaymentIntent = AppState::stripeActor(&req)
        .send(PaymentIntentMsg::Create(params))
        .await??;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(json!({
        "stripe_response": res,
    })))
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntentQuery {
    payment_intent_id: String
}

// GET /payment_intent
pub async fn retrieve_payment_intent(
    req: HttpRequest,
    query: Query<PaymentIntentQuery>,
) -> Result<HttpResponse, Error> {

    let payment_intent_id = query.into_inner().payment_intent_id;

    let res: PaymentIntent = AppState::stripeActor(&req)
        .send(PaymentIntentMsg::Retrieve(payment_intent_id))
        .await??;

    debug!("{:#?}", res);

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

// POST /payment_intent
pub async fn update_payment_intent(
    req: HttpRequest,
    query: Query<PaymentIntentQuery>,
    json: Json<stripe::PaymentIntentUpdateParams>,
) -> Result<HttpResponse, Error> {

    let payment_intent_id = query.into_inner().payment_intent_id;
    let params = json.into_inner();

    let res: PaymentIntent = AppState::stripeActor(&req)
        .send(PaymentIntentMsg::Update(payment_intent_id, params))
        .await??;

    debug!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

// POST /payment_intent
pub async fn confirm_payment_intent(
    req: HttpRequest,
    query: Query<PaymentIntentQuery>,
    json: Json<stripe::PaymentIntentConfirmParams>,
) -> Result<HttpResponse, Error> {

    let payment_intent_id = query.into_inner().payment_intent_id;
    let params = json.into_inner();
    println!("Payment Intent Confirm Params: {:#?}", params);

    let res: PaymentIntent = AppState::stripeActor(&req)
        .send(PaymentIntentMsg::Confirm(payment_intent_id, params))
        .await??;


    debug!("{:#?}", res);
    // do some DB stuff here
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

// POST /payment_intent
pub async fn capture_payment_intent(
    req: HttpRequest,
    query: Query<PaymentIntentQuery>,
    json: Json<stripe::PaymentIntentCaptureParams>,
) -> Result<HttpResponse, Error> {

    let payment_intent_id = query.into_inner().payment_intent_id;
    let params = json.into_inner();

    let res: PaymentIntent = AppState::stripeActor(&req)
        .send(PaymentIntentMsg::Capture(payment_intent_id, params))
        .await??;

    debug!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

// POST /payment_intent
pub async fn cancel_payment_intent(
    req: HttpRequest,
    query: Query<PaymentIntentQuery>,
    json: Json<stripe::PaymentIntentCancelParams>,
) -> Result<HttpResponse, Error> {

    let payment_intent_id = query.into_inner().payment_intent_id;
    let params = json.into_inner();

    let res: PaymentIntent = AppState::stripeActor(&req)
        .send(PaymentIntentMsg::Cancel(payment_intent_id, params))
        .await??;

    debug!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

// // GET /payment_intent
// pub async fn list_payment_intents(
//     req: HttpRequest,
//     json: Json<stripe::PaymentIntentListParams>,
// ) -> Result<HttpResponse, Error> {

//     let params = json.into_inner();

//     let res: List<PaymentIntent> = AppState::stripeActor(&req)
//         .send(ListMsg::<PaymentIntent, PaymentIntentListParams>::PaymentIntents(
//             params
//         ))
//         .await??;

//     debug!("{:#?}", res);
//     Ok(HttpResponse::Ok()
//         .content_type("application/json")
//         .json(res))
// }
