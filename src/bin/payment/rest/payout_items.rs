use actix::{Actor, Addr};
use actix_web::{
    client::ClientResponse,
    HttpRequest, HttpResponse,
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
use gm::utils::dates::from_datetimestr_to_naivedatetime;
use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use std::str::FromStr;

use crate::AppState;
use crate::models::{
    ErrJson,
    PayoutItem,
    PayoutStatus,
    B64Cursor,
    PayoutItemAggregates,
    PayoutPeriod,
};
use crate::models::connection::{
    ConnectionQuery,
    Edge,
    Connection,
    PageInfo,
    PageBasedConnectionQuery,
    PageBasedConnection,
    PageBasedConnectionPageInfo,
    PageBasedEdge,
};
use crate::db;
use crate::db::{ GetPool };


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadManyPayoutItemsBody {
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub start_date: chrono::NaiveDateTime,
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub end_date: chrono::NaiveDateTime,
    pub payout_status: Option<PayoutStatus>,
    pub store_id: Option<String>,
}


pub async fn read_payout_items_in_period(
    req: HttpRequest,
    json: Json<ReadManyPayoutItemsBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let start_date = body.start_date;
    let end_date = body.end_date;
    let store_id = body.store_id;
    let payout_status = match body.payout_status {
        Some(p) => Some(vec![p]),
        None => None,
    };

    debug!(
        "retrieving payout_items between : {:?} and {:?}",
        &start_date,
        &end_date,
    );

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_items = db::read_payout_items_in_period(
        &conn,
        start_date,
        end_date,
        payout_status,
        store_id,
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(json!({
        "payout_items": payout_items
    })))
}


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadManyPayoutItemsByIdBody {
    pub payout_item_ids: Vec<String>,
}

pub async fn read_payout_items_by_ids(
    req: HttpRequest,
    json: Json<ReadManyPayoutItemsByIdBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let payout_item_ids = body.payout_item_ids;

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_items = db::read_payout_items_by_ids(
        &conn,
        &payout_item_ids,
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(payout_items))
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadManyPayoutItemsByOrderItemIdBody {
    pub order_item_ids: Vec<String>,
}

pub async fn read_payout_items_by_order_item_ids(
    req: HttpRequest,
    json: Json<ReadManyPayoutItemsByOrderItemIdBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let order_item_ids = body.order_item_ids;

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let payout_items = db::read_payout_items_by_order_item_ids(
        &conn,
        &order_item_ids,
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(payout_items))
}



#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadPayoutItemsConnectionBody {
    month: i32,
    year: i32,
    payout_status: Option<PayoutStatus>,
    query: ConnectionQuery,
}

pub async fn read_payout_items_cursor_connection(
    req: HttpRequest,
    json: Json<ReadPayoutItemsConnectionBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let payout_period: PayoutPeriod = PayoutPeriod::new(
        body.year,
        body.month
    ).map_err(Error::from)?;

    let sort_ascending = body.query.sortAscending.clone();
    debug!("payout_items_connection: incoming body: {:?}", body);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let (
        vecPitems,
        numPages,
        isLastPage
    ) = db::read_payout_items_in_period_paginate_by_cursor(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        body.payout_status,
        body.query
    ).map_err(Error::from)?;

    let agg: PayoutItemAggregates = db::read_payout_item_aggregates(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        sort_ascending.unwrap_or(false),
    );

    let endCursor = match agg.count {
        0 => None,
        _ => Some(base64::encode(&format!("created_at:{}", agg.created_at)))
    };

    let connection = Connection::<PayoutItem> {
        pageInfo: PageInfo {
            endCursor: endCursor,
            isLastPage: isLastPage,
            totalPages: Some(numPages),
        },
        totalCount: Some(agg.count),
        totalAmount: Some(agg.amount_total),
        totalFees: Some(agg.fees_total),
        edges: vecPitems.into_iter().map(|payout_item| {
            let edgeCursor = format!("created_at:{:?}", &payout_item.created_at);
            Edge {
                cursor: Some(base64::encode(&edgeCursor)),
                node: payout_item
            }
        }).collect::<Vec<Edge<PayoutItem>>>()
    };

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(connection))
}



#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadPayoutItemsPageConnectionBody {
    month: i32,
    year: i32,
    payout_status: Option<PayoutStatus>,
    query: PageBasedConnectionQuery,
}

pub async fn read_payout_items_page_connection(
    req: HttpRequest,
    json: Json<ReadPayoutItemsPageConnectionBody>,
) -> Result<HttpResponse, Error> {

    let body = json.into_inner();
    let payout_period: PayoutPeriod = PayoutPeriod::new(
        body.year,
        body.month
    ).map_err(Error::from)?;

    let pageNumber = body.query.pageNumber.clone();
    let sort_ascending = body.query.sortAscending.clone();
    debug!("payout_items_connection: incoming body: {:?}", body);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let (
        vecPitems,
        numPages,
    ) = db::read_payout_items_in_period_paginate_by_page(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        body.payout_status,
        body.query
    ).map_err(Error::from)?;

    let agg: PayoutItemAggregates = db::read_payout_item_aggregates(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        sort_ascending.unwrap_or(false),
    );

    let connection = PageBasedConnection::<PayoutItem> {
        pageInfo: PageBasedConnectionPageInfo {
            isLastPage: pageNumber == numPages,
            totalPages: Some(numPages),
            pageNumber: Some(pageNumber.clone()),
        },
        totalCount: Some(agg.count),
        totalAmount: Some(agg.amount_total),
        totalFees: Some(agg.fees_total),
        edges: vecPitems.into_iter().map(|payout_item| {
            PageBasedEdge {
                node: payout_item
            }
        }).collect::<Vec<PageBasedEdge<PayoutItem>>>()
    };

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(connection))
}


#[test]
fn deserializes_payout_items_body() {

    let test_str = r#"
    {
        "month": 10,
        "year": 2019,
        "query": {
            "sortAscending": false,
            "cursor": null,
            "pageBackwards": false,
            "count": 5
        }
    }
    "#;

    let res = serde_json::from_str::<ReadPayoutItemsConnectionBody>(test_str);
    match res {
        Ok(cnn) => {
            assert_eq!(
                cnn.query.sortAscending,
                Some(false),
            )
        },
        Err(e) => panic!(e.to_string()),
    }
}
