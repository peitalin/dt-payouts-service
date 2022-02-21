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
    PaymentIntentCreateParams,
    PaymentIntentUpdateParams,
    PaymentIntentConfirmParams,
    PaymentIntentCancelParams,
    PaymentIntentCaptureParams,
    PaymentIntentListParams,
    PaymentIntent,
};

type PaymentIntentId = String;
/// https://stripe.com/docs/api/payment_intents/retrieve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentIntentMsg {
    /// Creates a new payment_intent.
    /// For more details see https://stripe.com/docs/api/payment_intents/create.
    Create(PaymentIntentCreateParams),
    /// Retrieves the details of a payment_intent.
    /// For more details see https://stripe.com/docs/api/payment_intents/retrieve.
    Retrieve(PaymentIntentId),
    /// Updates a payment_intent's properties.
    /// For more details see https://stripe.com/docs/api/payment_intents/update.
    Update(PaymentIntentId, PaymentIntentUpdateParams),
    /// Confirm that customer intends to pay with current or provided source. Upon confirmation, the PaymentIntent will attempt to initiate a payment.
    /// For more details see https://stripe.com/docs/api/payment_intents/confirm.
    Confirm(PaymentIntentId, PaymentIntentConfirmParams),
    /// Capture the funds of an existing uncaptured PaymentIntent where required_action="requires_capture".
    /// For more details see https://stripe.com/docs/api/payment_intents/capture.
    Capture(PaymentIntentId, PaymentIntentCaptureParams),
    /// A PaymentIntent object can be canceled when it is in one of these statuses: requires_source, requires_capture, requires_confirmation, requires_source_action.
    /// For more details see https://stripe.com/docs/api/payment_intents/cancel.
    Cancel(PaymentIntentId, PaymentIntentCancelParams),
}


impl Message for PaymentIntentMsg {
    type Result = StripeResponse<PaymentIntent>;
}

impl Handler<PaymentIntentMsg> for StripeClient {

    type Result = ResponseActFuture<Self, StripeResponse<PaymentIntent>>;

    fn handle(
        &mut self,
        msg: PaymentIntentMsg,
        _ctx: &mut Context<Self>
    ) -> Self::Result {

        let http_client = Arc::clone(&self.client);

        Box::pin(async move {
            match msg {
                PaymentIntentMsg::Create(body) => {
                    awc_post_body(http_client,
                        "/payment_intents",
                        body
                    ).await
                },
                PaymentIntentMsg::Retrieve(payment_intent_id) => {
                    awc_get(http_client,
                        &format!("/payment_intents/{}", payment_intent_id)
                    ).await
                },
                PaymentIntentMsg::Update(payment_intent_id, body) => {
                    awc_post_body(http_client,
                        &format!("/payment_intents/{}", payment_intent_id),
                        body
                    ).await
                }
                PaymentIntentMsg::Confirm(payment_intent_id, body) => {
                    awc_post_body(http_client,
                        &format!("/payment_intents/{}/confirm", payment_intent_id),
                        body
                    ).await
                },
                PaymentIntentMsg::Capture(payment_intent_id, body) => {
                    awc_post_body(http_client,
                        &format!("/payment_intents/{}/capture", payment_intent_id),
                        body
                    ).await
                },
                PaymentIntentMsg::Cancel(payment_intent_id, body) => {
                    awc_post_body(http_client,
                        &format!("/payment_intents/{}/cancel", payment_intent_id),
                        body
                    ).await
                },
            }
        }.into_actor(self))
    }
}
