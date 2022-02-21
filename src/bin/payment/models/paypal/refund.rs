use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use crate::models::paypal::PaypalLink;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalRefundResponse {
    pub id: String,
    pub invoice_id: Option<String>,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub create_time: Option<chrono::NaiveDateTime>,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub update_time: Option<chrono::NaiveDateTime>,
    pub state: Option<String>,
    pub amount: Option<PaypalTotal>,
    pub refund_from_transaction_fee: Option<PaypalValue>,
    pub total_refunded_amount: Option<PaypalValue>,
    pub refund_from_received_amount: Option<PaypalValue>,
    pub sale_id: Option<String>,
    pub status: Option<String>,
    pub invoice_number: Option<String>,
    pub links: Option<Vec<PaypalLink>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalTotal {
    pub total: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalValue {
    pub value: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalRefundDetails {
    pub id: String,
    pub amount: PaypalGrossAmount,
    pub seller_payable_breakdown: Option<PaypalSellerPayableBreakdown>,
    pub invoice_id: String,
    pub status: Option<String>,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub create_time: Option<chrono::NaiveDateTime>,
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub update_time: Option<chrono::NaiveDateTime>,
    pub links: Option<Vec<PaypalLink>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalSellerPayableBreakdown {
    pub gross_amount: PaypalGrossAmount,
    pub paypal_fee: PaypalGrossAmount,
    pub net_amount: PaypalGrossAmount,
    pub total_refunded_amount: PaypalGrossAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalGrossAmount {
    pub currency_code: Option<String>,
    pub value: String,
}


