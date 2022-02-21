use super::awc_handlers::{
    awc_get,
    awc_get_query,
    awc_post,
    awc_post_body,
    awc_delete,
};
use super::actor::{
    STRIPE_ENDPOINT_URL,
    StripeClient,
    StripeResponse,
};

///////// Actor Implementation /////////
use actix::{Addr, Handler, Context, Message};
use actix::prelude::{ ResponseActFuture, WrapFuture };
use actix_web::{ Error };
use std::sync::Arc;

use std::boxed::Box;
use std::pin::Pin;
use serde_derive::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::models::{ StripeError, ErrJson };


use gm::models::stripe::{
    CustomerCreateParams,
    CustomerUpdateParams,
    CustomerListParams,
    Customer,
};

type CustomerId = String;

/// https://stripe.com/docs/api/customers/retrieve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomerMsg {
    /// Creates a new customer.
    /// For more details see https://stripe.com/docs/api/customers/create.
    Create(CustomerCreateParams),
    /// Retrieves the details of a customer.
    /// You need only supply the unique customer identifier that was returned upon customer creation.
    /// For more details see https://stripe.com/docs/api/customers/retrieve.
    Retrieve(CustomerId),
    /// Updates a customer's properties.
    /// For more details see https://stripe.com/docs/api/customers/update.
    Update(CustomerId, CustomerUpdateParams),
    /// Permanently deletes a customer. It cannot be undone.
    /// Also immediately cancels any active subscriptions on the customer.
    /// https://stripe.com/docs/api/customers/delete
    Delete(CustomerId),
}


impl Message for CustomerMsg {
    type Result = StripeResponse<Customer>;
}

impl Handler<CustomerMsg> for StripeClient {

    type Result = ResponseActFuture<Self, StripeResponse<Customer>>;

    fn handle(
        &mut self,
        msg: CustomerMsg,
        _ctx: &mut Context<Self>
    ) -> Self::Result {

        let http_client = Arc::clone(&self.client);

        Box::pin(async move {
            match msg {
                CustomerMsg::Create(body) => {
                    awc_post_body(http_client,
                        "/customers",
                        body
                    ).await
                },
                CustomerMsg::Retrieve(customer_id) => {
                    awc_get(http_client,
                        &format!("/customers/{}", customer_id),
                    ).await
                },
                CustomerMsg::Update(customer_id, body) => {
                    awc_post_body(http_client,
                        &format!("/customers/{}", customer_id),
                        body
                    ).await
                }
                CustomerMsg::Delete(customer_id) => {
                    awc_delete(http_client,
                        &format!("/customers/{}", customer_id),
                    ).await
                },
            }
        }.into_actor(self))
    }
}
