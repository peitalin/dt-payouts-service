use actix_web::{
    HttpRequest, HttpResponse,
    web, web::Query, web::Json,
    Error,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use futures::future::{
    Future,
    Either,
};

use crate::db;
use crate::db::GetPool;
use crate::models::{
    DbError,
    ErrJson,
    PayoutMethod,
    PayoutType
};
use crate::{AppState};




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadPayoutMethodQuery {
    payout_method_id: String
}

pub async fn read_payout_method(
    req: HttpRequest,
    query: Query<ReadPayoutMethodQuery>,
) -> Result<HttpResponse, Error> {

    let payout_method_id = query.into_inner().payout_method_id;
    debug!("payout_method_id: {:?}", &payout_method_id);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_method = db::read_payout_method(&conn, payout_method_id)?;

    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payout_method))
}




#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPayoutMethodBody {
    store_or_user_id: String,
    payout_processor: Option<String>, // Paypal, Adyen
    payout_type: Option<PayoutType>, // Paypal, Bank, Card
    payout_email: Option<String>, // paypal_email
    payout_processor_id: Option<String>, // some other payment ID
}

pub async fn set_payout_method(
    req: HttpRequest,
    json: Json<SetPayoutMethodBody>,
) -> Result<HttpResponse, Error> {

    // let payout_method = json.into_inner().payout_method;
    let body = json.into_inner();
    debug!("received body: {:?}", &body);
    let payee_id = body.store_or_user_id.clone();

    let payout_method = PayoutMethod::new(
        payee_id,
        body.payout_processor,
        body.payout_type,
        body.payout_email,
        body.payout_processor_id,
    );
    debug!("writing payout_method {:?}", &payout_method);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_method = db::insert_payout_method_by_payee_id(
        &conn,
        payout_method
    )?;

    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payout_method))
}

