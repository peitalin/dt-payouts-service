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




#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePayoutBody {
    month: i32,
    year: i32,
    mode: Option<String>,
}

pub async fn create_payout(
    req: HttpRequest,
    json: Json<CreatePayoutBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let payout_period: PayoutPeriod = PayoutPeriod::new(
        body.year,
        body.month
    ).map_err(Error::from)?;

    debug!(
        "retrieving payout_items between: {:?} and {:?}",
        &payout_period.start_period,
        &payout_period.end_period,
    );

    // 1. send auth-cookie in request to user-service to get user role
    // to see if you have permission to approve payouts
    let auth_info: AuthInfo = rpc::rpc_get_auth_info(
        &AppState::from(&req).http_client,
        &req
    ).await?;

    // 2. guard against non-authorized access. returns if auth errors.
    match (body.mode, get_gm_environment().as_str()) {
        (None, _) => {
            is_worthy_enough(&auth_info.user_role).map_err(Error::from)?;
        },
        (Some(_mode), "production") => {
            is_worthy_enough(&auth_info.user_role).map_err(Error::from)?;
        },
        (Some(_mode), _) => {},
        // skip auth check when `mode` is provided in develop environment.
    };

    // 3. get Db connection from pool
    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    // 3a. get all UNPAID and REFUNDING items in the period
    let payout_items = db::read_payout_items_in_period(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        Some(vec![
            PayoutStatus::UNPAID,
            PayoutStatus::MISSING_PAYOUT_METHOD,
            // try and see if user has added payout second time around.
            PayoutStatus::REFUNDING
        ]),
        None,
    )?;

    // 3b. split UNPAID and REFUNDING items
    let payout_items_refunding: Vec<_> = payout_items.clone()
        .into_iter()
        .filter(|p: &PayoutItem| p.payout_status == PayoutStatus::REFUNDING)
        .collect::<Vec<PayoutItem>>();

    let payout_items_unpaid: Vec<_> = payout_items.clone()
        .into_iter()
        .filter(|p: &PayoutItem| {
            p.payout_status == PayoutStatus::UNPAID ||
            p.payout_status == PayoutStatus::MISSING_PAYOUT_METHOD
        })
        .collect::<Vec<PayoutItem>>();

    // 3c. get UNPAID items' storeIds/affiliateIds
    let payee_ids = payout_items_unpaid.iter()
        .map(|pitem: &PayoutItem| pitem.payee_id.clone())
        .collect::<Vec<String>>();

    info!("payout items: {:?}", payout_items_unpaid);
    info!("refund items: {:?}", payout_items_refunding);
    info!("retreiving user_ids for stores: {:?}", payee_ids);

    // 4a. Get payout_methods associated with payeeId
    let payout_methods = db::read_payout_methods_by_payee_ids(
        &conn,
        &payee_ids
    ).map_err(Error::from)?;

    // 4b. pull out payout emails data from rpc call
    let payout_emails_hashmap: HashMap<PayeeId, PayoutEmail> =
        payouts::create_payout_emails_hashmap(payout_methods.clone());

    // 4b. pull out paymethod_method data from rpc call
    let payout_methods_hashmap: HashMap<PayeeId, PayoutMethod> =
        payouts::create_payout_method_hashmap(payout_methods);

    debug!("payout_emails: {:?}", payout_emails_hashmap);

    // 5a. Group payout items, and create Payouts for each group (PayeeId)
    // includes refund items to deduct payouts.
    let payout_hashmap: HashMap<PayeeId, Payout> =
        payouts::aggregate_payout_totals_by_payee_id(
            payout_period,
            payout_items,
            payout_emails_hashmap,
            payout_methods_hashmap,
            auth_info.user_id,
        );

    // 5b. Partition Payouts by whether has payout email or not.
    let (
        payouts_missing_payout_method_vec,
        payouts_vec
    ): (Vec<_>, Vec<_>) = payout_hashmap
        .into_iter()
        .partition_map(|p: (PayeeId, Payout)| {
            // check whether email is empty
            match (p.0, p.1.payout_email.as_ref()) {
                (_, "") => EitherLR::Left(p.1),
                (_, _) => EitherLR::Right(p.1),
            }
        });


    // 5c. Extract IDs for each payout group
    let payout_item_ids = payouts_vec
        .iter()
        .flat_map(|p: &Payout| p.payout_item_ids.clone())
        .collect::<Vec<String>>();

    let missing_payout_method_ids = payouts_missing_payout_method_vec
        .iter()
        .flat_map(|p: &Payout| p.payout_item_ids.clone())
        .collect::<Vec<String>>();

    let refund_item_ids = payout_items_refunding
        .iter()
        .map(|pitem: &PayoutItem| pitem.id.clone())
        .collect::<Vec<String>>();

    // If no payoutItems are found in this month, return
    if payouts_vec.len() < 1 {
        debug!("No payouts are ready to be paid out...");
        if payout_items_unpaid.len() > 0 {
            debug!("Even though UNPAID payout_items exist,");
            debug!("These payoutitem items may be missing payout_emails!");
        };
        return Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(
                vec![] as Vec<Payout>
            ))
    }


    // 4 write groups to payout db
    let payout_writes = db::write_many_payouts(
        &conn,
        &payouts_vec,
        &payout_item_ids,
        &missing_payout_method_ids,
        &refund_item_ids,
    ).map_err(Error::from)?;

    debug!("Wrote payouts: {:?}", payout_writes);

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(
        payout_writes
    ))
}


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovePayoutBody {
    payout_ids: Vec<String>,
    payout_email_subject: Option<String>,
    payout_email_message: Option<String>,
}

