use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use crate::models::paypal::{
  PaypalLink,
  PaypalValue,
};
use crate::models::{
    Payout,
    Currency,
    PayoutStatus,
    PayeeType,
};
use std::collections::HashMap;


// https://developer.paypal.com/docs/api/payments.payouts-batch/v1/


///////////////////////////////////
/////// Paypal Payout Params //////
///////////////////////////////////

// items array (contains the payout_item object) required
// An array of individual payout items.
// Minimum length: 1.
// Maximum length: 15000.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayoutParams {
    pub sender_batch_header: PaypalSenderBatchHeader,
    pub items: Vec<PaypalPayout>,
}
impl PaypalPayoutParams {
    pub fn new(payouts: Option<Vec<Payout>>) -> Self {
        Self {
            sender_batch_header: PaypalSenderBatchHeader {
                sender_batch_id: format!("paypal_payout_{}", uuid::Uuid::new_v4()),
                email_subject: Some("Payout from Relay Marketplace".to_string()),
                email_message: Some(
                    "We’ve sent you a payout for your Relay earnings".to_string()
                ),
            },
            items: match payouts {
                None => vec![],
                Some(ps) => ps.iter()
                    .map(PaypalPayout::from)
                    .collect::<Vec<PaypalPayout>>(),
            },
        }
    }

    pub fn set_email_subject(mut self, subject: Option<String>) -> Self {
        if let Some(s) = subject {
            self.sender_batch_header = PaypalSenderBatchHeader {
                sender_batch_id: self.sender_batch_header.sender_batch_id,
                email_subject: Some(s),
                email_message: self.sender_batch_header.email_message,
            };
        }
        self
    }

    pub fn set_email_message(mut self, message: Option<String>) -> Self {
        if let Some(m) = message {
            self.sender_batch_header = PaypalSenderBatchHeader {
                sender_batch_id: self.sender_batch_header.sender_batch_id,
                email_subject: self.sender_batch_header.email_subject,
                email_message: Some(m),
            };
        }
        self
    }

