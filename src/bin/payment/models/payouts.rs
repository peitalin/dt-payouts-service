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
    UserPublic,
    PayoutMethod,
    PayoutType,
    PayoutItem,
};

pub type PayeeId = String;
pub type PayoutEmail = String;



#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[derive(Queryable, Insertable)]
#[table_name = "payouts"]
pub struct Payout {
    pub id: String,
    pub payee_id: String,
    pub payee_type: PayeeType,
    pub amount: i32,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub start_period: Option<chrono::NaiveDateTime>,
    pub end_period: Option<chrono::NaiveDateTime>,
    pub payout_date: Option<chrono::NaiveDateTime>,
    pub payout_status: PayoutStatus,
    pub payout_email: String,
    pub currency: Currency,
    pub payout_item_ids: Vec<String>,
    pub approved_by_ids: Vec<String>,
    pub payout_batch_id: Option<String>,
    pub details: Option<String>,
    pub paid_to_payment_method_id: Option<String>,
}

impl Payout {
    pub fn new(
        payee_id: String,
        payout_period: PayoutPeriod,
        approver_id: String,
        payout_email: String,
        paid_to_payment_method_id: Option<String>,
    ) -> Self {
        Payout {
            id: format!("payout_{}", uuid::Uuid::new_v4().to_string()),
            payee_id: payee_id.clone(),
            payee_type:
                if payee_id.starts_with("s") {
                    PayeeType::STORE
                } else if payee_id.starts_with("aff_buyer") {
                    PayeeType::BUYER_AFFILIATE
                } else if payee_id.starts_with("aff_seller") {
                    PayeeType::SELLER_AFFILIATE
                } else {
                    PayeeType::PLATFORM
                },
            amount: 0,
            created_at: Some(chrono::NaiveDateTime::from_timestamp(
                                    chrono::Utc::now().timestamp(), 0)),
            start_period: Some(payout_period.start_period),
            end_period: Some(payout_period.end_period),
            payout_date: Some(payout_period.payout_date),
            payout_status: PayoutStatus::PENDING_APPROVAL,
            payout_email: payout_email,
            currency: Currency::USD,
            payout_item_ids: vec![],
            approved_by_ids: vec![approver_id],
            payout_batch_id: None,
            details: None,
            paid_to_payment_method_id: paid_to_payment_method_id,
        }
    }

    pub fn add_amount(mut self, amount: i32) -> Self {
        self.amount += amount;
        self
    }

    pub fn set_payout_status(mut self, payout_status: PayoutStatus) -> Self {
        self.payout_status = payout_status;
        self
    }

    pub fn set_payout_email<S: ToString>(mut self, payout_email: S) -> Self {
        self.payout_email = payout_email.to_string();
        self
    }

    pub fn set_currency(mut self, currency: Currency) -> Self {
        self.currency = currency;
        self
    }

    pub fn set_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn set_payout_batch_id(mut self, payout_batch_id: String) -> Self {
        self.payout_batch_id = Some(payout_batch_id);
        self
    }

    pub fn set_payout_item_ids(mut self, payout_item_ids: Vec<String>) -> Self {
        self.payout_item_ids = payout_item_ids;
        self
    }

    pub fn append_payout_item_id(mut self, payout_item_id: String) -> Self {
        self.payout_item_ids.push(payout_item_id);
        self
    }

    pub fn append_approved_by_id(mut self, user_id: String) -> Self {
        self.approved_by_ids.push(user_id);
        self
    }

    pub fn set_paid_to_payment_method_id(mut self, payment_method_id: String) -> Self {
        self.paid_to_payment_method_id = Some(payment_method_id);
        self
    }

    fn finish(self) {}
}



pub fn create_payout_emails_hashmap(
    payout_methods: Vec<PayoutMethod>
) -> HashMap<PayeeId, PayoutEmail> {

    let mut payout_emails: HashMap<PayeeId, PayoutEmail> = HashMap::new();

    for pm in payout_methods.iter() {

        match pm.payout_type {
            None => {},
            Some(PayoutType::BANK) => unimplemented!(),
            Some(PayoutType::PAYPAL) => {
                match pm.payout_email.as_ref() {
                    Some(payout_email) => {
                        payout_emails.insert(
                            pm.payee_id.clone(),
                            payout_email.to_string()
                        );
                    },
                    None => {
                        debug!("No payout_email for user: {}", pm.payee_id);
                    },
                }
            }
        }

    }
    payout_emails
}

pub fn create_payout_method_hashmap(
    payout_methods: Vec<PayoutMethod>
) -> HashMap<PayeeId, PayoutMethod> {

    let mut pms: HashMap<PayeeId, PayoutMethod> = HashMap::new();

    for pm in payout_methods.iter() {
        pms.insert(
            pm.payee_id.clone(),
            pm.clone()
        );
    }
    pms
}