pub async fn approve_payout(
    req: HttpRequest,
    json: Json<ApprovePayoutBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let payout_ids = body.payout_ids;
    let payout_email_subject = body.payout_email_subject;
    let payout_email_message = body.payout_email_message;

    info!("incoming cookies: {:?}", req.cookies());
    // 1. send auth-cookie in request to user-service to get user role
    // to see if you have permission to approve payouts
    let auth_info: AuthInfo = rpc::rpc_get_auth_info(
        &AppState::from(&req).http_client,
        &req
    ).await?;
    // 2. guard against non-authorized access. returns if auth errors.
    is_worthy_enough(&auth_info.user_role).map_err(Error::from)?;


    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payouts_pending_approval = db::read_many_payouts(
        &conn,
        &payout_ids
    )?;

    debug!("payouts pending approval: {:?}", payouts_pending_approval);

    // if admin has already approved, group those payouts here
    let already_approved_by_this_admin = payouts_pending_approval
        .iter()
        .filter(|p: &&Payout| p.approved_by_ids.contains(&auth_info.user_id))
        .map(|p: &Payout| p.id.clone())
        .collect::<Vec<String>>();

    debug!(
        "payouts already approved by this admin: {:?}",
        already_approved_by_this_admin,
    );

    // 3. Approve payouts
    let signed_payouts = SignedPayouts::new(
        payouts_pending_approval,
        &auth_info.user_id,
    );

    // Payouts with 2 signatures
    let approved_ids = signed_payouts.get_ids(PayoutApprovalType::Approved);
    // Payouts with 1 signature
    let pending_ids = signed_payouts.get_ids(PayoutApprovalType::Pending);
    debug!("Twice approved payouts: {:?}", &approved_ids.payout_ids);

    if approved_ids.payout_ids.len() == 0 {
        // 4. If no twice approved payouts, simply return
        Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "approvedPayouts": signed_payouts.approved_payouts,
            "payoutsAlreadyApprovedIds": already_approved_by_this_admin,
            "paypalPayoutResponse": "",
        })))
    } else {

        // 4. Prepare Paypal Payout request body
        let paypal_payout_params = PaypalPayoutParams::new(None)
            .set_email_subject(payout_email_subject)
            .set_email_message(payout_email_message)
            .set_items(
                signed_payouts.approved_payouts.iter()
                    // skip $0.00 payouts
                    .filter(|p: &&Payout| p.amount != 0)
                    .map(PaypalPayout::from)
                    .collect::<Vec<PaypalPayout>>()
                    // Map Payouts to PaypalPayouts
            );

        debug!("{:?}", paypal_payout_params);

        // 4. Dispatch payouts to payout processor
        let paypal_payout_response: PaypalPayoutResponse = paypal::create_batch_payout(
            req,
            paypal_payout_params
        ).await?;

        // Set approver Ids for payouts
        // and set Payout status as PROCESSING
        let paid_payouts = db::approve_many_payouts(
            &conn,
            approved_ids,
            pending_ids
        ).map_err(Error::from)?;

        // set PayoutItems + Payouts statues from PROCESSING to PAID
        // set Payouts.payout_batch_id to payout_batch_id
        let _ = db::update_payouts_post_paypal_payout(
            &conn,
            &paid_payouts.approved_payout_ids,
            &paid_payouts.refunding_payout_ids,
            paypal_payout_response.batch_header.payout_batch_id.clone(),
        ).map_err(Error::from)?;

        Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "approvedPayouts": paid_payouts.approved_payouts,
            "payoutsAlreadyApprovedIds": already_approved_by_this_admin,
            "paypalPayoutResponse": paypal_payout_response,
        })))
    }
}




#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadPayoutsBody {
    month: i32,
    year: i32,
    payout_status: Option<PayoutStatus>,
    query: ConnectionQuery,
}

