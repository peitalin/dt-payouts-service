use actix::{Actor, Addr};
use actix_web::{
    client::ClientResponse,
    HttpRequest,
    HttpResponse,
    HttpMessage,
    web::Json,
    web::Query,
    Error,
};
use futures::{
    future,
    future::Either,
    Future,
    Stream,
};

use std::str::FromStr;
use std::collections::HashMap;
use itertools::{Itertools, Either as EitherLR};

use gm::utils::dates::from_datetimestr_to_naivedatetime;
use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use crate::bug_reporting::get_gm_environment;

use crate::AppState;
use crate::models::{
    // Errors
    ErrJson,
    AuthError,
    RpcError,
    //
    PayoutItem,
    PayoutStatus,
    PayoutPeriod,
    Payout,
    UserPublic,
    PayeeId, // String
    PayoutEmail, // String
    PayoutMethod,
    AuthInfo,
    UserRole,
    PaypalPayoutParams,
    PaypalPayout,
    PaypalPayoutResponse,
    PayoutAggregates,
    PayeeType,
};
use crate::models::payout_signatures::{
    SignedPayouts,
    PayoutApprovalType,
    PayoutIds,
    PaidPayouts,
};
use crate::models::payouts;
use crate::models::connection::{
    ConnectionQuery,
    Edge,
    Connection,
    PageInfo,
};
use crate::rest::paypal;
use crate::db;
use crate::db::{ GetPool };
use crate::rpc;


#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AffiliateType {
    SELLER_AFFILIATE,
    BUYER_AFFILIATE
}
impl From<AffiliateType> for PayeeType {
    fn from(a: AffiliateType) -> Self {
        match a {
            AffiliateType::BUYER_AFFILIATE => PayeeType::BUYER_AFFILIATE,
            AffiliateType::SELLER_AFFILIATE => PayeeType::SELLER_AFFILIATE,
        }
    }
}
fn from_affiliate_types_to_payee_types(
    o: Option<Vec<AffiliateType>>
) -> Option<Vec<PayeeType>> {
    match o {
        None => None,
        Some(v) => Some(
            v.into_iter()
            .map(|a: AffiliateType| PayeeType::from(a))
            .collect::<Vec<PayeeType>>()
        )
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetAffiliateCommissions {
    user_id: String,
}

pub async fn read_affiliate_commissions(
    req: HttpRequest,
    json: Json<GetAffiliateCommissions>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    debug!("json body: {:?}", body);

    let user_id = body.user_id;

    debug!("retrieving summary of payouts for user: {:?}", &user_id);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let seller_affiliate_commissions = db::read_payout_item_history_summaries(
        &conn,
        &user_id,
        Some(vec![PayeeType::SELLER_AFFILIATE])
    ).map_err(Error::from)?;

    let buyer_affiliate_commissions = db::read_payout_item_history_summaries(
        &conn,
        &user_id,
        Some(vec![PayeeType::BUYER_AFFILIATE])
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "sellerProgramCommission": seller_affiliate_commissions,
            "buyerProgramCommission": buyer_affiliate_commissions,
        })))
}
