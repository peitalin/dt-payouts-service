//// External Imports
use actix::{Actor, Handler, SyncContext, Message};
use actix_web::{
    http::StatusCode,
    error::ResponseError,
    HttpResponse,
    HttpRequest,
};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use failure::Error;

/////////////////////////
/// AppState Actor
/////////////////////////

pub struct DatabaseActor {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

// Implement Actor traits for DatabaseActor
impl Actor for DatabaseActor {
    type Context = SyncContext<Self>;
}

impl DatabaseActor {
    pub fn new(
        db_pool: Pool<ConnectionManager<PgConnection>>,
    ) -> DatabaseActor {

        DatabaseActor {
            pool: db_pool,
        }
    }
}

////////// GetContext Message Handler ////////////
// Handle messages that tell DatabaseActor to share it's DB pool.
// Allow DatabaseActor to share it's DB pool to other processes
// so other actors do not need to create their own pool.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GetPool {
    Postgres,
    // ShoppingDB,
}
// Actix Message Trait impl for Db queries
impl Message for GetPool {
    type Result = Result<PooledConnection<ConnectionManager<PgConnection>>, GetPoolError>;
}
impl Handler<GetPool> for DatabaseActor {
    type Result = Result<PooledConnection<ConnectionManager<PgConnection>>, GetPoolError>;

    fn handle(&mut self, msg: GetPool, _ctx: &mut SyncContext<Self>) -> Self::Result {
        match msg {
            GetPool::Postgres => {
                self.pool.get()
                    .map_err(|e| GetPoolError::PoolConnection(e.to_string()))
            },
        }
    }
}

#[derive(Debug, Serialize, Fail)]
pub enum GetPoolError {
    #[fail(display = "DB Pool Connection error: {}", _0)]
    PoolConnection(String),
}

impl ResponseError for GetPoolError {
    fn error_response(&self) -> HttpResponse {
       match self {
            GetPoolError::PoolConnection(s) => {
                warn!("{}", s);
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
       }
    }
}
