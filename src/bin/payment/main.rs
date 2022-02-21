#![allow(unused_imports)]
#![allow(unused_doc_comments)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![recursion_limit = "128"]

///////////////////////
//// External Imports
///////////////////////

// Actix: Actors
extern crate actix;
extern crate actix_rt;
extern crate actix_web;
extern crate base64;
extern crate dotenv;
#[macro_use] extern crate diesel;
#[macro_use] extern crate failure;
#[macro_use] extern crate json;
#[macro_use] extern crate log;
extern crate pretty_env_logger;
// Serde: Deserialization
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde;
extern crate rand;


#[macro_use]
extern crate lazy_static;

use actix::{Addr, Actor, SyncArbiter, Arbiter};
use actix_cors::Cors;
use actix_web::{
    http::{header, Method},
    middleware,
    middleware::{Logger},
    web, web::Data, web::Json, web::Query,
    App, Error, HttpRequest, HttpResponse, HttpServer
};
use futures::future::Future;
use std::cmp;

///////////////////////
//// CRATE MODULES
///////////////////////

// Root crate import. Exposes modules in ./src/lib.rs
extern crate gm;
use gm::db::create_postgres_pool;
use gm::utils::{
    init_logging,
};

/// Macros first
#[macro_use] mod macros;
mod db;
mod endpoints;
mod models;
mod payment_clients;
mod pricing;
mod rest;
mod rpc;
mod webhooks;
mod bug_reporting;

use db::{
    DatabaseActor,
};
use payment_clients::{
    PaypalClient,
    PaypalAccessToken,
    StripeClient,
};
use webhooks::{
    handle_stripe_refund_webhook,
    handle_paypal_refund_webhook,
};

//// Constants
const PORT: i32 = 8898;
const IP: &str = "0.0.0.0";
const DEFAULT_MAX_DB_CONNECTIONS: u32 = 6;
const NUM_ACTOR_THREADS: usize = 3;

