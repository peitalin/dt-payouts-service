// Needed for diesel table schemas
use diesel::prelude::*;
use gm::db::schema::payment_methods;
use gm::db::schema::payment_method_addresses;
use gm::utils::dates::from_timestamp_s_to_naivedatetime;
use gm::utils::dates::from_datetimestr_to_option_naivedatetime;


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Queryable, Insertable)]
#[table_name = "payment_methods"]
pub struct PaymentMethodDb {
    pub id: String,
    pub user_id: String,
    #[serde(deserialize_with = "from_timestamp_s_to_naivedatetime")]
    pub created_at: chrono::NaiveDateTime,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub customer_id: Option<String>,
    pub payment_processor: Option<String>,
    pub payment_method_types: Option<Vec<String>>,
    pub last4: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub details: Option<String>,
}

impl PaymentMethodDb {
    pub fn update_last4(mut self, last4: String) -> Self {
        self.last4 = Some(last4);
        self
    }
    pub fn update_exp_month(mut self, exp_month: i32) -> Self {
        self.exp_month = Some(exp_month);
        self
    }
    pub fn update_exp_year(mut self, exp_year: i32) -> Self {
        self.exp_year = Some(exp_year);
        self
    }
    pub fn update_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }
    pub fn update_email(mut self, email: Option<String>) -> Self {
        self.email = email;
        self
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Queryable, Insertable)]
#[table_name = "payment_method_addresses"]
pub struct PaymentMethodAddress {
    pub payment_method_id: String,
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub town: Option<String>,
}