use actix_web::{
    HttpRequest, HttpResponse,
    web, web::Query, web::Json,
    Error,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use futures::future::Future;

use crate::db;
use crate::db::GetPool;
use crate::models::{
    DbError,
    ErrJson,
    StripeError,
};
use crate::{AppState};
// import stripe traits to enable Requests
use gm::models::stripe;
use gm::models::stripe::{
    PaymentMethod,
    List,
};
use crate::payment_clients::{
    PaymentMethodMsg,
    ListMsg,
};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadManyPaymentMethodsBody {
    payment_method_ids: Vec<String>
}

pub async fn read_many_payment_methods(
    req: HttpRequest,
    json: Json<ReadManyPaymentMethodsBody>,
) -> Result<HttpResponse, Error> {

    let payment_method_ids = json.into_inner().payment_method_ids;
    debug!("payment_method_ids: {:?}", &payment_method_ids);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payment_methods = db::read_many_payment_methods(&conn, payment_method_ids)?;
    // Returns a list of PaymentMethodDbs (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payment_methods))
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadPaymentMethodsByUserIdBody {
    user_id: String
}

pub async fn read_payment_methods_by_user_id(
    req: HttpRequest,
    json: Json<ReadPaymentMethodsByUserIdBody>,
) -> Result<HttpResponse, Error> {

    let user_id = json.into_inner().user_id;
    debug!("user_id: {:?}", &user_id);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payment_methods = db::read_payment_methods_by_user_id(&conn, &user_id)?;
    // Returns a list of PaymentMethodDbs (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payment_methods))
}


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDetachPaymentMethodBody {
    user_id: String,
    payment_method_id: String,
}

pub async fn delete_and_detach_payment_method(
    req: HttpRequest,
    json: Json<DeleteDetachPaymentMethodBody>,
) -> Result<HttpResponse, Error> {

    let params = json.into_inner();
    let payment_method_id = params.payment_method_id;
    let user_id = params.user_id;
    debug!("detaching and deleting payment_method_id: {:?}", &payment_method_id);
    debug!("for user: {:?}", &user_id);

    let conn = AppState::databaseActor(&req)
                        .send(GetPool::Postgres)
                        .await??;

    let _stripe_pm_response: PaymentMethod = AppState::stripeActor(&req)
        .send(PaymentMethodMsg::Detach(payment_method_id.clone()))
        .await??;

    // if stripe detachment to fail (already attached to customer)
    // then this never executes because of ? operator above
    let payment_methods = db::delete_payment_method(
        &conn,
        &payment_method_id,
        &user_id
    )?;

    // Returns a list of PaymentMethodDbs (camelcased)
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(payment_methods))

}
