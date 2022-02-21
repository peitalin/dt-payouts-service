use crate::models::paypal::PaypalLink;
use gm::utils::dates::from_datetimestr_to_naivedatetime;
use gm::utils::deserialize_as_f64;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaypalRefundResponse {
    pub id: String,
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub create_time: chrono::NaiveDateTime,
    pub resource_type: String,
    pub event_type: String,
    pub summary: String,
    pub event_version: Option<String>,
    pub links: Option<Vec<PaypalLink>>,
    pub resource: PaypalResource
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaypalResource {
    pub id: String,
    pub state: String,
    pub sale_id: String,
    pub parent_payment: String,
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub update_time: chrono::NaiveDateTime,
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub create_time: chrono::NaiveDateTime,
    pub amount: PaypalAmount,
    pub links: Option<Vec<PaypalLink>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaypalAmount {
    #[serde(deserialize_with = "deserialize_as_f64")]
    pub total: f64,
    pub currency: String,
}

mod tests {
    use super::*;

    #[test]
    fn deserializes_paypal_refund_webhook_response() {

        let test_paypal_refund_response: &str = r#"
        {
            "id": "WH-2N242548W9943490U-1JU23391CS4765624",
            "create_time": "2014-10-31T15:42:24Z",
            "resource_type": "sale",
            "event_type": "PAYMENT.SALE.REFUNDED",
            "summary": "A 0.01 USD sale payment was refunded",
            "resource": {
                "sale_id": "9T0916710M1105906",
                "parent_payment": "PAY-5437236047802405NKRJ22UA",
                "update_time": "2014-10-31T15:41:51Z",
                "amount": {
                    "total": "-0.01",
                    "currency": "USD"
                },
                "create_time": "2014-10-31T15:41:51Z",
                "links": [
                {
                    "href": "https://api.paypal.com/v1/payments/refund/6YX43824R4443062K",
                    "rel": "self",
                    "method": "GET"
                },
                {
                    "href": "https://api.paypal.com/v1/payments/payment/PAY-5437236047802405NKRJ22UA",
                    "rel": "parent_payment",
                    "method": "GET"
                },
                {
                    "href": "https://api.paypal.com/v1/payments/sale/9T0916710M1105906",
                    "rel": "sale",
                    "method": "GET"
                }
                ],
                "id": "6YX43824R4443062K",
                "state": "completed"
            },
            "links": [
                {
                    "href": "https://api.paypal.com/v1/notifications/webhooks-events/WH-2N242548W9943490U-1JU23391CS4765624",
                    "rel": "self",
                    "method": "GET",
                    "encType": "application/json"
                },
                {
                    "href": "https://api.paypal.com/v1/notifications/webhooks-events/WH-2N242548W9943490U-1JU23391CS4765624/resend",
                    "rel": "resend",
                    "method": "POST",
                    "encType": "application/json"
                }
            ],
            "event_version": "1.0"
        }
        "#;

        let t2: PaypalRefundResponse = serde_json::from_str::<PaypalRefundResponse>(
            &test_paypal_refund_response
        ).expect("Err deserializing PaypalRefundResponse");

        assert_eq!(
            String::from("WH-2N242548W9943490U-1JU23391CS4765624"),
            t2.id
        );
        assert_eq!(
            String::from("9T0916710M1105906"),
            t2.resource.sale_id
        );
        assert_eq!(
            String::from("6YX43824R4443062K"),
            t2.resource.id
        );
        assert_eq!(
            -0.01,
            t2.resource.amount.total
        );
    }
}