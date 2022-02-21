
use actix::{Actor, Arbiter, Addr, Handler, Context, AsyncContext, Message};
use actix_web::{
    client::{Client, ClientRequest, ClientBuilder},
    Error,
};
use actix::prelude::{ ResponseActFuture, WrapFuture };
use std::sync::Arc;

use serde::Serialize;
use futures::future::{lazy, Future};
use std::time::{Duration, Instant};
use std::{rc::Rc, cell::RefCell, cell::Ref};
use rand::Rng;


use crate::models::ErrJson;
use crate::models::PaypalError;

type ActixWebClient = Arc<actix_web::client::Client>;

#[derive(Clone)]
pub struct PaypalClient {
    pub client: ActixWebClient,
    pub url: String,
    pub access_token_exp: Rc<RefCell<i32>>,
    pub access_token: Rc<RefCell<String>>,
    pub hb: Instant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaypalAccessToken {
    scope: String,
    access_token: String,
    token_type: String,
    app_id: String,
    expires_in: i32,
    nonce: String,
}

/// Payload to send to Paypal to request access token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantType {
    pub grant_type: String,
}

impl PaypalClient {

    /// Creates a new client pointed to `https://api.sandbox.paypal.com`
    pub fn new() -> Self {

        dotenv::dotenv().ok();
        let paypal_api_host = std::env::var("PAYPAL_API_HOST")
            .expect("PAYPAL_API_HOST not set in .env");

        PaypalClient {
            url: paypal_api_host,
            access_token_exp: Default::default(),
            access_token: Default::default(),
            hb: Instant::now(),
            client: Arc::new(
                actix_web::client::ClientBuilder::new()
                    .disable_timeout()
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", "starting..."))
                    .finish()
            )
        }
    }

    fn get_client_no_auth_header(&mut self) -> ActixWebClient {
        // PaypalRefresh does not need an Authorizatoin: Bearer header.
        let new_client: ActixWebClient = Arc::new(
            actix_web::client::ClientBuilder::new()
                .disable_timeout()
                .header("Content-Type", "application/json")
                .finish()
        );
        // inefficient, but safer
        self.client = new_client.clone();

        new_client
    }

    fn get_client(&mut self) -> ActixWebClient {
        // instantiates a new paypal client with the current
        // access token on each request.
        let access_token = Rc::clone(&self.access_token);
        let token = access_token.borrow();

        let new_client: ActixWebClient = Arc::new(
            actix_web::client::ClientBuilder::new()
                .disable_timeout()
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .finish()
        );
        // inefficient, but safer
        self.client = new_client.clone();

        new_client
    }

    /// helper method that sends ping to client every second.
    /// this method also checks heartbeats from client
    fn hb(&mut self, ctx: &mut <Self as Actor>::Context) {

        let mut rng = rand::thread_rng();
        let refresh_interval: u64 = rng.gen_range(300, 600); // 5min to 10min
        debug!("Paypal token refresh every {:?} seconds", &refresh_interval);
        // Get paypal access token
        ctx.notify(PaypalAuthRefresh::RefreshToken);

        // Then init heartbeat to keep requesting token
        ctx.run_interval(Duration::from_secs(refresh_interval), |_act, ctx| {
            // Send message to itself to refresh token every 5-10min
            ctx.notify(PaypalAuthRefresh::RefreshToken);
        });
    }

}

fn paypal_secret() -> (String, String) {
    dotenv::dotenv().ok();
    let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID")
        .expect("PAYPAL_CLIENT_ID not set in .env");
    let paypal_secret = std::env::var("PAYPAL_SECRET")
        .expect("PAYPAL_SECRET not set in .env");
    (paypal_client_id, paypal_secret)
}

impl Actor for PaypalClient {
    type Context = Context<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        // Then init heartbeat to keep requesting token
        self.hb(ctx);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaypalRequest<B: serde::Serialize + Send + 'static> {
    Get(String),
    PostBody(String, B),
}
// Note: Could make B a concrete type: serde_json::Value, which
// would be less generic, but can dispense with the Generic notations
// B: serde::Serialize + Send + 'static

impl<B: serde::Serialize + Send + 'static> Message for PaypalRequest<B> {
    type Result = Result<String, PaypalError>;
}


