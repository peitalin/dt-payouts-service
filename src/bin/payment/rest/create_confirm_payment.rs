use actix_web::{
    HttpRequest,
    HttpResponse,
    HttpMessage,
    web, web::Query, web::Json,
    Error,
};
use std::str::FromStr;
use gm::utils::dates::from_datetimestr_to_option_naivedatetime;
use gm::utils::dates::from_datetimestr_to_naivedatetime;

use crate::db;
use crate::db::GetPool;
use crate::models::{
    PaypalResponse,
    StripeError, DbError, ErrJson,
    Transaction,
    CartRpc,
    OrderDb,
    TxQuery,
    PayoutItem,
    OrderItemRpc,
    Currency,
    PayoutSplit,
    to_payout_items,
    Affiliate,
    CLICK_COOKIE_NAME,
};
use crate::rpc::AffiliateConversionResponse;

use crate::models::paypal::{
    PaypalPurchaseUnits,
    PaypalCaptures,
};
use crate::rest::affiliates::convert_buyer_affiliate;
use crate::pricing::PaymentFees;
use crate::{AppState};
use crate::rpc::{
    rpc_get_cart,
    rpc_create_order,
    rpc_cart_clear,
    rpc_confirm_order,
    rpc_save_payment_method,
    rpc_get_affiliate_profile_by_click_id,
    rpc_convert_click,
};

use crate::rest::paypal::{
    paypal_create_order,
    paypal_confirm_order,
};
use crate::rest::stripe::{
    create_payment_intent,
};
// import stripe traits to enable Requests
use gm::models::stripe;
use gm::models::stripe::{
    PaymentIntent,
    PaymentIntentCreateParams,
    PaymentIntentUpdateParams,
    PaymentIntentConfirmParams,
    PaymentIntentCaptureParams,
    PaymentIntentCancelParams,
};
use crate::payment_clients::{
    PaymentIntentMsg,
};


/////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////
/////// CREATE PAYMENT
/////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////


#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderPaymentParams {
    pub order_id: String,
    pub currency: String,
    pub subtotal: i32,
    pub taxes: i32,
    pub payment_processing_fee: i32,
    pub total: i32,
    pub payment_processor_data: String,
    pub order_items_rpc: Vec<OrderItemRpc>
}


#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct PaymentProcessorData {
    pub payment_method: Option<String>,
    pub save_payment_method: Option<bool>,
    pub customer_id: Option<String>,

    //// FRENZY TEST PARAMS ONLY
    /// If theses parameters are present, go into test mode.
    /// No external API calls made to Stripe, Paypal, Sendgrid, etc.
    #[serde(default)]
    pub mode: Option<String>, // "gm-frenzy";
    /// NOTE: serde Deserialize Option<NavaDateTime>
    ///
    /// serde(default) will first deserialize a missing field as Option::None
    /// otherwise serde(deserialize_with=...) will deserialize null/datetimestr inputs
    /// as Option<NaiveDateTime>
    ///
    /// https://stackoverflow.com/questions/44301748/how-can-i-deserialize-an-optional-field-with-custom-functions-using-serde
    #[serde(default)]
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub date: Option<chrono::NaiveDateTime>
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct StripePaymentProcessorData {
    pub payment_method: String,
    pub save_payment_method: Option<bool>,
    pub customer_id: Option<String>,
}

