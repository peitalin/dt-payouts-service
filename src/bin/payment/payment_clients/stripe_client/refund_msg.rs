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
    RefundCreateParams,
    RefundUpdateParams,
    RefundListParams,
    Refund,
};

type RefundId = String;

/// https://stripe.com/docs/api/refunds/retrieve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefundMsg {
    /// Creates a new refund.
    /// For more details see https://stripe.com/docs/api/refunds/create.
    Create(RefundCreateParams),
    /// Retrieves the details of a refund.
    /// You need only supply the unique refund identifier that was returned upon refund creation.
    /// For more details see https://stripe.com/docs/api/refunds/retrieve.
    Retrieve(RefundId),
    /// Updates the specified refund by setting the values of the parameters passed.
    /// Any parameters not provided will be left unchanged.  This request only accepts `metadata` as an argument.
    /// For more details see https://stripe.com/docs/api/refunds/update.
    Update(RefundId, RefundUpdateParams),
}


impl Message for RefundMsg {
    type Result = StripeResponse<Refund>;
}

impl Handler<RefundMsg> for StripeClient {

    type Result = ResponseActFuture<Self, StripeResponse<Refund>>;

    fn handle(
        &mut self,
        msg: RefundMsg,
        _ctx: &mut Context<Self>
    ) -> Self::Result {

        let http_client = Arc::clone(&self.client);

        Box::pin(async move {
            match msg {
                RefundMsg::Create(body) => {
                    awc_post_body(http_client,
                        "/refunds",
                        body
                    ).await
                },
                RefundMsg::Retrieve(refund_id) => {
                    awc_get(http_client,
                        &format!("/refunds/{}", refund_id),
                    ).await
                },
                RefundMsg::Update(refund_id, body) => {
                    awc_post_body(http_client,
                        &format!("/refunds/{}", refund_id),
                        body
                    ).await
                }
            }
        }.into_actor(self))
    }
}
