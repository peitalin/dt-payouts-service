use diesel::prelude::*;
use diesel::sql_types::{Double, Float8, Jsonb, Json, Text, BigInt, Timestamp, Nullable};
use diesel::serialize::{Output, ToSql};
use diesel::pg::Pg;
use diesel::deserialize::FromSql;
use std::str::FromStr;

use gm::db::schema::payout_splits;
use chrono::prelude::Utc;


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[table_name = "payout_splits"]
pub struct PayoutSplit {
    #[sql_type = "Text"]
    pub id: String,
    #[sql_type = "Timestamp"]
    pub created_at: chrono::NaiveDateTime,
    #[sql_type = "Text"]
    pub store_or_user_id: String,
    #[sql_type = "Text"]
    pub deal_type: PayoutDealType,
    #[sql_type = "Nullable<Timestamp>"]
    pub expires_at: Option<chrono::NaiveDateTime>,
    #[sql_type = "Float8"]
    pub rate: f64,
    /// referrer_id: This is the ID of the PayoutSplit of the Referrer
    /// A referrer may have multiple payoutsplits (SELLER_AFFILIATE, BUYER_AFFILIATE
    /// SELLER), this referred_id references a specific PayoutSplit for converting
    /// another REFERRED_SELLER.
    #[sql_type = "Nullable<Text>"]
    pub referrer_id: Option<String>,
}

impl PayoutSplit {
    pub fn new(
        store_or_user_id: String,
        deal_type: PayoutDealType,
        expires_at: Option<chrono::NaiveDateTime>,
        rate: f64,
        referrer_id: Option<String>,
    ) -> Self {

        let created_at = chrono::Utc::now();

        PayoutSplit {
            id: format!("psplit_{}", uuid::Uuid::new_v4().to_string()),
            created_at: chrono::NaiveDateTime::from_timestamp(
                created_at.timestamp(), 0
            ),
            store_or_user_id: store_or_user_id,
            deal_type: deal_type,
            expires_at: expires_at,
            rate: rate,
            referrer_id: referrer_id,
        }
    }

    pub fn update_referrer_id(mut self, referrer_id: String) -> Self {
        self.referrer_id = Some(referrer_id);
        self
    }

    pub fn update_deal_type(mut self, deal_type: PayoutDealType) -> Self {
        self.deal_type = deal_type;
        self
    }
}

impl FromSql<Json, Pg> for PayoutSplit {
    fn from_sql(maybe_bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<Json, Pg>>::from_sql(maybe_bytes)?;
        serde_json::from_value::<PayoutSplit>(value).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(AsExpression, FromSqlRow)]
#[sql_type = "Text"]
pub enum PayoutDealType {
    /// What a normal seller receives. Sellers without an entry default to platform default of 15%.
    SELLER,
    /// someone who refers another seller earns this rate
    SELLER_AFFILIATE,
    /// seller referred by an affiliate earns this rate
    REFERRED_SELLER,
    /// Deal given to someone who shared their ref link to the site
    BUYER_AFFILIATE,
}
impl PayoutDealType {
    pub fn as_string(&self) -> String {
        String::from(format!("{:?}", &self))
    }
}
impl Default for PayoutDealType {
    fn default() -> Self {
        PayoutDealType::SELLER
    }
}
impl ToSql<Text, Pg> for PayoutDealType {
    fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> ::diesel::serialize::Result {
        let stance = self.as_string();
        ToSql::<Text, Pg>::to_sql(&stance, out)
    }
}
impl FromSql<Text, Pg> for PayoutDealType {
    fn from_sql(maybe_bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let pdeal = <String as FromSql<Text, Pg>>::from_sql(maybe_bytes)
            .expect("Error parsing PayoutDealType: <String as FromSql<Text, Pg>>");
        Ok(PayoutDealType::from_str(&pdeal)?)
    }
}
impl FromStr for PayoutDealType {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pdeal = match s.trim() {
            "SELLER" | "seller" => PayoutDealType::SELLER,
            "SELLER_AFFILIATE" | "seller_affiliate" => PayoutDealType::SELLER_AFFILIATE,
            "REFERRED_SELLER" | "referred_seller" => PayoutDealType::REFERRED_SELLER,
            "BUYER_AFFILIATE" | "buyer_affiliate" => PayoutDealType::BUYER_AFFILIATE,
            _ => panic!("PayoutDealType from Pg does not match any known enum variant!"),
        };
        Ok(pdeal)
    }
}

