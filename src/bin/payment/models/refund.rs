// Needed for diesel table schemas
use diesel::prelude::*;
use gm::db::schema::refunds;


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(Queryable, Insertable)]
#[table_name = "refunds"]
pub struct Refund {
    pub id: String,
    pub transaction_id: String,
    pub order_id: String,
    pub order_item_ids: Option<Vec<String>>,
    pub created_at: chrono::NaiveDateTime,
    pub reason: Option<String>,
    pub reason_details: Option<String>,
}

impl Refund {
    fn new(
        id: String,
        transaction_id: String,
        order_id: String,
        order_item_ids: Option<Vec<String>>,
        created_at: chrono::NaiveDateTime,
        reason: Option<String>,
        reason_details: Option<String>,
    ) -> Self {
        Self {
            id: id,
            transaction_id: transaction_id,
            order_id: order_id,
            order_item_ids: order_item_ids,
            created_at: created_at,
            reason: reason,
            reason_details: reason_details,
        }
    }

    pub fn update_order_item_ids(mut self, order_item_ids: Vec<String>) -> Self {
        self.order_item_ids = Some(order_item_ids);
        self
    }

    pub fn update_order_id(mut self, order_id: String) -> Self {
        self.order_id = order_id;
        self
    }

    pub fn update_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn update_reason_details(mut self, reason_details: String) -> Self {
        self.reason_details = Some(reason_details);
        self
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RefundReason {
    Duplicate,
    Fraudulent,
    RequestedByCustomer,
}

impl RefundReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            RefundReason::Duplicate => "duplicate",
            RefundReason::Fraudulent => "fraudulent",
            RefundReason::RequestedByCustomer => "requested_by_customer",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "duplicate" => RefundReason::Duplicate,
            "fraudulent" | "fraud" => RefundReason::Fraudulent,
            "requested_by_customer" => RefundReason::RequestedByCustomer,
            _ => RefundReason::RequestedByCustomer,
        }
    }
}