/////////////////////////////////////////////////
//// Entry point for the Payment Service
/////////////////////////////////////////////////

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    init_logging("payment", "debug");
    // Check env variables for other RPC endpoints
    endpoints::check_env_var_endpoints_exist();
    // Set up bug reporting
    bug_reporting::setup_bug_reporting();

    let paypal_actor = PaypalClient::new().start();

    let max_db_connections = match std::env::var("MAX_DB_CONNECTIONS") {
        Ok(s) => match s.parse::<u32>() {
            Ok(v) => v,
            Err(_e) => DEFAULT_MAX_DB_CONNECTIONS
        },
        Err(_e) => DEFAULT_MAX_DB_CONNECTIONS
    };
    let max_connections_per_pool = cmp::max(1, max_db_connections / NUM_ACTOR_THREADS as u32);
    let database_actor: Addr<DatabaseActor> = SyncArbiter::start(
        NUM_ACTOR_THREADS,
        move || DatabaseActor::new(
            create_postgres_pool("DATABASE_URL", max_connections_per_pool),
        )
    );

    // Start the http server
    HttpServer::new(move || {
        // Start the actors, set AppState
        App::new()
        .app_data(AppState {
            database_actor: database_actor.clone(),
            paypal_actor: paypal_actor.clone(),
            http_client: AppState::create_client(),
            stripe_actor: StripeClient::new(
                get_stripe_api_key(),
                None,
                None,
            ).start(),
        })
        // Add middlewares
        .wrap(Logger::default())
        .wrap(Cors::new().supports_credentials().finish())

        ///////////// Stripe ///////////////
        .service(web::scope("/stripe")
            .service(web::scope("/paymentMethod")
                .service(web::resource("/create")
                    .route(web::post().to(rest::stripe::create_payment_method)))
                .service(web::resource("/retrieve")
                    .route(web::get().to(rest::stripe::retrieve_payment_method)))
                .service(web::resource("/update")
                    .route(web::post().to(rest::stripe::update_payment_method)))
                .service(web::resource("/attach")
                    .route(web::post().to(rest::stripe::attach_payment_method)))
                .service(web::resource("/detach")
                    .route(web::post().to(rest::stripe::detach_payment_method)))
                // .service(web::resource("/list")
                //     .route(web::post().to(rest::stripe::list_payment_methods)))
            )
            .service(web::scope("/paymentIntent")
                .service(web::resource("/create")
                    .route(web::post().to(rest::stripe::create_payment_intent)))
                .service(web::resource("/retrieve")
                    .route(web::get().to(rest::stripe::retrieve_payment_intent)))
                .service(web::resource("/update")
                    .route(web::post().to(rest::stripe::update_payment_intent)))
                .service(web::resource("/confirm")
                    .route(web::post().to(rest::stripe::confirm_payment_intent)))
                .service(web::resource("/capture")
                    .route(web::post().to(rest::stripe::capture_payment_intent)))
                .service(web::resource("/cancel")
                    .route(web::post().to(rest::stripe::cancel_payment_intent)))
                // .service(web::resource("/list")
                //     .route(web::get().to(rest::stripe::list_payment_intents)))
            )
            .service(web::scope("/setupIntent")
                .service(web::resource("/create")
                    .route(web::post().to(rest::stripe::create_setup_intent)))
                .service(web::resource("/retrieve")
                    .route(web::get().to(rest::stripe::retrieve_setup_intent)))
                .service(web::resource("/update")
                    .route(web::post().to(rest::stripe::update_setup_intent)))
                .service(web::resource("/confirm")
                    .route(web::post().to(rest::stripe::confirm_setup_intent)))
                .service(web::resource("/cancel")
                    .route(web::post().to(rest::stripe::cancel_setup_intent)))
            )
            .service(web::scope("/customer")
                .service(web::resource("/create")
                    .route(web::post().to(rest::stripe::create_customer)))
                .service(web::resource("/retrieve")
                    .route(web::get().to(rest::stripe::retrieve_customer)))
                .service(web::resource("/update")
                    .route(web::post().to(rest::stripe::update_customer)))
                .service(web::resource("/delete")
                    .route(web::delete().to(rest::stripe::delete_customer)))
                // .service(web::resource("/list")
                //     .route(web::get().to(rest::stripe::list_all_customers)))
            )
        )
        .service(web::scope("/paypal")
            .service(web::resource("/create/order")
                .route(web::post().to(rest::paypal::paypal_create_order)))
            .service(web::resource("/confirm/order")
                .route(web::post().to(rest::paypal::paypal_confirm_order)))
        )
        .service(web::scope("/refund")
            .service(web::resource("")
                .route(web::post().to(rest::refund_endpoint)))
            .service(web::resource("/read/many")
                .route(web::post().to(rest::read_refunds_by_ids)))
        )
        .service(web::scope("/tx")
            .service(web::resource("/read/many")
                .route(web::post().to(rest::read_many_transactions_by_ids)))
            .service(web::resource("/read/recent")
                .route(web::post().to(rest::read_recent_transactions)))
            .service(web::resource("/read/connection")
                .route(web::post().to(rest::read_transaction_connection)))
            .service(web::resource("/create/payment")
                .route(web::post().to(rest::create_payment)))
            .service(web::resource("/confirm/payment")
                .route(web::post().to(rest::confirm_payment)))
            .service(web::resource("/write/record-frontend-tx")
                .route(web::post().to(rest::record_frontend_tx)))
        )
        .service(web::scope("/payoutItems")
            .service(web::resource("/read/many")
                .route(web::post().to(rest::read_payout_items_by_ids)))
            .service(web::resource("/read/many/by/orderItemIds")
                .route(web::post().to(rest::read_payout_items_by_order_item_ids)))
            .service(web::resource("/read/connection")
                .route(web::post().to(rest::read_payout_items_cursor_connection)))
            .service(web::resource("/read/connection/pages")
                .route(web::post().to(rest::read_payout_items_page_connection)))
            .service(web::resource("/read/history/summaries")
                .route(web::post().to(rest::read_store_payout_history_summaries)))
            .service(web::resource("/read/affiliate/commissions")
                .route(web::post().to(rest::read_affiliate_commissions)))
        )
        .service(web::scope("/payouts")
            .service(web::resource("/create")
                .route(web::post().to(rest::create_payout)))
            .service(web::resource("/approve")
                .route(web::post().to(rest::approve_payout)))
            .service(web::resource("/read/connection")
                .route(web::post().to(rest::read_payouts_connection)))
            .service(web::resource("/read/many")
                .route(web::post().to(rest::read_payouts_by_ids)))
            .service(web::resource("/read/store")
                .route(web::post().to(rest::read_payouts_by_store_id_connection)))
            .service(web::resource("/read/store/in/period")
                .route(web::post().to(rest::read_payouts_by_store_id_in_period)))
        )
        .service(web::scope("/paymentMethods")
            .service(web::resource("/read/many")
                .route(web::post().to(rest::read_many_payment_methods)))
            .service(web::resource("/read/many/user")
                .route(web::post().to(rest::read_payment_methods_by_user_id)))
            .service(web::resource("/detach/delete")
                .route(web::post().to(rest::delete_and_detach_payment_method)))
        )
        .service(web::scope("/payoutMethod")
            .service(web::resource("/read")
                .route(web::get().to(rest::read_payout_method)))
            .service(web::resource("/write")
                .route(web::post().to(rest::set_payout_method)))
        )
        .service(web::scope("/payoutSplit")
            .service(web::resource("/read")
                .route(web::get().to(rest::read_payout_split)))
            .service(web::resource("/read/many")
                .route(web::post().to(rest::read_many_payout_splits)))
            .service(web::resource("/read/many/current")
                .route(web::post().to(rest::read_current_payout_splits_by_ids)))
            .service(web::resource("/read/many/of/user")
                .route(web::post().to(rest::read_payout_splits_of_user)))
            .service(web::resource("/write")
                .route(web::post().to(rest::write_payout_split)))
            .service(web::resource("/update")
                .route(web::post().to(rest::update_payout_split)))
            .service(web::resource("/write/seller/and/sellerAffiliate")
                .route(web::post().to(rest::write_seller_seller_aff_payout_split)))
            .service(web::resource("/delete/all/for/user")
                .route(web::delete().to(rest::delete_all_payout_splits_for_user)))
            .service(web::resource("/delete")
                .route(web::delete().to(rest::delete_payout_split)))
        )
        .service(web::scope("/webhooks")
            .service(web::scope("/refund")
                .service(web::resource("/stripe")
                    .route(web::post().to(handle_stripe_refund_webhook)))
                .service(web::resource("/paypal")
                    .route(web::post().to(handle_paypal_refund_webhook)))
            )
        )
        .service(web::scope("/test")
            .service(web::resource("")
                .route(web::get().to(crate::rest::test_handler)))
            .service(web::resource("/rpc")
                .route(web::get().to(rpc::rpc_test_handler)))
            .service(web::resource("/rpc/id")
                .route(web::get().to(rpc::rpc_test_id)))
            .service(web::resource("/rpc/cart/subtotal")
                .route(web::get().to(rpc::rpc_test_cart_handler)))
            .service(web::resource("/rpc/orders/confirm")
                .route(web::get().to(rpc::rpc_test_create_order)))

            .service(web::scope("/paypal")
                .service(web::resource("/refresh")
                    .route(web::get().to(rest::paypal::test_paypal_token_refresh)))
                .service(web::resource("/refund")
                    .route(web::get().to(rest::paypal::test_paypal_refund)))
                .service(web::resource("/invoice")
                    .route(web::get().to(rest::paypal::test_paypal_read_invoice)))
            )
        )
        .service(web::resource("/_health")
            .route(web::get().to(rest::health::retrieve_health_status)))
        // 404 routes
        .default_service(web::route().to(crate::rest::handle_404))
    })
    // .bind_ssl(format!("{}:{}", IP, PORT), ssl_builder)
    .bind(format!("{}:{}", IP, PORT))
    .expect(&format!("Cannot bind to {}:{}, address in use or invalid", IP, PORT))
    .run()
    .await
}

