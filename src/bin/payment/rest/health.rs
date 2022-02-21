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
use crate::AppState;
use crate::db::{ GetPool };

// GET /_health
pub async fn retrieve_health_status(
req: HttpRequest
) -> Result<HttpResponse, Error> {

  let _conn = AppState::databaseActor(&req)
              .send(GetPool::Postgres)
              .await??;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(json!({
        "isHealthy": true
    })))
}