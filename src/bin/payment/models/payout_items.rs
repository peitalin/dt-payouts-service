// Needed for diesel table schemas
use diesel::prelude::*;
use gm::db::schema::payouts;
use gm::db::schema::payout_items;
use gm::utils::dates::from_datetimestr_to_option_naivedatetime;

use diesel::deserialize::FromSql;
use diesel::result::Error::DeserializationError;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Double, Float, Jsonb, Json, Text, BigInt, Timestamp, Nullable};

use std::str::FromStr;
use uuid;
use itertools::Itertools;
use std::collections::HashMap;

use crate::models::paypal::{
  PaypalLink,
  PaypalValue,
};
use crate::models::{
    OrderItemRpc,
    ConnectionQuery,
    DbError,
    PayoutPeriod,
    Currency,
    PayoutMethod,
    PayoutType,
    PayeeType,
    PayoutStatus,
    PayeeId,
    PayoutEmail,
};
use crate::models::payouts::aggregate_payout_totals_by_payee_id;



#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Queryable, Insertable, QueryableByName)]
#[table_name = "payout_items"]
pub struct PayoutItem {
    pub id: String,
    pub payee_id: String,
    pub payee_type: PayeeType,
    pub amount: i32,
    pub payment_processing_fee: i32,
    pub created_at: chrono::NaiveDateTime,
    pub payout_status: PayoutStatus,
    pub currency: String,
    pub order_item_id: String,
    pub txn_id: String,
    pub payout_id: Option<String>,
}
impl PayoutItem {
    pub fn new(
        order_item_id: String,
        payee_id: String,
        payee_type: Option<PayeeType>,
        amount: i32,
        payment_processing_fee: i32,
        created_at: chrono::NaiveDateTime,
        currency: String,
        txn_id: String,
    ) -> Self {

        Self {
            id: format!("pitem_{}", uuid::Uuid::new_v4().to_string()),
            payee_id: payee_id.clone(),
            payee_type: payee_type.unwrap_or(PayeeType::STORE),
            amount: amount,
            payment_processing_fee: payment_processing_fee,
            created_at: created_at,
            payout_status: PayoutStatus::UNPAID,
            currency: currency,
            order_item_id: order_item_id,
            txn_id: txn_id,
            payout_id: None,
        }
    }

    pub fn to_refund(
        &self,
        created_at: chrono::NaiveDateTime,
        txn_id: String,
    ) -> Self {
        // clones existing payoutItem as a refundItem
        Self {
            id: format!("ritem_{}", uuid::Uuid::new_v4()),
            payee_id: self.payee_id.clone(),
            payee_type: self.payee_type.clone(),
            amount: -self.amount.clone(),
            payment_processing_fee: -self.payment_processing_fee.clone(),
            created_at: created_at,
            payout_status: PayoutStatus::REFUNDING,
            currency: self.currency.clone(),
            order_item_id: self.order_item_id.clone(),
            txn_id: txn_id,
            payout_id: None,
        }
    }

    pub fn set_payout_status(mut self, payout_status: PayoutStatus) -> Self {
        self.payout_status = payout_status;
        self
    }
}

impl std::default::Default for PayoutItem {
    fn default() -> Self {
        Self {
            id: format!("pitem_{}", uuid::Uuid::new_v4().to_string()),
            payee_id: String::from(""),
            payee_type: PayeeType::STORE,
            amount: 0,
            payment_processing_fee: 0,
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            payout_status: PayoutStatus::UNPAID,
            currency: Currency::USD.as_string(),
            order_item_id: String::from(""),
            txn_id: String::from(""),
            payout_id: None,
        }
    }
}


#[derive(Clone, Debug, Serialize, QueryableByName)]
pub struct PayoutItemAggregates {
    #[sql_type = "Timestamp"]
    pub created_at: chrono::NaiveDateTime,
    #[sql_type = "BigInt"]
    pub amount_total: i64,
    #[sql_type = "BigInt"]
    pub fees_total: i64,
    #[sql_type = "BigInt"]
    pub unpaid: i64,
    #[sql_type = "BigInt"]
    pub count: i64,
}

