use actix_web::{
    http,
    HttpRequest, HttpResponse,
    web,
    web::Json,
    web::Query,
    Error,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use futures::{
    future,
    future::Either,
    Future,
};
use std::str::FromStr;

use crate::db;
use crate::db::{
    GetPool, GetPoolError,
};
use crate::models::{
    StripeError,
    ErrJson,
    UpdateStripeCustomerId,
};
use crate::{AppState};
use crate::rpc::rpc_update_stripe_customer_id_for_user;

// import trait for stripe requests
use gm::models::stripe;
use gm::models::stripe::{
    CustomerCreateParams,
    CustomerUpdateParams,
    CustomerListParams,
    Customer,
    Deleted,
    List,
};
use gm::models::stripe::ids::CustomerId;
use crate::payment_clients::{
    CustomerMsg,
    ListMsg,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerIdQuery {
    /// Customer ID, starts with `cus_` prefix
    pub customer_id: String,
    /// Specifies which fields in the response should be expanded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<Vec<String>>,
}

//////////////////////////
/// Customer API handlers
/////////////////////////

pub async fn create_customer(
    req: HttpRequest,
    data: Json<CustomerCreateParams>,
) -> Result<HttpResponse, Error> {

    let customerData: CustomerCreateParams = data.into_inner();
    debug!("incoming input {:#?}", customerData);
    // Send Stripe Customer create request
    let res: stripe::Customer = AppState::stripeActor(&req)
        .send(CustomerMsg::Create(customerData))
        .await??;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}


pub async fn update_customer(
    req: HttpRequest,
    query: Query<CustomerIdQuery>,
    json: Json<CustomerUpdateParams>,
) -> Result<HttpResponse, Error> {

    let params: CustomerUpdateParams = json.into_inner();
    let q = query.into_inner();

    let customer_id = stripe::CustomerId::from_str(&q.customer_id)
        .map_err(|e| Error::from(StripeError::Customer(errJson!(e))))?;

    let res: stripe::Customer = AppState::stripeActor(&req)
        .send(CustomerMsg::Update(
            customer_id.to_string(),
            params
        ))
        .await??;

    println!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

pub async fn retrieve_customer(
    req: HttpRequest,
    query: Query<CustomerIdQuery>,
) -> Result<HttpResponse, Error> {

    let customer_id = stripe::CustomerId::from_str(&query.customer_id)
        .map_err(|e| Error::from(StripeError::Customer(errJson!(e))))?;

    let res: stripe::Customer = AppState::stripeActor(&req)
        .send(CustomerMsg::Retrieve(
            customer_id.to_string()
        ))
        .await??;

    println!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

pub async fn delete_customer(
    req: HttpRequest,
    query: Query<CustomerIdQuery>,
) -> Result<HttpResponse, Error> {

    let q = query.into_inner();
    let customer_id = stripe::CustomerId::from_str(&q.customer_id)
        .map_err(|e| Error::from(StripeError::Customer(errJson!(e))))?;

    let res = AppState::stripeActor(&req)
        .send(CustomerMsg::Delete(
            customer_id.to_string()
        ))
        .await??;

    debug!("{:#?}", res);
    // let deleted: Deleted<CustomerId> = Deleted::<CustomerId>::from(res);
    let deleted = res; // wrong type, need to implement generics for delete

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(deleted))
}

// pub async fn list_all_customers(
//     req: HttpRequest,
//     query: Query<CustomerListParams>,
// ) -> Result<HttpResponse, Error> {

//     // pub struct CustomerListParams<'a> {
//     //     pub created: Option<RangeQuery<Timestamp>>,
//     //     pub email: Option<String>,
//     //     pub ending_before: Option<String>,
//     //     pub limit: Option<i64>,
//     //     pub starting_after: Option<String>,
//     // }

//     let params: CustomerListParams = query.into_inner();

//     let customers: List<Customer> = AppState::stripeActor(&req)
//         .send(ListMsg::Customers(params))
//         .await??;

//     println!("{:#?}", customers);
//     Ok(HttpResponse::Ok()
//         .content_type("application/json")
//         .json(customers))
// }




