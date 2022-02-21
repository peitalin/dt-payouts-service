use super::awc_handlers::{
    awc_get,
    awc_get_query,
    awc_post,
    awc_post_body,
    awc_delete,
};

///////// Actor Implementation /////////
use actix::{Actor, Addr, Handler, Context, Message};
use actix_web::{
    client::{Client, ClientRequest, ClientBuilder},
    Error,
};
use actix::prelude::{ ResponseActFuture, WrapFuture };
use std::sync::Arc;

use std::boxed::Box;
use std::pin::Pin;
use serde_derive::{Deserialize, Serialize};
use crate::models::{ StripeError, ErrJson };

pub const STRIPE_ENDPOINT_URL: &str = "https://api.stripe.com/v1";


#[derive(Clone)]
pub struct StripeClient {
    pub client: Arc<actix_web::client::Client>,
    pub secret_key: String,
    pub url: String,
    pub stripe_account: Option<String>,
    pub client_id: Option<String>,
}

impl StripeClient {

    pub fn new(
        secret_key: String,
        stripe_account: Option<String>,
        client_id: Option<String>,
    ) -> Self {

        let actix_client = actix_web::client::ClientBuilder::new()
            // .header("Content-Type", "application/json")
            // .header("Accept", "application/json")
            .header("Accept-Encoding", "*")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", format!("Bearer {}", secret_key));

        let client = match (stripe_account.clone(), client_id.clone()) {
            (None, None)           => actix_client,
            (None, Some(cid))      => actix_client.header("client-id", cid),
            (Some(acc), None)      => actix_client.header("stripe-account", acc),
            (Some(acc), Some(cid)) => {
                actix_client
                    .header("stripe-account", acc)
                    .header("client-id", cid)
            },
        };

        StripeClient {
            client: Arc::new(client.finish()),
            secret_key: secret_key,
            url: String::from(STRIPE_ENDPOINT_URL),
            stripe_account: stripe_account,
            client_id: client_id,
        }
    }

}

impl Actor for StripeClient {
    type Context = Context<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, _ctx: &mut Self::Context) {
        // ctx.notify(StripeRefresh::RefreshToken);
    }
}

pub type StripeResponse<T> = Result<T, StripeError>;
// pub type StripeResponse<T> = Result<T, actix_web::Error>;


// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum StripeRequest<B: serde::Serialize> {
//     Get(String),
//     GetQuery(String, B),
//     Post(String),
//     PostBody(String, B),
//     Delete(String),
// }

// impl<T, B> Message for StripeRequest<B>
// where
//     T: DeserializeOwned + Send + 'static,
//     B: serde::Serialize + 'static
// {
//     type Result = StripeResponse<T>;
// }

// impl<T, B> Handler<StripeRequest<B>> for StripeClient
// where
//     T: DeserializeOwned + Send + 'static,
//     B: serde::Serialize + 'static
// {

//     type Result = ResponseActFuture<Self, StripeResponse<T>>;

//     fn handle(
//         &mut self,
//         msg: StripeRequest<B>,
//         _ctx: &mut Context<Self>
//     ) -> Self::Result {

//         let client = Arc::clone(&self.client);
//         let secret_key = self.secret_key.clone();

//         Box::pin(async move {
//             match msg {
//                 StripeRequest::Get(route) => {
//                     awc_get::<T>(client,
//                         &route
//                     ).await
//                 },
//                 StripeRequest::GetQuery(route, body) => {
//                     awc_get_query::<T, B>(client,
//                         &route,
//                         body
//                     ).await
//                 }
//                 StripeRequest::Post(route) => {
//                     awc_post::<T>(client,
//                         &route
//                     ).await
//                 },
//                 StripeRequest::PostBody(route, body) => {
//                     awc_post_body::<T, B>(client,
//                         &route,
//                         body
//                     ).await
//                 }
//                 StripeRequest::Delete(route) => {
//                     awc_delete::<T>(client,
//                         &route
//                     ).await
//                 },
//             }

//         }.into_actor(self))
//     }
// }