impl From<PaymentProcessorData> for StripePaymentProcessorData {
    fn from(p: PaymentProcessorData) -> Self {

        match (&p.customer_id, &p.payment_method) {
            (None, None)    => {
                debug!(r#"not saving payment method, missing
                    customer_id and payment_method_id"#);
            },
            (Some(_), None) => {
                debug!("not saving payment method, missing payment_method_id");
            },
            (None, Some(_)) => {
                debug!("not saving payment method, missing customer_id");
            },
            (Some(c), Some(p)) => {
                debug!("creating payment method: {:?} for customer {:?}", &p, &c);
            },
        };

        let pmethod = p.payment_method
            .expect("No PaymentMethodId: `pi_xxxx` for a stripe transaction");

        Self {
            payment_method: pmethod,
            save_payment_method: p.save_payment_method,
            customer_id: p.customer_id
        }
    }
}

//// FRENZY TEST PARAMS ONLY
/// If theses parameters are present, go into test mode.
/// No external API calls made to Stripe, Paypal, Sendgrid, etc.
#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct MockPaymentProcessorData {
    pub mode: String, // "gm-frenzy";
    /// NOTE: serde Deserialize NavaDateTime, no Option<..>
    #[serde(deserialize_with = "from_datetimestr_to_naivedatetime")]
    pub date: chrono::NaiveDateTime
}

impl From<PaymentProcessorData> for MockPaymentProcessorData {
    fn from(p: PaymentProcessorData) -> Self {
        Self {
            mode: p.mode.clone().expect("No field `mode` in MockPaymentProcessorData"),
            date: p.date.clone().expect("No field `date` in MockPaymentProcessorData"),
        }
    }
}


pub async fn create_payment(
    req: HttpRequest,
    json: Json<OrderPaymentParams>
) -> Result<HttpResponse, Error> {

    debug!(">>>>> create_payment(...)");
    debug!("req headers: {:?}", req.headers());
    // Parse the payment processor data
    let params: OrderPaymentParams = json.clone();
    debug!("json: {:?}", params);

    let pay_proc_data: PaymentProcessorData = serde_json::from_str(
        &params.payment_processor_data
    ).map_err(|e| Error::from(StripeError::DeserializationError(errJson!(e))))?;

    debug!("paymentProcessorData parsed: {:?}", pay_proc_data);

    match (&pay_proc_data.mode, &pay_proc_data.date) {
        (Some(_), Some(_)) => {
            // test params found, MOCK transaction
            create_payment_mock(
                params,
                MockPaymentProcessorData::from(pay_proc_data)
            ).await
        },
        _ => {
            // No test params, LIVE transaction
            create_payment_live(
                req,
                params,
                StripePaymentProcessorData::from(pay_proc_data)
            ).await
        }
    }

}


pub async fn create_payment_mock(
    params: OrderPaymentParams,
    mock_processor_data: MockPaymentProcessorData,
) -> Result<HttpResponse, Error> {

    debug!(
        "incoming MockPaymentProcessorData from gm-shopping: {:?}",
        &mock_processor_data
    );
    // Generate mock Stripe PaymentIntent to send back
    let mock_payment_intent = PaymentIntent::new_mock_data(
        params.total,
        mock_processor_data.date,
    );

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(mock_payment_intent))
}


