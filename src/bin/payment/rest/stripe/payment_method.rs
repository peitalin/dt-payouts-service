use actix_web::{
    HttpRequest, HttpResponse,
    web, web::Query, web::Json,
    Error,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use futures::future::Future;

use crate::db;
use crate::{AppState};
use crate::models::{StripeError, ErrJson};
// import trait for stripe requests
use gm::models::stripe;
use gm::models::stripe::{
    PaymentMethod,
    PaymentMethodsListParams,
    List,
};

use crate::payment_clients::{
    PaymentMethodMsg,
    ListMsg,
};

/// 1. Create Payment Method
pub async fn create_payment_method(
    req: HttpRequest,
    json: Json<stripe::PaymentMethodCreateParams>,
) -> Result<HttpResponse, Error> {

    // pub struct CreatePaymentMethodParams {
    //     pub type: PaymentMethodType,
    //     pub card: CreatePaymentMethodCardParams,
    //     pub billing_getails: Option<BillingDetails>,
    //     pub metadata: Option<HashMap<String, String>>,
    // }

    // pub struct CreatePaymentMethodCardParams {
    //     pub exp_month: String, // eg. "12"
    //     pub exp_year: String,  // eg. "17" or 2017"
    //     pub number: String,       // card number
    //     pub name: Option<String>, // cardholder's full name
    //     pub cvc: Option<String>,  // card security code
    // }

    let payment_method_params = json.into_inner();
    debug!("payment_method_params: {:#?}", payment_method_params);

    let res: PaymentMethod = AppState::stripeActor(&req)
        .send(PaymentMethodMsg::Create(payment_method_params))
        .await??;

    println!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}


/// 2. Update Payment Method

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentMethodQuery {
    pub id: String,
}

pub async fn update_payment_method(
    req: HttpRequest,
    query: Query<PaymentMethodQuery>,
    json: Json<stripe::PaymentMethodUpdateParams>
) -> Result<HttpResponse, Error> {

    let params = json.into_inner();
    let query = query.into_inner();
    debug!("payment_method: {:#?}", query);

    let res: PaymentMethod = AppState::stripeActor(&req)
        .send(PaymentMethodMsg::Update(
            query.id,
            params,
        ))
        .await??;

    println!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

/// 3. Retreive Payment Method
pub async fn retrieve_payment_method(
    req: HttpRequest,
    query: Query<stripe::PaymentMethodRetrieveParams>,
) -> Result<HttpResponse, Error> {

    let query = query.into_inner();
    debug!("payment_method_id: {:#?}", query.payment_method_id);

    let res: PaymentMethod = AppState::stripeActor(&req)
        .send(PaymentMethodMsg::Retrieve(
            query.payment_method_id
        ))
        .await??;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

/// 4. Attach Payment Method
pub async fn attach_payment_method(
    req: HttpRequest,
    query: Query<PaymentMethodQuery>,
    json: Json<stripe::PaymentMethodAttachParams>
) -> Result<HttpResponse, Error> {

    // pub struct AttachPaymentMethodsParams {
    //     pub customer: String,
    // }
    /// customer is a customer ID
    /// E.G: customer: cus_F8zWIRLoprqNcL

    let query = query.into_inner();
    let body = json.into_inner();

    let pm_response: PaymentMethod = AppState::stripeActor(&req)
        .send(PaymentMethodMsg::Attach(
            query.id,
            body
        ))
        .await??;

    println!("{:#?}", pm_response);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(pm_response))
}

/// 5. Detach Payment Method
pub async fn detach_payment_method(
    req: HttpRequest,
    query: Query<PaymentMethodQuery>,
) -> Result<HttpResponse, Error> {

    let query = query.into_inner();

    let pm_response: PaymentMethod = AppState::stripeActor(&req)
        .send(PaymentMethodMsg::Detach(
            query.id
        ))
        .await??;

    println!("{:#?}", pm_response);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(pm_response))
}

// /// 6. List Payment Methods
// pub async fn list_payment_methods(
//     req: HttpRequest,
//     json: Json<stripe::PaymentMethodsListParams>,
// ) -> Result<HttpResponse, Error> {

//     // pub struct ListPaymentMethodsParams {
//     //     pub customer: CustomerId,
//     //     pub type: String,
//     //     pub ending_before: Option<String>,
//     //     pub limit: Option<i32>,
//     //     pub starting_after: Option<String>,
//     // }

//     let params = json.into_inner();

//     let res: List<PaymentMethod> = AppState::stripeActor(&req)
//         .send(ListMsg::<PaymentMethod, PaymentMethodsListParams>::PaymentMethods(
//             params
//         ))
//         .await??;

//     println!("{:#?}", res);
//     Ok(HttpResponse::Ok()
//         .content_type("application/json")
//         .json(res))
// }
