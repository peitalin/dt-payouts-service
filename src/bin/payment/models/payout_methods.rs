use diesel::prelude::*;
use gm::db::schema::payout_methods;
use gm::utils::dates::from_datetimestr_to_naivedatetime;
use gm::utils::dates::from_datetimestr_to_option_naivedatetime;

use diesel::deserialize::FromSql;
use diesel::result::Error::DeserializationError;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Text};
use std::str::FromStr;


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Queryable, Insertable)]
#[table_name = "payout_methods"]
pub struct PayoutMethod {
    pub id: String,
    pub payee_id: String,
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub created_at: chrono::NaiveDateTime,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub payout_type: Option<PayoutType>, // Paypal, Bank, Card
    pub payout_email: Option<String>, // paypal_email
    pub payout_processor: Option<String>, // Paypal, Adyen
    pub payout_processor_id: Option<String>, // some other payment ID
}

impl PayoutMethod {
    pub fn new(
        payee_id: String,
        payout_processor: Option<String>, // Paypal, Adyen
        payout_type: Option<PayoutType>, // Paypal, Bank, Card
        payout_email: Option<String>, // paypal_email
        payout_processor_id: Option<String>, // some other payment ID
    ) -> Self {
        Self {
            id: format!("payout_method_{}", uuid::Uuid::new_v4().to_string()),
            payee_id: payee_id,
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            updated_at: Some(chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            )),
            payout_type: payout_type,
            payout_email: payout_email,
            payout_processor: payout_processor,
            payout_processor_id: payout_processor_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(AsExpression, FromSqlRow)]
#[serde(rename_all = "UPPERCASE")]
#[sql_type = "Text"]
pub enum PayoutType {
    PAYPAL,
    BANK,
}
impl PayoutType {
    pub fn as_string(&self) -> String {
        String::from(format!("{:?}", &self))
    }
}
impl Default for PayoutType {
    fn default() -> Self {
        PayoutType::PAYPAL
    }
}
impl ToSql<Text, Pg> for PayoutType {
    fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> ::diesel::serialize::Result {
        let stance = self.as_string();
        ToSql::<Text, Pg>::to_sql(&stance, out)
    }
}
impl FromSql<Text, Pg> for PayoutType {
    fn from_sql(maybe_bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let pstatus = <String as FromSql<Text, Pg>>::from_sql(maybe_bytes)
            .expect("Error parsing PayoutType: <String as FromSql<Text, Pg>>");
        Ok(PayoutType::from_str(&pstatus)?)
    }
}
impl std::str::FromStr for PayoutType {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pstatus = match s.trim() {
            "PAYPAL" | "paypal" | "Paypal" | "PayPal" => PayoutType::PAYPAL,
            "BANK" | "bank" | "Bank" => PayoutType::BANK,
            _ => panic!("PayoutType string does not match any known enum variant!"),
        };
        Ok(pstatus)
    }
}

