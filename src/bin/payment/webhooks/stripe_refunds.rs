use actix_web::{HttpResponse, HttpRequest, Error, web::Query};
use gm::models::stripe::{ Charge };
// use gm::models::stripe::ids::{ChargeId};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StripeRefundResponse {
    pub id: String,
    pub created: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub livemode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<StripeRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_webhooks: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    pub data: StripeRefundData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StripeRefundData {
    pub object: Charge,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StripeRequest {
    pub id: Option<String>,
    pub idempotency_key: Option<String>,
}



mod tests {
    use super::*;

    #[test]
    fn deserializes_stripe_refund_response_test() {

        let test_stripe_refund_response: &str = r#"
        {
            "created": 1326853478,
            "livemode": false,
            "id": "evt_00000000000000",
            "type": "charge.refunded",
            "object": "event",
            "request": null,
            "pending_webhooks": 1,
            "api_version": "2019-09-09",
            "data": {
                "object": {
                    "id": "ch_00000000000000",
                    "object": "charge",
                    "amount": 100,
                    "amount_refunded": 100,
                    "application": null,
                    "application_fee": null,
                    "application_fee_amount": null,
                    "balance_transaction": "txn_00000000000000",
                    "billing_details": {
                        "address": {
                            "city": null,
                            "country": null,
                            "line1": null,
                            "line2": null,
                            "postal_code": null,
                            "state": null
                        },
                        "email": null,
                        "name": "Jenny Rosen",
                        "phone": null
                    },
                    "captured": false,
                    "created": 1570102473,
                    "currency": "aud",
                    "customer": null,
                    "description": "My First Test Charge (created for API docs)",
                    "destination": null,
                    "dispute": null,
                    "failure_code": null,
                    "failure_message": null,
                    "fraud_details": {
                    },
                    "invoice": null,
                    "livemode": false,
                    "metadata": {
                    },
                    "on_behalf_of": null,
                    "order": null,
                    "outcome": null,
                    "paid": true,
                    "payment_intent": null,
                    "payment_method": "card_00000000000000",
                    "payment_method_details": {
                        "card": {
                            "brand": "visa",
                            "checks": {
                                "address_line1_check": null,
                                "address_postal_code_check": null,
                                "cvc_check": null
                            },
                            "country": "US",
                            "exp_month": 8,
                            "exp_year": 2020,
                            "fingerprint": "NkKgdxB8KrBtIsky",
                            "funding": "credit",
                            "installments": null,
                            "last4": "4242",
                            "three_d_secure": null,
                            "wallet": null
                        },
                        "type": "card"
                    },
                    "receipt_email": null,
                    "receipt_number": null,
                    "receipt_url": "https://pay.stripe.com/receipts/acct_1AE5YcKqy1M9WH1D/ch_1FPSZpKqy1M9WH1DCkPFxdee/rcpt_FvJCKo7WMVoccBBUCxejbzIBAc4dldh",
                    "refunded": true,
                    "refunds": {
                        "object": "list",
                        "data": [
                            {
                                "id": "re_00000000000000",
                                "object": "refund",
                                "amount": 100,
                                "balance_transaction": "txn_00000000000000",
                                "charge": "ch_00000000000000",
                                "created": 1570102473,
                                "currency": "aud",
                                "metadata": {
                                },
                                "reason": null,
                                "receipt_number": "1634-7954",
                                "source_transfer_reversal": null,
                                "status": "succeeded",
                                "transfer_reversal": null
                            }
                        ],
                        "has_more": false,
                        "total_count": 0,
                        "url": "/v1/charges/ch_1FPSZpKqy1M9WH1DCkPFxdee/refunds"
                    },
                    "review": null,
                    "shipping": null,
                    "source": {
                        "id": "card_00000000000000",
                        "object": "card",
                        "address_city": null,
                        "address_country": null,
                        "address_line1": null,
                        "address_line1_check": null,
                        "address_line2": null,
                        "address_state": null,
                        "address_zip": null,
                        "address_zip_check": null,
                        "brand": "Visa",
                        "country": "US",
                        "customer": null,
                        "cvc_check": null,
                        "dynamic_last4": null,
                        "exp_month": 8,
                        "exp_year": 2020,
                        "fingerprint": "NkKgdxB8KrBtIsky",
                        "funding": "credit",
                        "last4": "4242",
                        "metadata": {
                        },
                        "name": "Jenny Rosen",
                        "tokenization_method": null
                    },
                    "source_transfer": null,
                    "statement_descriptor": null,
                    "statement_descriptor_suffix": null,
                    "status": "succeeded",
                    "transfer_data": null,
                    "transfer_group": null,
                    "fee": 0
                }
            }
        }
        "#;

        let t2: StripeRefundResponse = serde_json::from_str::<StripeRefundResponse>(
            &test_stripe_refund_response
        ).expect("Err deserializing StripeRefundResponse");

        assert_eq!(
            String::from("evt_00000000000000"),
            t2.id
        );
        // assert_eq!(
        //     ChargeId.from_str("ch_00000000000000"),
        //     t2.data.object.id
        // );
        assert_eq!(
            String::from("Jenny Rosen"),
            t2.data.object.billing_details.name.unwrap()
        );
        assert_eq!(
            2020,
            t2.data.object
                .payment_method_details.expect("missing payment_method_details")
                .card.expect("no card on payment_method_details")
                .exp_year
        );
    }



    #[test]
    fn deserializes_stripe_refund_response_actual() {

        let test_stripe_refund_response: &str = r#"
        {
            "id": "evt_1FPjt1Kqy1M9WH1DeJzMNIup",
            "object": "event",
            "api_version": "2019-09-09",
            "created": 1570169011,
            "data": {
                "object": {
                "id": "ch_1FPGtBKqy1M9WH1DnKpv2JTx",
                "object": "charge",
                "amount": 1540,
                "amount_refunded": 400,
                "application": null,
                "application_fee": null,
                "application_fee_amount": null,
                "balance_transaction": "txn_1FPGtBKqy1M9WH1D3Ve1LiVY",
                "billing_details": {
                    "address": {
                        "city": null,
                        "country": null,
                        "line1": null,
                        "line2": null,
                        "postal_code": "54545",
                        "state": null
                    },
                    "email": "jade@paid.com",
                    "name": null,
                    "phone": null
                },
                "captured": true,
                "created": 1570057545,
                "currency": "usd",
                "customer": null,
                "description": null,
                "destination": null,
                "dispute": null,
                "failure_code": null,
                "failure_message": null,
                "fraud_details": {
                },
                "invoice": null,
                "livemode": false,
                "metadata": {
                },
                "on_behalf_of": null,
                "order": null,
                "outcome": {
                    "network_status": "approved_by_network",
                    "reason": null,
                    "risk_level": "normal",
                    "risk_score": 44,
                    "seller_message": "Payment complete.",
                    "type": "authorized"
                },
                "paid": true,
                "payment_intent": "pi_1FPGtAKqy1M9WH1D1xN2D51B",
                "payment_method": "pm_1FPGtAKqy1M9WH1DHnuly5Kl",
                "payment_method_details": {
                    "card": {
                    "brand": "mastercard",
                    "checks": {
                        "address_line1_check": null,
                        "address_postal_code_check": "pass",
                        "cvc_check": "pass"
                    },
                    "country": "US",
                    "exp_month": 5,
                    "exp_year": 2045,
                    "fingerprint": "0COpwuInl6rPIDl8",
                    "funding": "unknown",
                    "installments": null,
                    "last4": "5454",
                    "three_d_secure": null,
                    "wallet": null
                    },
                    "type": "card"
                },
                "receipt_email": "jade@paid.com",
                "receipt_number": null,
                "receipt_url": "https://pay.stripe.com/receipts/acct_1AE5YcKqy1M9WH1D/ch_1FPGtBKqy1M9WH1DnKpv2JTx/rcpt_Fv777NtOZb0LZedJTGCBnlXicOQv1Om",
                "refunded": false,
                "refunds": {
                    "object": "list",
                    "data": [
                    {
                        "id": "re_1FPjt0Kqy1M9WH1DIW1eVrF6",
                        "object": "refund",
                        "amount": 400,
                        "balance_transaction": "txn_1FPjt0Kqy1M9WH1DRWLEjKG8",
                        "charge": "ch_1FPGtBKqy1M9WH1DnKpv2JTx",
                        "created": 1570169010,
                        "currency": "usd",
                        "metadata": {
                        },
                        "reason": "requested_by_customer",
                        "receipt_number": null,
                        "source_transfer_reversal": null,
                        "status": "succeeded",
                        "transfer_reversal": null
                    }
                    ],
                    "has_more": false,
                    "total_count": 1,
                    "url": "/v1/charges/ch_1FPGtBKqy1M9WH1DnKpv2JTx/refunds"
                },
                "review": null,
                "shipping": null,
                "source": null,
                "source_transfer": null,
                "statement_descriptor": null,
                "statement_descriptor_suffix": null,
                "status": "succeeded",
                "transfer_data": null,
                "transfer_group": null
                },
                "previous_attributes": {
                }
            },
            "livemode": false,
            "pending_webhooks": 1,
            "request": {
                "id": "req_9PzHCtUl3wVRdA",
                "idempotency_key": null
            },
            "type": "charge.refunded"
        }
        "#;

        let t2: StripeRefundResponse = match serde_json::from_str::<StripeRefundResponse>(
            &test_stripe_refund_response
        ) {
            Ok(res) => res,
            Err(e) => {
                debug!("error: {:?}", e);
                panic!("wtf: {:?}", e)
            }
        };

        assert_eq!(
            String::from("jade@paid.com"),
            t2.data.object.billing_details.email.unwrap()
        );

    }
}