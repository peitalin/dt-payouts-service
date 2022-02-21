use actix::{Actor, Addr};
use actix_web::{
    HttpRequest,
    HttpResponse,
    HttpMessage,
    web::Json,
    web::Query,
    Error,
};
use crate::AppState;
use crate::models::{
    // Errors
    ErrJson,
    AuthError,
    RpcError,
    PayeeType,
};
use crate::db;
use crate::db::{ GetPool };
use crate::rpc;


pub async fn read_affiliate_commissions(
    req: HttpRequest,
) -> Result<HttpResponse, Error> {

    debug!("retrieving summary of payouts for gm Platform");

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let platform_earnings = db::read_payout_item_history_summaries(
        &conn,
        "gm-platform",
        Some(vec![PayeeType::PLATFORM])
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "platformEarnings": platform_earnings,
        })))
}
