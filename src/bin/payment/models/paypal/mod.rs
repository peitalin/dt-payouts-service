pub mod payouts_create;
pub mod payouts_cancel;
pub mod payouts_error;
pub mod payouts_get;
pub mod refund;

pub use payouts_create::*;
pub use payouts_cancel::*;
pub use payouts_error::*;
pub use payouts_get::*;
pub use refund::*;

use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use crate::models::Transaction;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalResponse {
    #[serde(default)]
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub create_time: Option<chrono::NaiveDateTime>,
    #[serde(default)]
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub update_time: Option<chrono::NaiveDateTime>,
    pub status: Option<String>,
    pub id: String,
    pub intent: Option<String>,
    pub payer: Option<PaypalPayer>,
    pub purchase_units: Option<Vec<PaypalPurchaseUnits>>,
    pub links: Option<Vec<PaypalLink>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayer {
    pub email_address: Option<String>,
    pub payer_id: String,
    pub address: Option<PaypalAddress>,
    pub name: Option<PaypalName>,
    pub phone: Option<PaypalPhone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalName {
    pub given_name: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPhone {
    pub phone_number: Option<NationalNumber>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationalNumber {
    pub national_number: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalAddress {
    pub address_line_1: Option<String>,
    pub admin_area_2: Option<String>,
    pub admin_area_1: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
}

// {
//   'create_time': '2019-06-08T05:58:54Z',
//   'update_time': '2019-06-08T05:58:54Z',
//   'id': '6TP15053L3858374E',
//   'intent': 'CAPTURE',
//   'status': 'COMPLETED',
//   'payer': {'email_address': 'jade@gm.com',
//   'payer_id': 'DKPJV8SETUYMQ',
//   'address': {'address_line_1': '1 Main St',
//     'admin_area_2': 'San Jose',
//     'admin_area_1': 'CA',
//     'postal_code': '95131',
//     'country_code': 'US'},
//   'name': {'given_name': 'jade', 'surname': 'Tolstoy'},
//   'phone': {'phone_number': {'national_number': '4084473359'}}},
//   'purchase_units': [
//      PURCHASEUNITS
//   ],
//   'links': [
//     {
//       'href': 'https://api.sandbox.paypal.com/v2/checkout/orders/6TP15053L3858374E',
//       'rel': 'self',
//       'method': 'GET',
//       'title': 'GET'
//     }
//   ]
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPurchaseUnits {
    pub reference_id: Option<String>,
    pub soft_descriptor: Option<String>,
    pub amount: PaypalAmount,
    pub payee: PaypalPayee,
    pub shipping: Option<PaypalShipping>,
    pub payments: PaypalPayments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalAmount {
    pub value: String,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayee {
    pub email_address: Option<String>,
    pub merchant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalShipping {
    pub name: Option<PaypalPayeeName>,
    pub address: Option<PaypalAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayeeName {
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalPayments {
    pub captures: Option<Vec<PaypalCaptures>>,
}

// {
//   'reference_id': 'default',
//   'soft_descriptor': 'PAYPAL *PYPLTEST',
//   'amount': {'value': '0.01', 'currency_code': 'USD'},
//   'payee': {
//     'email_address': 'barco.03-facilitator@gmail.com',
//     'merchant_id': 'YQZCHTGHUK5P8'
//   },
//   'shipping': {
//     'name': {'full_name': 'jade Tolstoy'},
//     'address': {
//       'address_line_1': '1 Main St',
//       'admin_area_2': 'San Jose',
//       'admin_area_1': 'CA',
//       'postal_code': '95131',
//       'country_code': 'US'
//     }
//   },
//   'payments': {
//     'captures': [ CAPTURES ]
//   }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalCaptures {
    pub status: Option<String>,
    pub id: String,
    pub final_capture: Option<bool>,
    pub create_time: String,
    pub update_time: String,
    pub amount: PaypalAmount,
    pub seller_protection: Option<PaypalSellerProtection>,
    pub links: Option<Vec<PaypalLink>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalSellerProtection {
    pub status: Option<String>,
    pub dispute_categories: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaypalLink {
    pub href: Option<String>,
    pub rel: Option<String>,
    pub method: Option<String>,
    pub title: Option<String>,
    pub encType: Option<String>,
}



/////////////////////////////////
/////// Tests
////////////////////////////////


mod tests {
    use super::*;

    #[test]
    fn deserializes_paypal_response_1() {

        let test_paypal_response: &str = r#"
        {
            "create_time":"2019-07-19T01:39:39Z",
            "update_time":"2019-07-19T01:39:39Z",
            "id":"05126684DT458323A",
            "intent":"CAPTURE",
            "status":"COMPLETED",
            "payer":{
                "email_address":"jade@gm.com",
                "payer_id":"DKPJV8SETUYMQ",
                "address":{
                    "address_line_1":"1 Main St",
                    "admin_area_2":"San Jose",
                    "admin_area_1":"CA",
                    "postal_code":"95131",
                    "country_code":"US"
                },
                "name":{
                    "given_name":"jade",
                    "surname":"Tolstoy"
                },
                "phone":{
                    "phone_number":{
                        "national_number":"4084473359"
                    }
                }
            },
            "purchase_units":[
                {
                    "reference_id":"default",
                    "amount":{
                        "value":"14.55",
                        "currency_code":"USD"
                    },
                    "payee":{
                        "email_address":"barco.03-facilitator@gmail.com",
                        "merchant_id": "YQZCHTGHUK5P8"
                    },
                    "shipping":{
                        "name":{
                            "full_name":"jade Tolstoy"
                        },
                        "address":{
                            "address_line_1":"1 Main St",
                            "admin_area_2":"San Jose",
                            "admin_area_1":"CA",
                            "postal_code":"95131",
                            "country_code":"US"
                        }
                    },
                    "payments":{
                        "captures":[
                            {
                                "id":"8WC31125DT623342N",
                                "status":"COMPLETED",
                                "final_capture":true,
                                "create_time":"2019-07-19T01:39:39Z",
                                "update_time":"2019-07-19T01:39:39Z",
                                "amount":{"value":"14.55", "currency_code":"USD"},
                                "seller_protection":{
                                    "status":"ELIGIBLE",
                                    "dispute_categories":["ITEM_NOT_RECEIVED","UNAUTHORIZED_TRANSACTION"]
                                },
                                "links":[
                                    {"href":"https://api.sandbox.paypal.com/v2/payments/captures/8WC31125DT623342N","rel":"self","method":"GET","title":"GET"},
                                    {"href":"https://api.sandbox.paypal.com/v2/payments/captures/8WC31125DT623342N/refund","rel":"refund","method":"POST","title":"POST"},
                                    {"href":"https://api.sandbox.paypal.com/v2/checkout/orders/05126684DT458323A","rel":"up","method":"GET","title":"GET"}
                                ]
                            }
                        ]
                    }
                }
            ],
            "links":[
                {"href":"https://api.sandbox.paypal.com/v2/checkout/orders/05126684DT458323A","rel":"self","method":"GET","title":"GET"}
            ]
        }
        "#;

        let t2: PaypalResponse = serde_json::from_str::<PaypalResponse>(&test_paypal_response)
            .expect("Err deserializing PaypalResponse");
        let t3: Transaction = Transaction::from(t2.clone());

        assert_eq!(
            String::from("jade@gm.com"),
            t2.payer.expect("payer field missing")
            .email_address.expect("email_address field missing")
        );
        assert_eq!(format!("txn_{}", t2.id), t3.id);
    }


    #[test]
    fn deserializes_paypal_response_2() {

        let test_paypal_response: &str = r#"
        {
            "create_time":"2019-09-30T13:49:11Z",
            "update_time":"2019-09-30T13:49:11Z",
            "id":"14U27371L6972973T",
            "intent":"CAPTURE",
            "status":"COMPLETED",
            "payer":{
                "email_address":"nadia@gm.com",
                "payer_id":"ATSY6LZTPHUPS",
                "address":{
                    "country_code":"US"
                },
                "name":{
                    "given_name":"Nadia",
                    "surname":"Supernova"
                }
            },
            "purchase_units":[
                {
                    "reference_id":"default",
                    "amount":{
                        "value":"29.89",
                        "currency_code":"USD"
                    },
                    "payee":{
                        "email_address":"s4143868-facilitator@gmail.com",
                        "merchant_id":"EKBZYVBPSZENE"
                    },
                    "shipping":{
                        "name":{
                            "full_name":"Nadia Supernova"
                        },
                        "address":{
                            "address_line_1":"1 Main St",
                            "admin_area_2":"San Jose",
                            "admin_area_1":"CA",
                            "postal_code":"95131",
                            "country_code":"US"
                        }
                    },
                    "payments":{
                        "captures":[
                            {
                                "status":"COMPLETED",
                                "id":"6VB40248DY409084T",
                                "final_capture":true,
                                "create_time": "2019-09-30T13:49:11Z",
                                "update_time":"2019-09-30T13:49:11Z",
                                "amount":{
                                    "value":"29.89",
                                    "currency_code":"USD"
                                },
                                "seller_protection":{
                                    "status":"ELIGIBLE",
                                    "dispute_categories":["ITEM_NOT_RECEIVED", "UNAUTHORIZED_TRANSACTION"]
                                },
                                "links":[
                                    {
                                        "href":"https://api.sandbox.paypal.com/v2/payments/captures/6VB40248DY409084T",
                                        "rel":"self",
                                        "method":"GET",
                                        "title":"GET"
                                    },
                                    {
                                        "href":"https://api.sandbox.paypal.com/v2/payments/captures/6VB40248DY409084T/refund",
                                        "rel":"refund",
                                        "method":"POST",
                                        "title":"POST"
                                    },
                                    {
                                        "href":"https://api.sandbox.paypal.com/v2/checkout/orders/14U27371L6972973T",
                                        "rel":"up",
                                        "method":"GET",
                                        "title":"GET"
                                    }
                                ]
                            }
                        ]
                    }
                }
            ],
            "links":[
                {
                    "href":"https://api.sandbox.paypal.com/v2/checkout/orders/14U27371L6972973T",
                    "rel":"self",
                    "method":"GET",
                    "title":"GET"
                }
            ]
        }
        "#;

        let t2: PaypalResponse = serde_json::from_str::<PaypalResponse>(&test_paypal_response)
            .expect("Err deserializing PaypalResponse");
        let t3: Transaction = Transaction::from(t2.clone());

        assert_eq!(
            String::from("nadia@gm.com"),
            t2.payer.expect("payer field missing")
            .email_address.expect("email_address field missing")
        );
        assert_eq!(format!("txn_{}", t2.id), t3.id);
    }


    #[test]
    fn deserializes_paypal_response_production_response_3() {

        let test_paypal_response: &str = r#"
        {
            "create_time":"2020-03-04T01:14:15Z",
            "update_time":"2020-03-04T01:16:06Z",
            "id":"8V028869SF9814200",
            "intent":"CAPTURE",
            "status":"COMPLETED",
            "payer":{
                "email_address":"kengphotolap@hotmail.com",
                "payer_id":"9EQCYPT86PTQ6",
                "address":{"country_code":"TH"},
                "name":{"given_name":"keerati","surname":"nongmeesap"}
            },
            "purchase_units":[
                {
                    "reference_id":"default",
                    "soft_descriptor":"PAYPAL *RELAY",
                    "amount":{
                        "value":"15.00",
                        "currency_code":"USD"
                    },
                    "payee":{
                        "email_address":"johnny@relaydownloads.com","merchant_id":"KVW4L8AGXEYZN"
                    },
                    "shipping":{
                        "name":{ "full_name":"keerati nongmeesap" },
                        "address":{
                            "address_line_1":"624 Condolette Ladprao 46",
                            "admin_area_2":"na",
                            "admin_area_1":"na",
                            "postal_code":"10240",
                            "country_code":"TH"
                        }
                    },
                    "payments":{
                        "captures":[
                            {
                                "status":"COMPLETED",
                                "id":"827113479M1544247",
                                "final_capture":true,
                                "create_time":"2020-03-04T01:16:06Z",
                                "update_time":"2020-03-04T01:16:06Z",
                                "amount":{"value":"15.00","currency_code":"USD"},
                                "seller_protection":{"status":"NOT_ELIGIBLE"},
                                "links":[
                                    {"href":"https://api.paypal.com/v2/payments/captures/827113479M1544247","rel":"self","method":"GET","title":"GET"},
                                    {"href":"https://api.paypal.com/v2/payments/captures/827113479M1544247/refund","rel":"refund","method":"POST","title":"POST"},
                                    {"href":"https://api.paypal.com/v2/checkout/orders/8V028869SF9814200","rel":"up","method":"GET","title":"GET"}
                                ]
                            }
                        ]
                    }
                }
            ],
            "links":[{"href":"https://api.paypal.com/v2/checkout/orders/8V028869SF9814200","rel":"self","method":"GET","title":"GET"}]
        }
        "#;

        let t2: PaypalResponse = serde_json::from_str::<PaypalResponse>(&test_paypal_response)
            .expect("Err deserializing PaypalResponse");
        let t3: Transaction = Transaction::from(t2.clone());

        assert_eq!(
            String::from("kengphotolap@hotmail.com"),
            t2.payer.expect("payer field missing")
            .email_address.expect("email_address field missing")
        );
        assert_eq!(format!("txn_{}", t2.id), t3.id);
    }

    ////// REFUNDS


    #[test]
    fn deserializes_paypal_refund_response() {

        let test_paypal_response: &str = r#"
        {
            "id":"216381144F441871Y",
            "create_time":"2019-10-12T03:59:28Z",
            "update_time":"2019-10-12T03:59:28Z",
            "state":"completed",
            "amount": {
                "total":"1.50",
                "currency":"USD"
            },
            "refund_from_transaction_fee": {
                "currency":"USD",
                "value":"0.05"
            },
            "total_refunded_amount": {
                "currency":"USD",
                "value":"15.00"
            },
            "refund_from_received_amount": {
                "currency":"USD",
                "value":"1.45"
            },
            "sale_id": "9YW12349VC009901M",
            "invoice_number": "invoice_ff442335-8bac-4ca2-b7c4-fb1373fe8e9c",
            "links": [
                {
                    "href":"https://api.sandbox.paypal.com/v1/payments/refund/216381144F441871Y",
                    "rel":"self","method":"GET"
                },
                {
                    "href":"https://api.sandbox.paypal.com/v1/payments/sale/9YW12349VC009901M",
                    "rel":"sale",
                    "method":"GET"
                }
            ]
        }
        "#;

        let t2: PaypalRefundResponse = serde_json::from_str::<PaypalRefundResponse>(&test_paypal_response)
            .expect("Err deserializing PaypalRefundResponse");

        assert_eq!(
            String::from("216381144F441871Y"),
            t2.id,
        );
    }

    #[test]
    fn deserializes_paypal_refund_details() {

        let test_paypal_response: &str = r#"
        {
            "id":"98C172368X154215P",
            "amount": {
                "currency_code":"USD",
                "value":"1.50"
            },
            "seller_payable_breakdown":{
                "gross_amount":{
                    "currency_code":"USD",
                    "value":"1.50"
                },
                "paypal_fee":{
                    "currency_code":"USD",
                    "value":"0.05"
                },
                "net_amount":{
                    "currency_code":"USD",
                    "value":"1.45"
                },
                "total_refunded_amount":{
                    "currency_code":"USD",
                    "value":"37.11"
                }
            },
            "invoice_id":"5",
            "status":"COMPLETED",
            "create_time":"2019-10-11T10:24:31-07:00",
            "update_time":"2019-10-11T10:24:31-07:00",
            "links":[
                {
                    "href":"https://api.sandbox.paypal.com/v2/payments/refunds/98C172368X154215P",
                    "rel":"self",
                    "method":"GET"
                },
                {
                    "href":"https://api.sandbox.paypal.com/v2/payments/captures/9YW12349VC009901M",
                    "rel":"up",
                    "method":"GET"
                }
            ]
        }
        "#;

        let t2: PaypalRefundDetails = serde_json::from_str::<PaypalRefundDetails>(&test_paypal_response)
            .expect("Err deserializing PaypalRefundDetails");

        assert_eq!(
            String::from("98C172368X154215P"),
            t2.id,
        );
    }

}