pub async fn create_payment_live(
    req: HttpRequest,
    params: OrderPaymentParams,
    payment_processor_data: StripePaymentProcessorData,
) -> Result<HttpResponse, Error> {

    debug!(
        "incoming StripePaymentProcessorData from gm-shopping: {:?}",
        &payment_processor_data
    );

    // Create payment intent data
    let stripe_params = stripe::PaymentIntentCreateParams {
        amount: params.total as u64,
        currency: stripe::Currency::from_str(&params.currency).unwrap_or(stripe::Currency::USD),
        customer: payment_processor_data.customer_id.clone(),
        payment_method: Some(payment_processor_data.payment_method.clone()),
        payment_method_types: Some(vec![stripe::PaymentIntentMethodType::Card]),
        save_payment_method: payment_processor_data.save_payment_method.clone(),
        capture_method: Some(stripe::PaymentIntentCaptureMethod::Automatic),
        confirm: Some(false), // Do not try auto-confirm. Do in 2nd step
        confirmation_method: Some("automatic".to_string()),
        description: None,
        // application_fee_amount: None,
        // metadata: None,
        // on_behalf_of: None,
        // receipt_email: None,
        // return_url: None,
        // shipping: None,
        // statement_descriptor: None,
        // transfer_data: None,
        // transfer_group: None,
    };

    // Create the Stripe payment intent.
    let pi_created: stripe::PaymentIntent = AppState::stripeActor(&req)
        .send(PaymentIntentMsg::Create(stripe_params.clone()))
        .await??;

    debug!("Stripe payment intent created: {:?}", pi_created);

    // Ask the user service to save the payment method (if applicable)
    match payment_processor_data.save_payment_method {
        None => {
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(pi_created))
        },
        Some(save_pm) => if !save_pm {
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(pi_created))

        } else {

            // Check that pm_create actually did have a customer_id attached
            match (pi_created.customer.clone(), pi_created.payment_method.clone()) {
                (None, None)    => debug!(
                    r#"not creating payment method, missing
                    customer_id and payment_method_id"#
                ),
                (Some(_), None) => debug!(
                    "not creating payment method, missing payment_method_id"
                ),
                (None, Some(_)) => debug!(
                    "not creating payment method, missing customer_id"
                ),
                (Some(customer_id), Some(payment_method_id)) => {
                    debug!(
                        "creating payment method: {:?} for customer {:?}",
                        &payment_method_id,
                        &customer_id,
                    );
                    let a = rpc_save_payment_method(
                        AppState::httpClient(&req),
                        &req,
                        payment_method_id,
                        customer_id
                    ).await;
                    debug!("response from gm-user: {:?}", a);
                },
            };

            Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(pi_created))
        }
    }
}


/////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////
/////// CONFIRM PAYMENT
/////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct PaymentConfirmProcessorData {
    pub payment_intent: PaymentIntent,
    pub customer: Option<String>,
    //// FRENZY TEST PARAMS ONLY
    /// If theses parameters are present, go into test mode.
    /// No external API calls made to Stripe, Paypal, Sendgrid, etc.

    #[serde(default)]
    pub mode: Option<String>, // "gm-frenzy";
    /// NOTE: serde Deserialize Option<NavaDateTime>
    /// serde(default) will first deserialize null/undefined => Option
    /// otherwise the from_datetimestr deserializer may panic
    #[serde(default)]
    #[serde(deserialize_with = "from_datetimestr_to_option_naivedatetime")]
    pub date: Option<chrono::NaiveDateTime>
}

pub async fn confirm_payment(
    req: HttpRequest,
    json: Json<OrderPaymentParams>
) -> Result<HttpResponse, Error> {

    debug!(">>>>> confirm_payment(...)");
    debug!("req headers: {:?}", req.headers());
    debug!("req cookies: {:?}", req.cookies());

    // Parse the payment processor data
    let order_params: OrderPaymentParams = json.clone();
    debug!("confirm_payment: incoming payment_processor_data: {:?}", order_params);

    let pay_proc_data: PaymentConfirmProcessorData = serde_json::from_str(
        &order_params.payment_processor_data
    ).map_err(Error::from)?;

    debug!("pay_proc_data: {:?}", &pay_proc_data);
    let payment_intent = pay_proc_data.payment_intent.clone();
    let customer       = pay_proc_data.customer.clone();

    // convert buyer-affiliate and retrieve userId if there is one.
    let buyer_affiliate_user_id = convert_buyer_affiliate(&req, order_params.order_id.clone()).await;
    debug!("buyer_affiliate_user_id: {:?}", &buyer_affiliate_user_id);

    // check for Mock params
    match (&pay_proc_data.mode, &pay_proc_data.date) {
        (Some(_), Some(mock_date)) => {
            // test params found, MOCK confirmation
            // currently same as LIVE since there are no outgoing API calls
            // for the payment confirm step (done on frontend)
            confirm_payment_handler(
                req,
                order_params,
                payment_intent,
                customer,
                mock_date, // &chrono::NaiveDateTime
                buyer_affiliate_user_id,
            ).await
        },
        _ => {
            // No test params, LIVE confirmation
            let created_at = chrono::NaiveDateTime::from_timestamp(
                payment_intent.created as i64, 0
            );
            confirm_payment_handler(
                req,
                order_params,
                payment_intent,
                customer,
                &created_at,
                buyer_affiliate_user_id,
            ).await
        }
    }
}



