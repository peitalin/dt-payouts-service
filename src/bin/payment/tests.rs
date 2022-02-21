
#[cfg(test)]
mod tests {
    use actix_http::httpmessage::HttpMessage;
    use serde::{Deserialize, Serialize};
    use std::time::SystemTime;

    use super::*;
    use crate::models::{StripeError, ErrJson};
    use crate::actix_web::{
        Responder,
        http::header
    };
    use actix_web::{test, web, App, HttpResponse, http::StatusCode};
    use actix_web::dev::Service;

    use crate::get_stripe_api_key;
    use crate::payment_clients::stripe_client::StripeClient;


    // #[test]
    // fn deserializes_stripe_customer_create_params() {
    //     let test_str = r#"
    //     {
    //     }
    //     "#;

    //     let res = serde_json::from_str::<stripe::CustomerCreateParams>(test_str);
    //     match res {
    //         Ok(cart) => assert_eq!(cart.id, String::from("cart_000000001")),
    //         Err(e) => panic!(e.to_string()),
    //     }
    // }

    #[test]
    fn serde_sq_encodes_payment_intent_properl() {

        // Create payment intent data
        let stripeparams = stripe::PaymentIntentCreateParams {
            amount: 1233 as u64,
            currency: stripe::Currency::USD,
            customer: Some(String::from("cus_123123123")),
            payment_method: Some(String::from("pm_123123123123")),
            payment_method_types: Some(vec![stripe::PaymentIntentMethodType::Card]),
            // payment_method_types: vec![stripe::PaymentIntentMethodType::Card],

            // application_fee_amount: None,
            capture_method: Some(stripe::PaymentIntentCaptureMethod::Automatic),
            confirm: Some(false), // Do not try auto-confirm. Do in 2nd step
            confirmation_method: Some("automatic".to_string()),
            description: None,
            // metadata: None,
            // on_behalf_of: None,
            // receipt_email: None,
            // return_url: None,
            save_payment_method: Some(false),
            // shipping: None,
            // statement_descriptor: None,
            // transfer_data: None,
            // transfer_group: None,
        };

        let form_body = serde_qs::to_string(&stripeparams)
            .map_err(|e| StripeError::DeserializationError(errJson!(e)));

        match form_body {
            Ok(f) => println!("POST form: {:?}\n\n", &f),
            Err(e) => panic!("failed {:?}\n", e)
        }
    }

    #[actix_rt::test]
    async fn actix_http_test() {

        let mut app = test::init_service(
            App::new()
                .service(web::resource("/test").to(|| async { HttpResponse::Ok() }))
        ).await;

        // Create request object
        let req = test::TestRequest::with_uri("/test").to_request();

        // Execute application
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn serde_qs_encodes_and_calls_stripe() {

        let mut app = test::init_service(
            App::new().service(web::resource("/test").to(|| async move {

                let stripeparams = stripe::PaymentIntentCreateParams {
                    amount: 1233 as u64,
                    currency: stripe::Currency::USD,
                    customer: Some(String::from("cus_123123123")),
                    payment_method: Some(String::from("pm_123123123123")),
                    payment_method_types: Some(vec![stripe::PaymentIntentMethodType::Card]),
                    // payment_method_types: None,

                    // application_fee_amount: None,
                    capture_method: Some(stripe::PaymentIntentCaptureMethod::Automatic),
                    confirm: Some(false), // Do not try auto-confirm. Do in 2nd step
                    confirmation_method: Some("automatic".to_string()),
                    description: None,
                    // metadata: None,
                    // on_behalf_of: None,
                    // receipt_email: None,
                    // return_url: None,
                    save_payment_method: Some(false),
                    // shipping: None,
                    // statement_descriptor: None,
                    // transfer_data: None,
                    // transfer_group: None,
                };
                // pub const STRIPE_ENDPOINT_URL: &str = "https://api.stripe.com/v1";
                let client = StripeClient::new(get_stripe_api_key(), None, None).client;

                // NO NEED TO url encode as form if using .send_form()
                let form_body = match serde_qs::to_string(&stripeparams) {
                    Ok(f) => f,
                    Err(e) => panic!(format!("serde_qs error: {:?}", e)),
                };

                println!("\n\n===============.........==============\n");
                println!("\nPOST form: {:?}\n", &form_body);
                let res = client
                    .post(format!("https://api.stripe.com/v1/payment_intents"))
                    // .send_form(&stripeparams)
                    .send_body(&form_body)
                    .await;

                /// NOTE: send_form used serde_urlencoded library which cannot do arrays
                /// send_body(form_body) uses pre-urlencoded body from serde_qs
                /// https://github.com/actix/actix-web/issues/1329

                println!("payment_intent response: {:?}", &res);
                if let Ok(mut response) = res {
                    let bytes = response.body().await;
                    println!("\n\n");
                    println!("payment_intent API success response: {:?}", &bytes);
                }
                println!("\n===============.........==============\n\n");

                HttpResponse::Ok()
            }))
        ).await;


        // Create request object
        let req = test::TestRequest::with_uri("/test").to_request();

        // Execute application
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

    }
}