impl PayoutItemAggregates {
    pub fn new() -> Self {
        Self {
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            amount_total: 0,
            fees_total: 0,
            unpaid: 0,
            count: 0,
        }
    }
}

impl std::default::Default for PayoutItemAggregates {
    fn default() -> Self {
        Self {
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            amount_total: 0,
            fees_total: 0,
            unpaid: 0,
            count: 0,
        }
    }
}



#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize, QueryableByName)]
pub struct PayoutItemHistorySummaries {
    #[sql_type = "Nullable<Json>"]
    pub today: Option<SummaryStatistics>,
    #[sql_type = "Nullable<Json>"]
    pub last_7_days: Option<SummaryStatistics>,
    #[sql_type = "Nullable<Json>"]
    pub last_30_days: Option<SummaryStatistics>,
    #[sql_type = "Nullable<Json>"]
    pub last_period: Option<SummaryStatistics>,
    #[sql_type = "Nullable<Json>"]
    pub current_period: Option<SummaryStatistics>,
    #[sql_type = "Nullable<Json>"]
    pub all_time: Option<SummaryStatistics>,
}
impl PayoutItemHistorySummaries {
    pub fn new() -> Self {
        Self {
            today: Some(SummaryStatistics { ..Default::default() }),
            last_7_days: Some(SummaryStatistics { ..Default::default() }),
            last_30_days: Some(SummaryStatistics { ..Default::default() }),
            last_period: Some(SummaryStatistics { ..Default::default() }),
            current_period: Some(SummaryStatistics { ..Default::default() }),
            all_time: Some(SummaryStatistics { ..Default::default() }),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, QueryableByName)]
pub struct SummaryStatistics {
    #[sql_type = "BigInt"]
    #[serde(rename(serialize = "amountTotal"))]
    pub amount_total: i64,
    #[sql_type = "BigInt"]
    pub unpaid: i64,
    #[sql_type = "BigInt"]
    pub count: i64,
}
impl FromSql<Json, Pg> for SummaryStatistics {
    fn from_sql(maybe_bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<Json, Pg>>::from_sql(maybe_bytes)?;
        serde_json::from_value::<SummaryStatistics>(value).map_err(|e| e.into())
    }
}



#[test]
fn sums_payout_items_correctly() {
    use crate::db;

    let conn = gm::db::establish_connection_pg("DATABASE_URL");

    let payout_period = PayoutPeriod::new(2019, 10).unwrap();

    let (payout_items, _count, _isLastPage) = db::read_payout_items_in_period_paginate_by_cursor(
        &conn,
        payout_period.start_period,
        payout_period.end_period,
        Some(PayoutStatus::UNPAID),
        ConnectionQuery {
            sortAscending: Some(true),
            cursor: Some(base64::encode(&String::from("created_at:2019-10-10T00:00:00Z"))),
            pageBackwards: Some(false),
            count: 3,
        }
    ).unwrap();

    let mut test_hmap: HashMap<PayeeId, PayoutEmail> = HashMap::new();
    let test_pm_hmap: HashMap<PayeeId, PayoutMethod> = HashMap::new();

    let payout_items = payout_items.into_iter().map(|pitem: PayoutItem| {
        test_hmap.insert(
            pitem.payee_id.clone(),
            format!("{}@testemail.com", pitem.payee_id.clone())
        );
        pitem
    }).collect::<Vec<PayoutItem>>();

    let payout_hmap = aggregate_payout_totals_by_payee_id(
        payout_period,
        payout_items,
        test_hmap,
        test_pm_hmap,
        String::from("user_123123123123"),
    );

    println!("\n__payout_groups__\n{:#?}\n", payout_hmap);
    for (payee_id, payout_group) in payout_hmap.iter() {
        assert_eq!(payout_group.payee_id, *payee_id)
    };

}

