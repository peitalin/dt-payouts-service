use actix_web::{
    HttpRequest,
    HttpResponse,
    HttpMessage,
    web, web::Query, web::Json,
    Error,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use futures::future::{
    Future,
    Either,
};

use crate::db;
use crate::db::GetPool;
use crate::models::{
    DbError,
    ErrJson,
    PayoutSplit,
    PayoutDealType,
    CLICK_COOKIE_NAME,
    RpcError,
    AffiliateError,
    Affiliate,
};
use crate::{AppState};
use crate::pricing::{
    PLATFORM_FEE_PERCENTAGE, // 15%
    SELLER_AFFILIATE_FEE_PERCENTAGE, // 5%
    BUYER_AFFILIATE_FEE_PERCENTAGE, // 5%
};
use crate::models::get_one_year_from_now;
use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use crate::rpc::rpc_get_affiliate_profile_by_click_id;


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadManyPayoutSplitsBody {
    payout_split_ids: Vec<String>,
}

pub async fn read_many_payout_splits(
    req: HttpRequest,
    json: Json<ReadManyPayoutSplitsBody>,
) -> Result<HttpResponse, Error> {

    let payout_split_ids: Vec<String> = json.into_inner().payout_split_ids;
    debug!("payout_split_ids: {:?}", &payout_split_ids);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_splits = db::read_payout_splits_by_ids(&conn, payout_split_ids)?;
    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payout_splits))
}



#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadPayoutSplitBody {
    payout_split_id: String,
}

pub async fn read_payout_split(
    req: HttpRequest,
    query: Query<ReadPayoutSplitBody>,
) -> Result<HttpResponse, Error> {

    let payout_split_id = query.into_inner().payout_split_id;
    debug!("payout_split_id: {:?}", &payout_split_id);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_split = db::read_payout_split(&conn, payout_split_id)?;
    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payout_split))
}



#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritePayoutSplitBody {
    store_or_user_id: String,
    deal_type: PayoutDealType,
    expires_at: Option<chrono::NaiveDateTime>,
    rate: Option<f64>,
    referrer_id: Option<String>,
}

pub async fn write_payout_split(
    req: HttpRequest,
    json: Json<WritePayoutSplitBody>,
) -> Result<HttpResponse, Error> {

    let params = json.into_inner();
    debug!("json: {:?}", &params);
    let payout_split = PayoutSplit::new(
        params.store_or_user_id,
        params.deal_type,
        params.expires_at,
        params.rate.unwrap_or(BUYER_AFFILIATE_FEE_PERCENTAGE),
        params.referrer_id,
    );
    debug!("payout_split: {:?}", &payout_split);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_split = db::write_payout_split(&conn, payout_split)?;

    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payout_split))
}



#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteSellerAndSellerAffiliatePayoutSplitBody {
    /// This is the seller who signs up (store ID)
    referred_seller_id: String,
    /// This is the user ID of the affiliate who referred the seller
    affiliate_user_id: String,
    /// This is the seller affiliate who referred to referred seller
    seller_affiliate: SellerAffiliatePayoutSplitBody
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerAffiliatePayoutSplitBody {
    /// Default is 1yr from now if not provided
    #[serde(default)]
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    expires_at: Option<chrono::NaiveDateTime>,
    /// Only for Seller Affiliate, sellers do not need to set one, default is 85%
    /// unless manually set by admins.
    /// Default Seller Affiliate fee is 5% if not provided
    rate: Option<f64>,
}