impl<B: serde::Serialize + Send + 'static> Handler<PaypalRequest<B>> for PaypalClient {

    type Result = ResponseActFuture<Self, Result<String, PaypalError>>;

    fn handle(
        &mut self,
        msg: PaypalRequest<B>,
        _ctx: &mut Context<Self>
    ) -> Self::Result {

        // let client = Arc::clone(&self.client);
        let client = self.get_client(); // WIth Authorization: Bearer header
        let url = self.url.clone();

        Box::pin(async move {
            match msg {
                PaypalRequest::Get(route) => {
                    let request_url = format!("{}{}", url, route);
                    debug!("GET endpoint: {}", &request_url);

                    let mut response = client
                        .get(request_url)
                        .send()
                        .await
                        .map_err(|e| PaypalError::NetworkError(errJson!(e)))?;

                    println!("Response: {:?}", response);

                    let bytes = response.body()
                        .await
                        .map_err(|e| PaypalError::DeserializationError(errJson!(e)))?;

                    std::str::from_utf8(&bytes)
                        .map(|s| String::from(s))
                        .map_err(|e| PaypalError::DeserializationError(errJson!(e)))

                },
                PaypalRequest::PostBody(route, body) => {
                    let request_url = format!("{}{}", url, route);
                    debug!("POST endpoint: {}", &request_url);

                    let mut response = client
                        .post(request_url)
                        .send_json(&body)
                        .await
                        .map_err(|e| PaypalError::NetworkError(errJson!(e)))?;

                    println!("Response: {:?}", response);

                    let bytes = response.body()
                        .await
                        .map_err(|e| PaypalError::DeserializationError(errJson!(e)))?;

                    std::str::from_utf8(&bytes)
                        .map(|s| String::from(s))
                        .map_err(|e| PaypalError::DeserializationError(errJson!(e)))
                }
            }
        }.into_actor(self))
    }
}




////////////// Heartbeat Auth Refresh Handler


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaypalAuthRefresh {
    RefreshToken
}

impl Message for PaypalAuthRefresh {
    type Result = Result<serde_json::Value, PaypalError>;
}

impl Handler<PaypalAuthRefresh> for PaypalClient {

    type Result = ResponseActFuture<Self, Result<serde_json::Value, PaypalError>>;

    fn handle(
        &mut self,
        _msg: PaypalAuthRefresh,
        _ctx: &mut Context<Self>
    ) -> Self::Result {

        dotenv::dotenv().ok();
        let (paypal_client_id, paypal_secret) = paypal_secret();
        // ref counted ref to self.access_token
        // this is sent into a closure which has a shorter life-time
        // if the closure borrows self.access_token, it will take it with when it ends
        // so self.access_token needs to be ref-counted to ensure its not destroyed
        let self_access_token = Rc::clone(&self.access_token);
        let url = format!("{}/v1/oauth2/token", &self.url);
        // instantiate new client to make REST call
        // let client = Arc::clone(&self.client);
        let client = self.get_client_no_auth_header(); // Without Authorization: Bearer header

        debug!("=> Refreshing Paypal Token...");
        debug!("url: {:?}", url);
        debug!("PAYPAL_CLIENT_ID: {:?}", paypal_client_id);

        // Box::pin(async move {}.into_actor(self)) is a special async wrapper
        // for actors to make async calls
        // Note: can't have self inside this closure. Must use Arc<T> or refcounts
        Box::pin(async move {

            // self.client may not have Authorization header to begin with.
            // This will set it at run-time.
            let req_client = client
                .post(url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Accept-Language", "en_US")
                .basic_auth(paypal_client_id, Some(&paypal_secret));

            debug!("Request client: {:?}", req_client);

            let mut response = req_client
                .send_form(&GrantType {
                    grant_type: String::from("client_credentials")
                })
                .await
                .map_err(|e| PaypalError::InternalError(errJson!(e)))?;

            let bytes = response.body()
                .await
                .map_err(|e| PaypalError::DeserializationError(errJson!(e)))?;
            println!("Paypal raw response: {:?}", bytes);

            let token: Option<PaypalAccessToken> =
                serde_json::from_slice(&bytes).ok();

            if let Some(t) = token {

                debug!("Setting Paypal token: `Bearer {:?}`", &t);
                /// Replace old self.access_token with new token from Paypal
                let old_token = self_access_token.replace(t.access_token.clone());

                // replace(&self, t: T) -> T
                // Replaces the wrapped value with a new one, returning the old value, without deinitializing either one.
                // Panics if the value is currently borrowed.

                /// use std::cell::RefCell;
                /// let cell = RefCell::new(5);
                /// let old_value = cell.replace(6);
                /// assert_eq!(old_value, 5);
                /// assert_eq!(cell, RefCell::new(6));

                println!("=========================================");
                println!("Replacing paypal token:");
                println!("Old: {:?}", old_token);
                println!("New: {:?}", &t.access_token);
                println!("=========================================");

                Ok(json!({
                    "status": "PaypalAuthRefresh complete.",
                    "token": &t.access_token
                }))
            } else {
                println!("No paypal token found...");

                Ok(json!({
                    "status": "PaypalAuthRefresh failed.",
                    "token": ""
                }))
            }

        }.into_actor(self))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalRefreshResponse {
    pub status: String,
    pub token: String,
}

