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
    SetupIntentCreateParams,
    SetupIntentUpdateParams,
    SetupIntentConfirmParams,
    SetupIntentCancelParams,
    SetupIntent,
};

type SetupIntentId = String;
/// https://stripe.com/docs/api/setup_intents/retrieve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupIntentMsg {
    /// Creates a new setup_intent.
    /// For more details see https://stripe.com/docs/api/setup_intents/create
    Create(SetupIntentCreateParams),
    /// Retrieves the details of a setup_intent.
    /// For more details see https://stripe.com/docs/api/setup_intents/retrieve
    Retrieve(SetupIntentId),
    /// Updates a setup_intent's properties.
    /// For more details see https://stripe.com/docs/api/setup_intents/update
    Update(SetupIntentId, SetupIntentUpdateParams),
    /// Confirm that customer intends to pay with current or provided source.
    /// Upon confirmation, the SetupIntent will attempt to initiate a payment.
    /// For more details see https://stripe.com/docs/api/setup_intents/confirm
    Confirm(SetupIntentId, SetupIntentConfirmParams),
    /// A SetupIntent object can be canceled when it is in one of these statuses:
    /// For more details see https://stripe.com/docs/api/setup_intents/cancel
    Cancel(SetupIntentId, SetupIntentCancelParams),
}


impl Message for SetupIntentMsg {
    type Result = StripeResponse<SetupIntent>;
}

impl Handler<SetupIntentMsg> for StripeClient {

    type Result = ResponseActFuture<Self, StripeResponse<SetupIntent>>;

    fn handle(
        &mut self,
        msg: SetupIntentMsg,
        _ctx: &mut Context<Self>
    ) -> Self::Result {

        let http_client = Arc::clone(&self.client);

        Box::pin(async move {
            match msg {
                SetupIntentMsg::Create(body) => {
                    awc_post_body(http_client,
                        "/setup_intents",
                        body
                    ).await
                },
                SetupIntentMsg::Retrieve(setup_intent_id) => {
                    awc_get(
                        http_client,
                        &format!("/setup_intents/{}", setup_intent_id)
                    ).await
                },
                SetupIntentMsg::Update(setup_intent_id, body) => {
                    awc_post_body(http_client,
                        &format!("/setup_intents/{}", setup_intent_id),
                        body
                    ).await
                }
                SetupIntentMsg::Confirm(setup_intent_id, body) => {
                    awc_post_body(http_client,
                        &format!("/setup_intents/{}/confirm", setup_intent_id),
                        body
                    ).await
                },
                SetupIntentMsg::Cancel(setup_intent_id, body) => {
                    awc_post_body(http_client,
                        &format!("/setup_intents/{}/cancel", setup_intent_id),
                        body
                    ).await
                },
            }
        }.into_actor(self))
    }
}