pub async fn write_seller_seller_aff_payout_split(
    req: HttpRequest,
    json: Json<WriteSellerAndSellerAffiliatePayoutSplitBody>,
) -> Result<HttpResponse, Error> {

    let params = json.into_inner();
    debug!("json: {:?}", &params);
    debug!("req cookies: {:?}", req.cookies());

    // 1. check to see if affiliate cookie is present.
    // Return early if not.
    let seller_affiliate: Affiliate = match req.cookie(CLICK_COOKIE_NAME) {
        None => {
            debug!("No cookie: {:?} provided", CLICK_COOKIE_NAME);
            debug!("Skipping PayoutSplit creation.");
            // Exit early if cookie not present
            return Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .json(json!({
                        "referredSeller": None as Option<String>,
                        "sellerAffiliate": None as Option<String>
                    })))
        },
        Some(click_cookie) => {
            rpc_get_affiliate_profile_by_click_id(
                AppState::httpClient(&req),
                click_cookie.value()
            ).await?
        },
    };

    let expiry_date = match params.seller_affiliate.expires_at {
        Some(d) => Some(d),
        None => {
            debug!("No expire_at date given, setting default as 1yr from now");
            Some(get_one_year_from_now())
        },
    };

    let seller_aff_payout_split = PayoutSplit::new(
        seller_affiliate.user_id,
        PayoutDealType::SELLER_AFFILIATE,
        expiry_date,
        params.seller_affiliate.rate
            .unwrap_or(SELLER_AFFILIATE_FEE_PERCENTAGE), // 5% default if not provided
        None,
    );

    let referred_seller_payout_split = PayoutSplit::new(
        params.referred_seller_id,
        PayoutDealType::REFERRED_SELLER,
        None, // no expiry on the seller's 85% default rate
        1.0 - PLATFORM_FEE_PERCENTAGE, // 85% default rate for sellers
        Some(seller_aff_payout_split.id.clone()), // payoutSplitId of the referrer
    );

    debug!("Writing: ");
    debug!("referred seller's payout split: {:?}", &referred_seller_payout_split);
    debug!("seller affiliate's payout split: {:?}", &seller_aff_payout_split);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let (ref_seller_psplit, seller_aff_psplit) = db::write_two_payout_splits(
        &conn,
        referred_seller_payout_split,
        seller_aff_payout_split
    )?;

    // Returns PayoutSplits for seller and his referrer
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(json!({
        "referredSeller": ref_seller_psplit,
        "sellerAffiliate": seller_aff_psplit
    })))
}




#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadCurrentPayoutSplitsByStoreIdBody {
    store_or_user_ids: Vec<String>,
    payout_deal_types: Option<Vec<PayoutDealType>>,
}
pub async fn read_current_payout_splits_by_ids(
    req: HttpRequest,
    json: Json<ReadCurrentPayoutSplitsByStoreIdBody>,
) -> Result<HttpResponse, Error> {

    let json = json.into_inner();
    let store_or_user_ids: Vec<String> = json.store_or_user_ids;
    let payout_deal_types: Option<Vec<PayoutDealType>> = json.payout_deal_types;
    debug!("store_or_user_ids: {:?}", &store_or_user_ids);
    debug!("payout_deal_types: {:?}", &payout_deal_types);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_splits = db::read_current_payout_splits_by_store_or_user_ids(
        &conn,
        &store_or_user_ids,
        payout_deal_types,
    )?;
    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payout_splits))
}


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadPayoutSplitsOfUser {
    user_id: String,
    payout_deal_types: Option<Vec<PayoutDealType>>,
}
pub async fn read_payout_splits_of_user(
    req: HttpRequest,
    json: Json<ReadPayoutSplitsOfUser>,
) -> Result<HttpResponse, Error> {

    let json = json.into_inner();
    let user_id: String = json.user_id;
    let payout_deal_types: Option<Vec<PayoutDealType>> = json.payout_deal_types;
    debug!("user_id: {:?}", &user_id);
    debug!("payout_deal_types: {:?}", &payout_deal_types);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_splits = db::read_payout_splits_of_user_id(
        &conn,
        user_id,
        payout_deal_types,
    )?;
    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payout_splits))
}


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePayoutSplit {
    payout_split_id: String,
}

pub async fn delete_payout_split(
    req: HttpRequest,
    query: Query<DeletePayoutSplit>,
) -> Result<HttpResponse, Error> {

    let params = query.into_inner();
    debug!("json: {:?}", &params);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_split = db::delete_payout_split(
        &conn,
        &params.payout_split_id
    )?;

    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payout_split))
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePayoutSplitUserId {
    user_id: String,
}