    pub fn set_items(mut self, items: Vec<PaypalPayout>) -> Self {
        self.items = items;
        self
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayoutResponse {
    pub batch_header: PaypalBatchHeader,
    pub links: Vec<PaypalLink>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalBatchHeader {
    pub payout_batch_id: String,
    pub batch_status: String,
    pub sender_batch_header: PaypalSenderBatchHeader,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalSenderBatchHeader {
    pub sender_batch_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayoutNotificationMethod {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<PaypalPayoutPhone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayoutPhone {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub national_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayout {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_type: Option<PaypalRecipientType>, // PHONE | EMAIL | PAYPAL_ID
    pub amount: PaypalValue,
    pub sender_item_id: String,
    pub receiver: String,
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_notification_method: Option<PaypalPayoutNotificationMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaypalRecipientType {
    PHONE,
    EMAIL,
    PAYPAL_ID
}

impl std::fmt::Display for PaypalRecipientType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
}

impl From<&Payout> for PaypalPayout {
    fn from(p: &Payout) -> Self {
        Self {
            recipient_type: Some(PaypalRecipientType::EMAIL),
            amount: PaypalValue {
                value: format!("{:?}", (p.amount as f64) / 100.0),
                currency: p.currency.as_string().to_uppercase(),
            },
            sender_item_id: p.id.clone(),
            receiver: p.payout_email.clone(),
            note: Some(format!("payeeId: {}", p.payee_id)),
            alternate_notification_method: None,
        }
    }
}


// curl -v -X POST https://api.sandbox.paypal.com/v1/payments/payouts \
// -H "Content-Type: application/json" \
// -H "Authorization: Bearer A21AAGMCn3em-8PmXWjKMKWAUU6nnvMyFt46ajjlz1VkJWbN3NCJBAV_sRwqnROxyayGTxcVKZFEgG03VJFbDx-x94cTwY6tg" \
// -d '{
//   "sender_batch_header": {
//     "sender_batch_id": "Payouts_2018_ckememwkljdfoqijerj",
//     "email_subject": "You have a payout!",
//     "email_message": "You have received a payout! Thanks for using our service!"
//   },
//   "items": [
//     {
//       "recipient_type": "EMAIL",
//       "amount": {
//         "value": "9.87",
//         "currency": "USD"
//       },
//       "note": "Thanks for your patronage!",
//       "sender_item_id": "201403140001",
//       "receiver": "jade_p@gm.com",
//       "alternate_notification_method": {
//         "phone": {
//           "country_code": "61",
//           "national_number": "433641770"
//         }
//       }
//     },
//     {
//       "recipient_type": "EMAIL",
//       "amount": {
//         "value": "1.87",
//         "currency": "USD"
//       },
//       "note": "Thanks for your patronage!",
//       "sender_item_id": "201403140001",
//       "receiver": "nadia@gm.com",
//       "alternate_notification_method": {
//         "phone": {
//           "country_code": "61",
//           "national_number": "9999988888"
//         }
//       }
//     },
//     {
//       "recipient_type": "PHONE",
//       "amount": {
//         "value": "112.34",
//         "currency": "USD"
//       },
//       "note": "Thanks for your support!",
//       "sender_item_id": "201403140002",
//       "receiver": "91-734-234-1234"
//     },
//     {
//       "recipient_type": "PAYPAL_ID",
//       "amount": {
//         "value": "5.32",
//         "currency": "USD"
//       },
//       "note": "Thanks for your patronage!",
//       "sender_item_id": "201403140003",
//       "receiver": "G83JXTJ5EHCQ2"
//     }
//   ]
// }'

#[test]
fn deserializes_paypal_payout_params() {

    let test_p = Payout {
        id: uuid::Uuid::new_v4().to_string(),
        payee_id: String::from("store-id123123123"),
        payee_type: PayeeType::STORE,
        amount: 1234,
        created_at: None,
        start_period: None,
        end_period: None,
        payout_date: None,
        payout_status: PayoutStatus::PENDING_APPROVAL,
        payout_email: "jade@paypal.com".to_string(),
        currency: Currency::USD,
        payout_item_ids: vec!["pi_111".to_string(), "pi_222".to_string()],
        approved_by_ids: vec!["usr_111".to_string(), "usr_222".to_string()],
        payout_batch_id: None,
        details: None,
        paid_to_payment_method_id: None,
    };

    let test_pp = PaypalPayout::from(&test_p);
    let test_str = serde_json::to_string(&test_pp).unwrap();
    let json_res: PaypalPayout = serde_json::from_str::<PaypalPayout>(&test_str).unwrap();
    // PaypalPayout.items
    // println!("\n>>>>>>>>\n{}\n", test_str);
    assert_eq!(json_res.amount.value, "12.34".to_string());
    assert_eq!(json_res.amount.currency, "USD".to_string());

    let paypal_payout_params = PaypalPayoutParams::new(None)
        .set_email_message(Some("znayu".to_string()))
        .set_email_subject(Some("zaya".to_string()))
        .set_items(vec![json_res.clone()]);

    // PaypalPayoutParams
    println!("\n>>>>>>>>\n{:#?}\n", paypal_payout_params);
    assert_eq!(
        paypal_payout_params.sender_batch_header.email_subject,
        Some("zaya".to_string())
    );

}

#[test]
fn deserializes_paypal_payout_create_response() {

    let test_str = r#"
    {
        "batch_header": {
            "payout_batch_id": "9NEWQJY3AP8YY",
            "batch_status":"PENDING",
            "sender_batch_header": {
                "sender_batch_id":"Payouts_2018_ckememwkljdfopqijerj",
                "email_subject":"You have a payout!",
                "email_message":"You have received a payout! Thanks for using our service!"
            }
        },
        "links":[
            {
                "href":"https://api.sandbox.paypal.com/v1/payments/payouts/9NEWQJY3AP8YY",
                "rel":"self",
                "method":"GET",
                "encType":"application/json"
            }
        ]
    }
    "#;

    let res = serde_json::from_str::<PaypalPayoutResponse>(test_str);
    match res {
        Ok(batch_response) => assert_eq!(
          batch_response.batch_header.payout_batch_id,
          String::from("9NEWQJY3AP8YY")
        ),
        Err(e) => panic!(e.to_string()),
    }
}



//// The PayPal-generated payout status. If the payout passes preliminary checks, the status is PENDING. The possible values are:
/// DENIED. Your payout requests were denied, so they were not processed. Check the error messages to see any steps necessary to fix these issues.
/// PENDING. Your payout requests were received and will be processed soon.
/// PROCESSING. Your payout requests were received and are now being processed.
/// SUCCESS. Your payout batch was processed and completed. Check the status of each item for any holds or unclaimed transactions.
/// CANCELED. The payouts file that was uploaded through the PayPal portal was cancelled by the sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaypalBatchStatus {
    DENIED,
    PENDING,
    PROCESSING,
    SUCCESS,
    CANCELED,
}

/// The transaction status. The possible values are:
/// SUCCESS. Funds have been credited to the recipient’s account.
/// FAILED. This payout request has failed, so funds were not deducted from the sender’s account.
/// PENDING. Your payout request was received and will be processed.
/// UNCLAIMED. The recipient for this payout does not have a PayPal account. A link to sign up for a PayPal account was sent to the recipient. However, if the recipient does not claim this payout within 30 days, the funds are returned to your account.
/// RETURNED. The recipient has not claimed this payout, so the funds have been returned to your account.
/// ONHOLD. This payout request is being reviewed and is on hold.
/// BLOCKED. This payout request has been blocked.
/// REFUNDED. This payout request was refunded.
/// REVERSED. This payout request was reversed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaypalTransactionStatus {
    SUCCESS,
    FAILED,
    PENDING,
    UNCLAIMED,
    RETURNED,
    ONHOLD,
    BLOCKED,
    REFUNDED,
    REVERSED,
}