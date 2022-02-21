// Needed for diesel table schemas
use diesel::prelude::*;
use gm::db::schema::transactions;
// Date deserialization
use gm::utils::dates::from_datetimestr_to_naivedatetime;
use chrono::Utc;


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartRpc {
    pub id: String,
    pub user_id: Option<String>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub items: Option<Vec<String>>,
    pub applied_discount_codes: Option<Vec<String>>,
    pub subtotal: i32,
    pub taxes: i32,
    pub payment_processing_fee: i32,
    pub total: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartDb {
    pub id: String,
    pub user_id: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub items: Option<Vec<String>>,
    pub applied_discount_codes: Option<Vec<String>>,
    pub subtotal: Option<i32>,
    pub taxes: Option<i32>,
    pub fees: Option<i32>,
    pub total: Option<i32>,
}