pub async fn delete_all_payout_splits_for_user(
    req: HttpRequest,
    query: Query<DeletePayoutSplitUserId>,
) -> Result<HttpResponse, Error> {

    let params = query.into_inner();
    debug!("json: {:?}", &params);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let delete_response = db::delete_all_payout_splits_for_user_id(
        &conn,
        &params.user_id
    )?;

    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(delete_response))
}


pub async fn update_payout_split(
    req: HttpRequest,
    json: Json<WritePayoutSplitBody>,
) -> Result<HttpResponse, Error> {

    let params = json.into_inner();
    debug!("json: {:?}", &params);

    if params.deal_type == PayoutDealType::SELLER_AFFILIATE {
        return Err(AffiliateError::CreateSellerAffiliateManually(
                errJson!("Cannot manually create SELLER_AFFILIATE PayoutSplit")
            )).map_err(Error::from)
    }

    let payout_split = PayoutSplit::new(
        params.store_or_user_id,
        params.deal_type.clone(),
        params.expires_at,
        params.rate.unwrap_or(BUYER_AFFILIATE_FEE_PERCENTAGE),
        params.referrer_id,
    );

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;


    if params.deal_type == PayoutDealType::SELLER {
        /// For SELLER and REFERRED_SELLER there is potentially a clash.
        /// A seller may have a REFERRED_SELLER payoutsplit, which may be
        /// overritten by a manually created SELLER PayoutSplit.
        ///
        /// This function prevents that from happening accidentally,
        /// by first checking if REFERRED_SELLER PayoutSplits exist for a seller,
        /// then creating a SELLER PayoutSplit that correctly retains the
        /// REFERRED_SELLER's PayoutSplit.referrer_id
        let arg_store_or_user_ids = vec![payout_split.store_or_user_id.clone()];
        let arg_payout_deal_types = vec![PayoutDealType::REFERRED_SELLER];

        debug!(
            r#"Checking if new PayoutSplit.store_or_user_id has
            a pre-existing REFERRED_SELLER: {:?}"#,
            &payout_split
        );
        let vec_existing_psplit = db::read_current_payout_splits_by_store_or_user_ids(
            &conn,
            &arg_store_or_user_ids,
            Some(arg_payout_deal_types)
        ).ok();

        if let Some(vpsplit) = vec_existing_psplit {
            if let Some(existing_psplit) = vpsplit.iter().next() {
                if let Some(referrer_id_str) = &existing_psplit.referrer_id {

                    debug!(r#"A REFERRED_SELLER PayoutSplit exists, update
                    referred_id on manually created PayoutSplit before saving"#);

                    let newest_payout_split = db::write_payout_split(
                        &conn,
                        payout_split
                            .update_referrer_id(referrer_id_str.clone())
                            .update_deal_type(PayoutDealType::REFERRED_SELLER)
                        // &str -> String
                    )?;

                    // Returns a PayoutMethod with updated referrer_id
                    return Ok(HttpResponse::Ok()
                            .content_type("application/json")
                            .json(newest_payout_split))
                }
            }
        }
    }

    debug!(r#"No existing PayoutSplit exists, carry on with creating a
        manually created SELLER PayoutSplit"#);

    let newest_payout_split = db::write_payout_split(&conn, payout_split)?;

    // Returns a PayoutMethod (camelcased)
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(newest_payout_split))
}












#[test]
fn deserializes_seller_affiliate_payout_split_body_null_expiry() {

    let test_str = r#"
    {
        "expiresAt": null,
        "rate": 0.05
    }
    "#;

    let res = serde_json::from_str::<SellerAffiliatePayoutSplitBody>(test_str);
    debug!("response: {:?}", res);

    match res {
        Ok(s) => assert_eq!(s.rate, Some(0.05)),
        Err(_e) => {
            debug!("Deserializing date: null should fail");
        }
    }
}











#[test]
fn deserializes_seller_affiliate_payout_split_body_no_expiry() {

    let test_str = r#"
    {
        "rate": 0.05
    }
    "#;

    let res = serde_json::from_str::<SellerAffiliatePayoutSplitBody>(test_str);
    debug!("response: {:?}", res);

    match res {
        Ok(s) => assert_eq!(s.rate, Some(0.05)),
        Err(e) => panic!(e.to_string()),
    }
}