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
    List,
    PaymentMethod,
    PaymentMethodsListParams,
    PaymentIntent,
    PaymentIntentListParams,
    Refund,
    RefundListParams,
    Customer,
    CustomerListParams,
};

/// https://stripe.com/docs/api/payment_methods/retrieve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListMsg where
{
    /// Capture the funds of an existing uncaptured Customer where required_action="requires_capture".
    /// For more details see https://stripe.com/docs/api/customers/list.
    Customers(CustomerListParams),
    /// List all payment_intents.
    /// For more details see https://stripe.com/docs/api/payment_intents/list.
    PaymentIntents(PaymentIntentListParams),
    /// Lists all the PaymentMethods of a Customer
    /// For more details see https://stripe.com/docs/api/payment_methods/list
    PaymentMethods(PaymentMethodsListParams),
    /// Capture the funds of an existing uncaptured Refund where required_action="requires_capture".
    /// For more details see https://stripe.com/docs/api/refunds/list.
    Refunds(RefundListParams),
}


impl Message for ListMsg {
    type Result = StripeResponse<serde_json::Value>;
}

impl Handler<ListMsg> for StripeClient {

    type Result = ResponseActFuture<Self, StripeResponse<serde_json::Value>>;

    fn handle(
        &mut self,
        msg: ListMsg,
        _ctx: &mut Context<Self>
    ) -> Self::Result {

        let http_client = Arc::clone(&self.client);

        Box::pin(async move {
            match msg {
                ListMsg::Customers(query) => {
                    awc_get_query(http_client,
                        "/customers",
                        serde_json::to_value(query)?
                    ).await
                },
                ListMsg::PaymentIntents(query) => {
                    awc_get_query(http_client,
                        "/payment_intents",
                        serde_json::to_value(query)?
                    ).await
                },
                ListMsg::PaymentMethods(query) => {
                    awc_get_query(http_client,
                        "/payment_methods",
                        serde_json::to_value(query)?
                    ).await
                },
                ListMsg::Refunds(query) => {
                    awc_get_query(http_client,
                        "/refunds",
                        serde_json::to_value(query)?
                    ).await
                },
            }
        }.into_actor(self))
    }
}