pub async fn confirm_payment_handler(
    req: HttpRequest,
    order_params: OrderPaymentParams,
    payment_intent: PaymentIntent,
    customer: Option<String>,
    created_at: &chrono::NaiveDateTime,
    buyer_affiliate_user_id: Option<String>,
) -> Result<HttpResponse, Error> {


    // Execute and confirm the Stripe payment intent.
    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let tx_id = format!("txn_{}", payment_intent.id.clone());

    // Create payout_items
    let payout_items: Vec<PayoutItem> = to_payout_items(
        &conn,
        order_params.order_items_rpc.clone(),
        &tx_id,
        &created_at,
        buyer_affiliate_user_id
    );
    debug!("created payout_items: {:?}", &payout_items);

    // seller pays payment_processing_fee for each orderItem:
    // this figure is for tx reference only, not payoutItems
    // sum over transaction fees on the PayoutItems
    let payment_proc_fee = payout_items
        .iter()
        .fold(0, |acc: i32, pitem: &PayoutItem| {
            acc + pitem.payment_processing_fee
        });

    let tx = Transaction {
        id: tx_id.clone(),
        subtotal: order_params.subtotal,
        taxes: order_params.taxes,
        payment_processing_fee: payment_proc_fee,
        created_at: *created_at,
        currency: Currency::from_str(&payment_intent.currency.to_string()).ok(),
        charge_id: Some(payment_intent.id.clone().to_string()),
        customer_id: customer,
        order_id: Some(order_params.order_id),
        payment_processor: Some("Stripe".to_string()),
        payment_method_id: payment_intent.payment_method.clone(),
        payment_intent_id: Some(payment_intent.id.to_string()),
        refund_id: None,
        details: None,
    };

    debug!("created tx: {:?}", &tx);

    // Write both transaction and payout_items to DB in single transaction
    // for double-entry accounting.
    let (_tx_result, _pitems_result) = db::write_transaction_and_payout_items(
        &conn,
        &tx,
        &payout_items
    ).map_err(Error::from)?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .json(json!({
        "transaction": tx,
    })))
}





#[test]
fn deserializes_checkout_confirm_params() {

    let test_str = r#"
    {
        "orderId":"order_52b66e8d-3899-439a-b6a4-cf866f2d5514",
        "currency":"USD",
        "subtotal":140,
        "taxes":0,
        "paymentProcessingFee":0,
        "total":140,
        "paymentProcessorData": "{\"paymentIntent\":{\"id\":\"pi_1GNYPaKqy1M9WH1D9ROXHGL8\", \"object\":\"payment_intent\", \"amount\":140, \"canceled_at\":null, \"cancellation_reason\":null, \"capture_method\":\"automatic\", \"client_secret\":\"pi_1GNYPaKqy1M9WH1D9ROXHGL8_secret_Jq9sJnOFBMkIPJKs95Ksp101f\", \"confirmation_method\":\"automatic\", \"created\":1584424582, \"currency\":\"usd\", \"description\":null, \"last_payment_error\":null, \"livemode\":false, \"next_action\":null, \"payment_method\":\"pm_1GGLvnKqy1M9WH1DECNPO90q\", \"payment_method_types\":[\"card\"], \"receipt_email\":null, \"setup_future_usage\":null, \"shipping\":null, \"source\":null, \"status\":\"succeeded\" }}",
        "orderItemsRpc":[
            {
                "id":"oitem_40a590a8-9a93-438d-a396-aac0cab4f00c",
                "actualPrice":140,
                "storeId": "store_123123123",
                "createdAt":"2020-03-17T05:56:22.462Z",
                "currency":"USD"
            }
        ]
    }
    "#;
    // NOTE: do not newline the paymentProcessorData: String
    // it will complain about "control characters"

    let order_res: OrderPaymentParams = serde_json::from_str::<OrderPaymentParams>(test_str)
        .expect("OrderPaymentParams");

    assert_eq!(order_res.order_id, String::from("order_52b66e8d-3899-439a-b6a4-cf866f2d5514"));

    let sconfirm = serde_json::from_str::<PaymentConfirmProcessorData>(
        &order_res.payment_processor_data
    );

    match sconfirm {
        Ok(s) => assert_eq!(s.payment_intent.id, String::from("pi_1GNYPaKqy1M9WH1D9ROXHGL8")),
        Err(e) => panic!(e.to_string()),
    }

}