pub fn aggregate_payout_totals_by_payee_id(
    payout_period: PayoutPeriod,
    payout_items: Vec<PayoutItem>,
    payout_emails: HashMap<PayeeId, PayoutEmail>,
    payout_methods: HashMap<PayeeId, PayoutMethod>,
    approver_id: String,
) -> HashMap<PayeeId, Payout> {

    let hmap: HashMap<PayeeId, Payout> = HashMap::new();

    // 1. group PayoutItems by storeId
    // NOTE: only consecutive elements are assigned to the same group.
    // Therefore, sort by payeeId (storeId) first.
    payout_items.into_iter()
        .sorted_by_key(|pitem: &PayoutItem| pitem.payee_id.clone())
        .group_by(|pitem| pitem.payee_id.clone())
        .into_iter()
        .fold(hmap, |mut hmap_acc, (payee_id, pitem_group)| {

            debug!("------------ Aggregating Payout -------------");
            let payout_email = payout_emails.get(&payee_id)
                .unwrap_or(&String::from("")).to_string();

            let paid_to_payment_method_id = match payout_methods.get(&payee_id) {
                Some(pm) => Some(pm.id.clone()),
                None => None
            };

            // 2. Create a Payout for each StoreId/PayeeId
            let payout = Payout::new(
                payee_id.clone(),
                payout_period.clone(),
                approver_id.clone(),
                payout_email.clone(),
                paid_to_payment_method_id.clone(),
            );

            // 3. For each group, fold/aggregate subtotals into Payout,
            // then insert to HashMap
            hmap_acc.insert(
                payee_id,
                pitem_group.into_iter().fold(
                    payout, // initial accumulator value
                    |accumulator: Payout, pitem: PayoutItem| {
                        debug!("id: {:?}", &pitem.id);
                        debug!("payeeId: {:?}", &pitem.payee_id);
                        debug!("amount: {:?}\n", &pitem.amount);
                        accumulator
                            .add_amount(pitem.amount.clone())
                            .append_payout_item_id(pitem.id.clone())
                    })
            );

            hmap_acc
        })
}



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(AsExpression, FromSqlRow)]
#[sql_type = "Text"]
pub enum PayoutStatus {
    // payout states
    UNPAID,
    MISSING_PAYOUT_METHOD,
    PENDING_APPROVAL,
    PROCESSING,
    PAID,
    RETAINED,
    // refund states
    REFUNDING,
    PENDING_REFUND,
    REFUNDED,
}
impl PayoutStatus {
    pub fn as_string(&self) -> String {
        String::from(format!("{:?}", &self))
    }
}
impl Default for PayoutStatus {
    fn default() -> Self {
        PayoutStatus::UNPAID
    }
}
impl ToSql<Text, Pg> for PayoutStatus {
    fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> ::diesel::serialize::Result {
        let stance = self.as_string();
        ToSql::<Text, Pg>::to_sql(&stance, out)
    }
}
impl FromSql<Text, Pg> for PayoutStatus {
    fn from_sql(maybe_bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let pstatus = <String as FromSql<Text, Pg>>::from_sql(maybe_bytes)
            .expect("Error parsing PayoutStatus: <String as FromSql<Text, Pg>>");
        Ok(PayoutStatus::from_str(&pstatus)?)
    }
}
impl FromStr for PayoutStatus {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pstatus = match s.trim() {
            "UNPAID" => PayoutStatus::UNPAID,
            "MISSING_PAYOUT_METHOD" => PayoutStatus::MISSING_PAYOUT_METHOD,
            "PENDING_APPROVAL" => PayoutStatus::PENDING_APPROVAL,
            "PROCESSING" => PayoutStatus::PROCESSING,
            "RETAINED" => PayoutStatus::RETAINED,
            "PAID" => PayoutStatus::PAID,
            "REFUNDING" => PayoutStatus::REFUNDING,
            "PENDING_REFUND" => PayoutStatus::PENDING_REFUND,
            "REFUNDED" => PayoutStatus::REFUNDED,
            _ => panic!("PayoutStatus from Pg does not match any known enum variant!"),
        };
        Ok(pstatus)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[derive(AsExpression, FromSqlRow)]
#[sql_type = "Text"]
pub enum PayeeType {
    STORE,
    PLATFORM,
    BUYER_AFFILIATE,
    SELLER_AFFILIATE,
}
impl PayeeType {
    pub fn as_string(&self) -> String {
        String::from(format!("{:?}", &self))
    }
}
impl Default for PayeeType {
    fn default() -> Self {
        PayeeType::STORE
    }
}
impl ToSql<Text, Pg> for PayeeType {
    fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> ::diesel::serialize::Result {
        let stance = self.as_string();
        ToSql::<Text, Pg>::to_sql(&stance, out)
    }
}
impl FromSql<Text, Pg> for PayeeType {
    fn from_sql(maybe_bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let pstatus = <String as FromSql<Text, Pg>>::from_sql(maybe_bytes)
            .expect("Error parsing PayeeType: <String as FromSql<Text, Pg>>");
        Ok(PayeeType::from_str(&pstatus)?)
    }
}
impl FromStr for PayeeType {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pstatus = match s.trim() {
            "STORE" | "store" => PayeeType::STORE,
            "BUYER_AFFILIATE" | "buyer_affiliate" => PayeeType::BUYER_AFFILIATE,
            "SELLER_AFFILIATE" | "seller_affiliate" => PayeeType::SELLER_AFFILIATE,
            "PLATFORM" | "platform" => PayeeType::PLATFORM,
            _ => panic!("PayeeStype from Pg does not match any known enum variant!"),
        };
        Ok(pstatus)
    }
}


#[derive(Clone, Debug, Serialize, QueryableByName)]
pub struct PayoutAggregates {
    #[sql_type = "Timestamp"]
    pub created_at: chrono::NaiveDateTime,
    #[sql_type = "Nullable<Timestamp>"]
    pub payout_date: Option<chrono::NaiveDateTime>,
    #[sql_type = "BigInt"]
    pub amount_total: i64,
    #[sql_type = "BigInt"]
    pub count: i64,
}

impl PayoutAggregates {
    pub fn new() -> Self {
        Self {
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            payout_date: None,
            amount_total: 0,
            count: 0,
        }
    }
}

impl std::default::Default for PayoutAggregates {
    fn default() -> Self {
        Self {
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            payout_date: None,
            amount_total: 0,
            count: 0,
        }
    }
}



