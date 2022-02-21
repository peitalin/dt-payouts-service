use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::prelude::*;
// Modules in this folder
pub mod schema;
pub mod actor;

///////////////////////////////
/// Establish PostgreSQL Connection
//////////////////////////////

/// establish_connection_pg() -> PgConnection
pub fn establish_connection_pg(database_env_var: &str) -> PgConnection {
    // Get Database login details from .env
    dotenv::dotenv().ok();
    let database_url = std::env::var(database_env_var)
        .expect(&format!("{} must be set in .env", database_env_var));

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_postgres_pool(database_env_var: &str, pool_limit: u32) -> Pool<ConnectionManager<PgConnection>> {
    // Get Database login details from .env
    dotenv::dotenv().ok();
    let database_url = std::env::var(database_env_var)
            .expect(&format!("{} must be set in .env", database_env_var));

    // r2d2 connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    // let pool = diesel::r2d2::Pool::new(manager).unwrap();
    let pool = Pool::builder()
        .max_size(pool_limit)
        .build(manager); // Result<Pool<M>, Error>

    let pool: Pool<ConnectionManager<PgConnection>> = match pool {
        Ok(pool) => pool,
        Err(e) => {
            warn!("Failed to create pool: {:?}", e.to_string());
            panic!("Check if PostgreSql is accessible at: {:?}",
                std::env::var(database_env_var));
        },
    };
    pool
}
