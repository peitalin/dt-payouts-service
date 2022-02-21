use actix::{Actor};
use actix_web::{
    HttpRequest,
    HttpResponse,
    web::Json,
    web::Query,
    Error,
};
use futures::{
    future,
    future::Either,
    Future,
};
use std::str::FromStr;
use serde::Serialize;

use crate::AppState;
use crate::models::{
    PaypalResponse,
    PaypalError,
    Transaction,
    TransactionId,
    DbError, ErrJson,
    PayoutItem,
    Currency,
};
use crate::models::paypal::{
    PaypalPayoutParams,
    PaypalPayout,
    PaypalPayoutResponse, // response after creating a payout
    PaypalPaidoutBatchResponse, // response to GET payout
    PaypalErrorResponse, // response if Paypal payout request errors
    PaypalPaidoutItemDetails,
};
// message actions
use crate::payment_clients::{
    PaypalRequest,
};

use crate::db;
use crate::db::{ GetPool };




pub async fn create_batch_payout(
    req: HttpRequest,
    paypal_payout_params: PaypalPayoutParams,
) -> Result<PaypalPayoutResponse, Error> {

    let payout_response = AppState::paypalActor(&req)
        .send(
            PaypalRequest::PostBody::<PaypalPayoutParams>(
                String::from("/v1/payments/payouts"),
                paypal_payout_params,
            )
        )
        .await??;

    debug!("{:?}", &payout_response);

    let payout_response2 = serde_json::from_str::<PaypalPayoutResponse>(
        &payout_response
    ).map_err(|e| Error::from(PaypalError::DeserializationError(errJson!(e))));

    match payout_response2 {
        // If Paypal Payout successful, return success reponse
        Ok(res) => Ok(res),
        // if Payout Errors, deserialize error message and return error
        Err(_e) => Err(handle_payout_error(payout_response)),
    }
}


pub fn handle_payout_error(payout_response1: String) -> Error {
    // if Payout Errors, deserialize error message and return error
    let payout_error = serde_json::from_str::<PaypalErrorResponse>(
        &payout_response1
    ).map_err(|e| Error::from(PaypalError::DeserializationError(errJson!(e))));

    match payout_error {
        // error deserialization error, unexpected error response
        Err(e) => e,
        Ok(err) => {

            match err.name {
                Some(n) => {
                    if n == String::from("INSUFFICIENT_FUNDS") {
                        return Error::from(PaypalError::InsufficientFunds(errJson!(
                            "INSUFFICIENT FUNDS"
                        )))
                    }
                    if n == String::from("VALIDATION_ERROR") {
                        return Error::from(PaypalError::ValidationError(errJson!(
                            "VALIDATION_ERROR"
                        )))
                    } else {
                        return Error::from(PaypalError::InternalError(errJson!(
                            "UNHANDLED_ERROR"
                        )))
                    }
                },
                None => {
                    return Error::from(PaypalError::InternalError(errJson!(
                        "UNHANDLED_ERROR"
                    )))
                }
            }
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalGetPayoutQuery {
    payout_batch_id: String,
}
pub async fn get_paypal_paidout_payout(
    req: HttpRequest,
    query: Query<PaypalGetPayoutQuery>
) -> Result<PaypalPaidoutBatchResponse, Error> {

    let url = PaypalRequest::Get::<serde_json::Value>(
        format!("/v1/payments/payouts/{}", query.payout_batch_id)
    );

    let payout = AppState::paypalActor(&req)
                    .send(url)
                    .await??;

    debug!("{:?}", &payout);

    serde_json::from_str::<PaypalPaidoutBatchResponse>(
        &payout
    ).map_err(|e| Error::from(PaypalError::DeserializationError(errJson!(e))))

}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalGetPayoutsItemQuery {
    payouts_item_id: String,
}
pub async fn get_paypal_paidout_item_details(
    req: HttpRequest,
    query: Query<PaypalGetPayoutsItemQuery>
) -> Result<PaypalPaidoutItemDetails, Error> {

    let url = PaypalRequest::Get::<serde_json::Value>(
        format!("/v1/payments/payouts-item/{}", query.payouts_item_id)
    );

    let payout = AppState::paypalActor(&req)
                    .send(url)
                    .await??;

    debug!("{:?}", &payout);

    serde_json::from_str::<PaypalPaidoutItemDetails>(
        &payout
    ).map_err(|e| Error::from(PaypalError::DeserializationError(errJson!(e))))

}