pub async fn read_payouts_connection(
    req: HttpRequest,
    json: Json<ReadPayoutsBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let payout_period: PayoutPeriod = PayoutPeriod::new(
        body.year,
        body.month
    ).map_err(Error::from)?;

    let sort_ascending = body.query.sortAscending.clone();
    let payout_status = body.payout_status;
    let query = body.query;

    debug!(
        "retrieving payout_items between : {:?} and {:?}",
        &payout_period.start_period,
        &payout_period.end_period,
    );

    // 2. Do Db actions
    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;


    let (payouts, numPages, isLastPage) = db::read_many_payouts_in_period_paginated(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        payout_status,
        query,
    ).map_err(Error::from)?;

    let next_payout_period = PayoutPeriod::get_next_payout_period(payout_period)
        .map_err(Error::from)?;
    // payouts for 1May~1June will be created on 15th June, so need to increment
    // period to 1Jun~1July to retrieve the payouts
    // created on 15th June, for May period.
    let agg: PayoutAggregates = db::read_payout_aggregates(
        &conn,
        next_payout_period.start_period,
        next_payout_period.end_period,
        sort_ascending.unwrap_or(false),
    );

    let endCursor = match agg.count {
        0 => None,
        _ => Some(base64::encode(&format!("created_at:{}", agg.created_at)))
    };
    // Note: because payouts are inserted in batches, cursor will likely not
    // be unique, need to use limit-offset style paginators

    let connection = Connection::<Payout> {
        pageInfo: PageInfo {
            endCursor: endCursor,
            isLastPage: isLastPage,
            totalPages: Some(numPages),
        },
        totalCount: Some(agg.count),
        totalAmount: Some(agg.amount_total),
        totalFees: None,
        edges: payouts.into_iter().map(|payout| {
            let edgeCursor = format!("created_at:{:?}", &payout.created_at);
            Edge {
                cursor: Some(base64::encode(&edgeCursor)),
                node: payout
            }
        }).collect::<Vec<Edge<Payout>>>()
    };

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(connection))
}



#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetPayoutsByIdsBody {
    payout_ids: Vec<String>,
}

pub async fn read_payouts_by_ids(
    req: HttpRequest,
    json: Json<GetPayoutsByIdsBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let payout_ids = body.payout_ids;

    debug!("retrieving payouts by ids: {:?}", &payout_ids);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;


    let payouts = db::read_many_payouts(&conn, &payout_ids)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(payouts))
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetPayoutsByStoreIdBody {
    /// store_id is also the same id for paying affiliates
    /// e.g. payee_id
    store_id: String,
    query: ConnectionQuery,
}

pub async fn read_payouts_by_store_id_connection(
    req: HttpRequest,
    json: Json<GetPayoutsByStoreIdBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let store_id = body.store_id;
    let sort_ascending = body.query.sortAscending.clone();
    let query = body.query;

    debug!("retrieving payouts by store_id: {:?}", &store_id);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let (payouts, numPages, isLastPage) = db::read_many_payouts_by_payee_id_paginated(
        &conn,
        &store_id,
        query,
    )?;

    let agg: PayoutAggregates = db::read_payout_aggregates_by_store_id(
        &conn,
        &store_id,
        sort_ascending.unwrap_or(false),
    );

    let endCursor = match agg.count {
        0 => None,
        _ => Some(base64::encode(&format!("created_at:{}", agg.created_at)))
    };

    let connection = Connection::<Payout> {
        pageInfo: PageInfo {
            endCursor: endCursor,
            isLastPage: isLastPage,
            totalPages: Some(numPages),
        },
        totalCount: Some(agg.count),
        totalAmount: Some(agg.amount_total),
        totalFees: None,
        edges: payouts.into_iter().map(|payout| {
            let edgeCursor = format!("created_at:{:?}", &payout.created_at);
            Edge {
                cursor: Some(base64::encode(&edgeCursor)),
                node: payout
            }
        }).collect::<Vec<Edge<Payout>>>()
    };

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(connection))

}


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetPayoutsByStoreIdInPeriodBody {
    /// store_id is also the same id for paying affiliates
    /// e.g. payee_id
    store_id: String,
    month: i32,
    year: i32
}

pub async fn read_payouts_by_store_id_in_period(
    req: HttpRequest,
    json: Json<GetPayoutsByStoreIdInPeriodBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let store_id = body.store_id;

    let payout_period: PayoutPeriod = PayoutPeriod::new(
        body.year,
        body.month
    ).map_err(Error::from)?;

    debug!("retrieving payouts by store_id: {:?}", &store_id);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payouts: Vec<Payout> = db::read_payouts_for_payee_id_in_period(
        &conn,
        &store_id,
        payout_period.start_period,
        payout_period.end_period
    )?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(payouts))
}



#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetPayoutHistorySummaries {
    store_id: String,
}

pub async fn read_store_payout_history_summaries(
    req: HttpRequest,
    json: Json<GetPayoutHistorySummaries>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let store_id = body.store_id;

    debug!("retrieving summary of payouts for store: {:?}", &store_id);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_history_summaries = db::read_payout_item_history_summaries(
        &conn,
        &store_id,
        Some(vec![PayeeType::STORE])
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(payout_history_summaries))
}

pub fn is_worthy_enough(user_role: &UserRole) -> Result<String, AuthError> {
    match user_role {
        UserRole::USER => Err(
            AuthError::NotWorthyEnough(ErrJson::new("USER role not worthy enough"))
        ),

        UserRole::ANON => Err(
            AuthError::NotWorthyEnough(ErrJson::new("ANON role not worthy enough"))
        ),

        UserRole::SYSTEM => Err(
            AuthError::NotWorthyEnough(ErrJson::new("SYSTEM role not worthy enough"))
        ),

        UserRole::PLATFORM_ADMIN => Ok(String::from("Authorized")),
    }
}
