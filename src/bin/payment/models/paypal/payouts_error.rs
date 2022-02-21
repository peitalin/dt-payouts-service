

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalErrorResponse {
    pub name: Option<String>,
    pub message: Option<String>,
    pub debug_id: Option<String>,
    pub information_link: Option<String>,
    pub details: Option<Vec<PaypalPayoutErrorDetails>>,
    // Ignore this for now until we see what the link object looks like
    pub links: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayoutErrorDetails {
    pub field: Option<String>,
    pub location: Option<String>,
    pub issue: Option<String>,
}

#[test]
fn deserializes_paypal_get_payout_item_cancel() {

    let test_str = r#"
    {
        "name":"VALIDATION_ERROR",
        "message":"Invalid request - see details",
        "debug_id":"c2e05372cf151",
        "information_link":"https://developer.paypal.com/docs/api/payments.payouts-batch/#errors",
        "details": [
            {
                "field":"items[0].receiver",
                "location":"body",
                "issue":"Receiver is invalid or does not match with type"
            }
        ],
        "links": []
    }
    "#;

    let res = serde_json::from_str::<PaypalErrorResponse>(test_str);
    match res {
        Ok(paypal_error) => {
            assert_eq!(
                paypal_error.name,
                Some(String::from("VALIDATION_ERROR"))
            );
        },
        Err(e) => panic!(e.to_string()),
    }
}


#[test]
fn deserializes_paypal_payout_insufficient_funds() {

    let test_str = r#"
        {
            "name":"INSUFFICIENT_FUNDS",
            "message":"Sender does not have sufficient funds. Please add funds and retry.",
            "debug_id":"65a015e0b529d",
            "information_link":"https://developer.paypal.com/docs/api/payments.payouts-batch/#errors",
            "links":[]
        }
    "#;

    let res = serde_json::from_str::<PaypalErrorResponse>(test_str);
    match res {
        Ok(paypal_error) => {
            assert_eq!(
                paypal_error.name,
                Some(String::from("INSUFFICIENT_FUNDS"))
            );
        },
        Err(e) => panic!(e.to_string()),
    }
}


#[test]
fn deserializes_paypal_auth_error() {

    let test_str = r#"
        {
            "error":"invalid_client",
            "error_description":"Client Authentication failed"
        }
    "#;

    let res = serde_json::from_str::<PaypalErrorResponse>(test_str);
    match res {
        Ok(paypal_error) => {
            assert_eq!(
                paypal_error.error,
                Some(String::from("invalid_client"))
            );
        },
        Err(e) => panic!(e.to_string()),
    }
}

#[test]
fn deserializes_paypal_refund_duplicate_transaction() {

    let test_str = r#"
        {
            "name":"DUPLICATE_TRANSACTION",
            "message":"Requested invoice number was already used.",
            "information_link":"https://developer.paypal.com/docs/api/payments/#errors",
            "debug_id":"dd73d19a07ff4"
        }
    "#;

    let res = serde_json::from_str::<PaypalErrorResponse>(test_str);
    match res {
        Ok(item) => {
            assert_eq!(
                item.name,
                Some(String::from("DUPLICATE_TRANSACTION"))
            );
        },
        Err(e) => panic!(e.to_string()),
    }
}
