
use crate::models::paypal::{
    PaypalPaidoutItemDetails,
};


#[test]
fn deserializes_paypal_get_payout_item_cancel() {

    let test_str = r#"
    {
        "payout_item_id": "5KUDKLF8SDC7S",
        "transaction_id": "1DG93452WK758815H",
        "activity_id": "0E158638XS0329101",
        "transaction_status": "RETURNED",
        "payout_item_fee": {
            "currency": "USD",
            "value": "0.35"
        },
        "payout_batch_id": "CQMWKDQF5GFLL",
        "sender_batch_id": "Payouts_2018_100006",
        "payout_item": {
            "recipient_type": "EMAIL",
            "amount": {
                "value": "9.87",
                "currency": "USD"
            },
            "note": "Thanks for your patronage!",
            "receiver": "receiver@example.com",
            "sender_item_id": "14Feb_234"
        },
        "time_processed": "2018-01-27T10:17:41Z",
        "errors": {
            "name": "RECEIVER_UNREGISTERED",
            "message": "Receiver is unregistered",
            "information_link": "https://developer.paypal.com/docs/api/payments.payouts-batch#errors"
        },
        "links": [
            {
                "rel": "self",
                "href": "https://api.sandbox.paypal.com/v1/payments/payouts-item/5KUDKLF8SDC7S",
                "method": "GET"
            },
            {
                "rel": "batch",
                "href": "https://api.sandbox.paypal.com/v1/payments/payouts/CQMWKDQF5GFLL",
                "method": "GET"
            }
        ]
    }
    "#;

    let res = serde_json::from_str::<PaypalPaidoutItemDetails>(test_str);
    match res {
        Ok(item) => {
            assert_eq!(
                item.payout_item_id,
                String::from("5KUDKLF8SDC7S")
            );
        },
        Err(e) => panic!(e.to_string()),
    }
}
