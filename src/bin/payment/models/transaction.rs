// Needed for diesel table schemas
use diesel::prelude::*;
use diesel::sql_types::{Double, Integer, Text, BigInt, Timestamp};

use gm::db::schema::transactions;
use std::str::FromStr;

use crate::models::{ PaypalResponse, Refund, Currency };
use crate::models::paypal::{
    PaypalPurchaseUnits,
    PaypalCaptures,
};
use gm::models::stripe;


#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Queryable, Insertable)]
#[table_name = "transactions"]
pub struct Transaction {
    pub id: String,
    pub subtotal: i32,
    pub taxes: i32,
    pub payment_processing_fee: i32,
    pub created_at: chrono::NaiveDateTime,
    pub currency: Option<Currency>,
    pub charge_id: Option<String>,
    pub customer_id: Option<String>,
    pub order_id: Option<String>,
    pub payment_processor: Option<String>,
    pub payment_method_id: Option<String>,
    pub payment_intent_id: Option<String>,
    pub refund_id: Option<String>,
    pub details: Option<String>,
}

impl Transaction {

    pub fn update_prices(
        mut self,
        subtotal: i32,
        taxes: i32,
        payment_processing_fee: i32,
    ) -> Self {
        // total = subtotal + taxes + payment_processing_fee
        // subtotal = seller_payment + platform_fee + affiliate_fee
        self.subtotal = subtotal;
        self.taxes = taxes;
        self.payment_processing_fee = payment_processing_fee;
        self
    }

    pub fn update_payment_processor(mut self, payment_processor: Option<String>) -> Self {
        self.payment_processor = payment_processor;
        self
    }

    pub fn update_details(mut self, details: Option<String>) -> Self {
        self.details = details;
        self
    }

    pub fn update_id(mut self, id: String) {
        self.id = id;
    }

    pub fn update_transaction_id(&mut self, transaction_id: String) {
        self.id = transaction_id;
    }

    pub fn update_order_id(&mut self, order_id: String) {
        self.order_id = Some(order_id);
    }

    pub fn update_charge_id(&mut self, charge_id: String) {
        self.charge_id = Some(charge_id);
    }

    pub fn update_refund_id(&mut self, refund_id: String) {
        self.refund_id = Some(refund_id);
    }

    pub fn update_customer_id(&mut self, customer_id: String) {
        self.customer_id = Some(customer_id);
    }

    pub fn update_with_paypal_response(&mut self, p: PaypalResponse) {

        let purchase_unit: PaypalPurchaseUnits = p.purchase_units
            .expect("purchase_units field missing")
            .into_iter().next()
            .expect("purchase_units[0] missing!");

        let captures: PaypalCaptures = purchase_unit.payments.captures
            .expect("puchase_unit.payments.captures missing!")
            .into_iter().next()
            .expect("purchase_unit.payments.captures[0] missing!");

        self.id = format!("txn_{}", p.id.clone());
        self.charge_id = Some(captures.id);
        self.created_at = p.create_time.unwrap_or(
            chrono::NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0)
        );
        self.currency = Currency::from_str(&purchase_unit.amount.currency_code).ok();
        self.payment_processor = Some(String::from("Paypal"));
        self.payment_method_id = p.payer.map(|y| y.payer_id);
    }
}

impl std::default::Default for Transaction {
    fn default() -> Self {
        Self {
            id: format!("txn_{}", uuid::Uuid::new_v4().to_string()),
            subtotal: 0,
            taxes: 0,
            payment_processing_fee: 0,
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            currency: Some(Currency::USD),
            charge_id: None,
            customer_id: None,
            order_id: None,
            payment_processor: None,
            payment_method_id: None,
            payment_intent_id: None,
            refund_id: None,
            details: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionId {
    pub tx_id: String
}

impl From<stripe::PaymentIntent> for Transaction {
    fn from(p: stripe::PaymentIntent) -> Self {
        Self {
            id: format!("temp_{}", p.id),
            charge_id: None,
            subtotal: 0,
            taxes: 0,
            payment_processing_fee: 0,
            currency: Currency::from_str(&p.currency.to_string()).ok(),
            created_at: chrono::NaiveDateTime::from_timestamp(p.created, 0),
            customer_id: p.customer,
            order_id: None,
            payment_processor: Some(String::from("Stripe")),
            payment_method_id: p.payment_method,
            payment_intent_id: Some(p.id.to_string()),
            refund_id: None,
            details: None,
        }
    }
}

impl From<PaypalResponse> for Transaction {
    fn from(p: PaypalResponse) -> Self {

        let purchase_unit: PaypalPurchaseUnits = p.purchase_units
            .expect("purchase_units field missing")
            .into_iter().next()
            .expect("purchase_units[0] missing!");

        let captures: PaypalCaptures = purchase_unit.payments.captures
            .expect("puchase_unit.payments.captures missing!")
            .into_iter().next()
            .expect("purchase_unit.payments.captures[0] missing!");

        let _amount: i32 = purchase_unit.amount.value
            .replace(".", "")
            .parse::<i32>()
            .expect("Err parsing purchase_unit.amount.value as i32");

        Self {
            id: format!("txn_{}", p.id.clone()),
            charge_id: Some(captures.id),
            subtotal: 0,
            taxes: 0,
            payment_processing_fee: 0,
            currency: Currency::from_str(&purchase_unit.amount.currency_code).ok(),
            customer_id: p.payer.clone().unwrap().email_address,
            order_id: None,
            created_at: p.create_time.unwrap_or(
                chrono::NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0)
            ),
            payment_processor: Some(String::from("Paypal")),
            payment_method_id: p.payer.map(|y| y.payer_id),
            payment_intent_id: None,
            refund_id: None,
            details: None,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxQuery {
    pub cart_id: String,
    pub payment_processor: Option<String>,
    pub details: Option<String>
}

#[derive(Clone, Debug, Serialize, QueryableByName)]
pub struct TransactionAggregates {
    #[sql_type = "Timestamp"]
    pub created_at: chrono::NaiveDateTime,
    #[sql_type = "BigInt"]
    pub subtotal_sum: i64,
    #[sql_type = "BigInt"]
    pub fees_total: i64,
    #[sql_type = "BigInt"]
    pub count: i64,
}

impl TransactionAggregates {
    pub fn new() -> Self {
        Self {
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            subtotal_sum: 0,
            fees_total: 0,
            count: 0,
        }
    }
}

impl std::default::Default for TransactionAggregates {
    fn default() -> Self {
        Self {
            created_at: chrono::NaiveDateTime::from_timestamp(
                chrono::Utc::now().timestamp(), 0
            ),
            subtotal_sum: 0,
            fees_total: 0,
            count: 0,
        }
    }
}