//// AppState
pub struct AppState {
    pub database_actor: Addr<DatabaseActor>,
    pub stripe_actor: Addr<StripeClient>,
    pub paypal_actor: Addr<PaypalClient>,
    pub http_client: actix_web::client::Client,
}

impl AppState {

    pub fn from(req: &HttpRequest) -> &AppState {
        &req.app_data::<AppState>().expect("AppState error")
    }

    pub fn databaseActor(req: &HttpRequest) -> &Addr<DatabaseActor> {
        &req.app_data::<AppState>().expect("AppState error")
            .database_actor
    }

    pub fn paypalActor(req: &HttpRequest) -> &Addr<PaypalClient> {
        &req.app_data::<AppState>().expect("AppState error")
            .paypal_actor
    }

    pub fn stripeActor(req: &HttpRequest) -> &Addr<StripeClient> {
        &req.app_data::<AppState>().expect("AppState error")
            .stripe_actor
    }

    pub fn httpClient(req: &HttpRequest) -> &actix_web::client::Client {
        &req.app_data::<AppState>().expect("AppState error")
            .http_client
    }

    pub fn create_client() -> actix_web::client::Client {
        actix_web::client::ClientBuilder::new()
            .header("Content-Type", "application/json")
            .header("Accept-Encoding", "*")
            .finish()
    }
}

pub fn get_stripe_api_key() -> String {
    dotenv::dotenv().ok();
    std::env::var("STRIPE_API_KEY")
        .expect("STRIPE_API_KEY not set in .env!")
}