#[test]
fn deserializes_payment_processor_data_with_null_date() {

    let test_str = r#"
    {
        "paymentMethod":"pm_1Gf6LJKqy1M9WH1DDgMqeQfS",
        "savePaymentMethod":false,
        "mock":null,
        "date":null
    }
    "#;

    let ppd = serde_json::from_str::<PaymentProcessorData>(test_str);

    match ppd {
        Ok(p) => assert_eq!(
            p.payment_method.expect("payment_method panic"),
            String::from("pm_1Gf6LJKqy1M9WH1DDgMqeQfS")
        ),
        Err(e) => {
            // Null is not None, use serde_aux to handle null values
            // https://docs.rs/serde-aux/0.6.1/serde_aux/field_attributes/fn.deserialize_default_from_null.html
            debug!("This should fail...... {:?}", e.to_string());
        }
    }
}


#[test]
fn deserializes_payment_processor_data_with_missing_date() {

    let test_str = r#"
    {
        "paymentMethod":"pm_1Gf6LJKqy1M9WH1DDgMqeQfS",
        "savePaymentMethod":false,
        "mock":null
    }
    "#;

    let ppd = serde_json::from_str::<PaymentProcessorData>(test_str);

    match ppd {
        Ok(p) => assert_eq!(
            p.payment_method.expect("payment_method panic"),
            String::from("pm_1Gf6LJKqy1M9WH1DDgMqeQfS")
        ),
        Err(e) => panic!(e.to_string()),
    }
}


#[test]
fn deserializes_confirm_payment_payment_processor_data_with_datestring() {

    let test_str = r#"
    {
        "paymentIntent":{
            "id":"pi_1GNYPaKqy1M9WH1D9ROXHGL8",
            "object":"payment_intent",
            "amount":140,
            "canceled_at":null,
            "cancellation_reason":null,
            "capture_method":"automatic",
            "client_secret":"pi_1GNYPaKqy1M9WH1D9ROXHGL8_secret_Jq9sJnOFBMkIPJKs95Ksp101f",
            "confirmation_method":"automatic",
            "created":1584424582,
            "currency":"usd",
            "description":null,
            "last_payment_error":null,
            "livemode":false,
            "next_action":null,
            "payment_method":"pm_1GGLvnKqy1M9WH1DECNPO90q",
            "payment_method_types":["card"],
            "receipt_email":null,
            "setup_future_usage":null,
            "shipping":null,
            "source":null,
            "status":"succeeded"
        },
        "mock":null,
        "date":"2020-03-29T22:07:05.123Z"
    }
    "#;
        // "createdAt":"2020-03-17T05:56:22.462Z",

    let ppd = serde_json::from_str::<PaymentConfirmProcessorData>(test_str);

    match ppd {
        Ok(p) => assert_eq!(
            p.payment_intent.id,
            String::from("pi_1GNYPaKqy1M9WH1D9ROXHGL8")
        ),
        Err(e) => panic!(e.to_string()),
    }
}