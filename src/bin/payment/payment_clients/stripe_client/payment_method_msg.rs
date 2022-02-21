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
    PaymentMethodCreateParams,
    PaymentMethodUpdateParams,
    PaymentMethodsListParams,
    PaymentMethodAttachParams,
    PaymentMethod,
};

type PaymentMethodId = String;
/// https://stripe.com/docs/api/payment_methods/retrieve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethodMsg {
    /// Creates a new PaymentMethod.
    /// For more details see https://stripe.com/docs/api/payment_methods/create
    Create(PaymentMethodCreateParams),
    /// Retrieves a PaymentMethod by its Id which starts with "pm_".
    /// For more details see https://stripe.com/docs/api/payment_methods/retrieve
    Retrieve(PaymentMethodId),
    /// Updates a PaymentMethod
    /// For more details see https://stripe.com/docs/api/payment_methods/update
    Update(PaymentMethodId, PaymentMethodUpdateParams),
    /// Attaches a new PaymentMethod to a Customer
    /// For more details see https://stripe.com/docs/api/payment_methods/retrieve
    Attach(PaymentMethodId, PaymentMethodAttachParams),
    /// Detaches a PaymentMethod from a Customer
    /// For more details see https://stripe.com/docs/api/payment_methods/detach
    Detach(PaymentMethodId),
}


impl Message for PaymentMethodMsg {
    type Result = StripeResponse<PaymentMethod>;
}

impl Handler<PaymentMethodMsg> for StripeClient {

    type Result = ResponseActFuture<Self, StripeResponse<PaymentMethod>>;

    fn handle(
        &mut self,
        msg: PaymentMethodMsg,
        _ctx: &mut Context<Self>
    ) -> Self::Result {

        let http_client = Arc::clone(&self.client);

        Box::pin(async move {
            match msg {
                PaymentMethodMsg::Create(body) => {
                    awc_post_body(http_client,
                        "/payment_methods",
                        body
                    ).await
                },
                PaymentMethodMsg::Retrieve(payment_method_id) => {
                    awc_get(http_client,
                        &format!("/payment_methods/{}", payment_method_id)
                    ).await
                },
                PaymentMethodMsg::Update(payment_method_id, body) => {
                    awc_post_body(http_client,
                        &format!("/payment_methods/{}", payment_method_id),
                        body
                    ).await
                }
                PaymentMethodMsg::Attach(payment_method_id, body) => {
                    awc_post_body(http_client,
                        &format!("/payment_methods/{}/attach", payment_method_id),
                        body
                    ).await
                },
                PaymentMethodMsg::Detach(payment_method_id) => {
                    awc_post(http_client,
                        &format!("/payment_methods/{}/detach", payment_method_id)
                    ).await
                },
            }
        }.into_actor(self))
    }
}
