use actix_web::{
    HttpRequest, HttpResponse,
    web, web::Query, web::Json,
    Error,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use futures::{
    future,
    future::Either,
    Future,
};

use crate::db;
use crate::db::GetPool;
use crate::models::{
    StripeError, DbError, ErrJson,
    UserId,
    UpdateUserProfile,
    PaymentMethodDb,
    PaymentMethodAddress,
};
use crate::{AppState};
use crate::rpc::{rpc_setup_intents_for_user};

use gm::models::stripe;
use gm::models::stripe::{
    SetupIntent,
    PaymentMethod,
};

use crate::payment_clients::{
    SetupIntentMsg,
    PaymentMethodMsg,
};


// POST /setup_intent/create
pub async fn create_setup_intent(
    req: HttpRequest,
    json: Json<stripe::SetupIntentCreateParams>,
    query: Query<UserId>,
) -> Result<HttpResponse, Error> {

    // This path will instantly confirm the setup intent,
    let user_id = query.into_inner().user_id;
    let setup_intent_create_params = json.into_inner();
    debug!("json body: {:?}", &setup_intent_create_params);

    let conn = AppState::databaseActor(&req)
                .send(GetPool::Postgres)
                .await??;

    let (
        stripe_setup_intent_response,
        db_payment_method_result,
    ) = handle_create_setup_intent(
        req,
        setup_intent_create_params,
        user_id,
        conn
    ).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "stripe_response": stripe_setup_intent_response,
            "db_response": db_payment_method_result,
        })))
}


pub async fn handle_create_setup_intent(
    req: HttpRequest,
    setup_intent_create_params: stripe::SetupIntentCreateParams,
    user_id: String,
    conn: PooledConnection<ConnectionManager<PgConnection>>
) -> Result<(stripe::SetupIntent, PaymentMethodDb), Error> {

    let stripe_response: SetupIntent = AppState::stripeActor(&req)
        .send(SetupIntentMsg::Create(setup_intent_create_params))
        .await??;

    debug!("{:?}", stripe_response);
    // 2. write payment method details to payment_methods table

    let payment_method = PaymentMethodDb {
        id: stripe_response.payment_method
            .clone().expect("payment_method_id missing"),
        user_id: user_id,
        created_at: chrono::NaiveDateTime::from_timestamp(
            stripe_response.created.expect("stripe_response.created missing"),
            0
        ),
        updated_at: None,
        customer_id: stripe_response.customer.clone(),
        payment_processor: Some(String::from("Stripe")),
        payment_method_types: stripe_response.payment_method_types.clone(),
        // last4: stripe_response.payment_method_options,
        last4: None,
        exp_month: None,
        exp_year: None,
        email: None,
        name: None,
        details: Some(stripe_response.id.clone()),
    };


    let stripe_payment_method: PaymentMethod = AppState::stripeActor(&req)
        .send(PaymentMethodMsg::Retrieve(
            payment_method.id.clone()
        ))
        .await??;

    println!("payment method: {:#?}", stripe_payment_method);

    match stripe_payment_method.card {
        None => Err(Error::from(StripeError::PaymentMethod(
            ErrJson::new("No card returned from Stripe")
        ))),
        Some(card) => {

            match stripe_payment_method.billing_details.address {
                None => (),
                Some(address) => {
                    let payment_method_address = PaymentMethodAddress {
                        payment_method_id: payment_method.id.clone(),
                        line1: address.line1,
                        line2: address.line2,
                        city: address.city,
                        state: address.state,
                        postal_code: address.postal_code,
                        country: address.country,
                        town: address.town,
                    };
                    let _ = db::write_payment_method_address(
                        &conn, payment_method_address
                    );
                }
            };

            let db_res = db::write_payment_method(
                &conn,
                payment_method
                    .update_last4(card.last4)
                    .update_exp_month(card.exp_month as i32)
                    .update_exp_year(card.exp_year as i32)
                    .update_name(stripe_payment_method.billing_details.name)
                    .update_email(stripe_payment_method.billing_details.email)
            ).map_err(Error::from)?;

            Ok((stripe_response, db_res))
        }
    }

}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupIntentQuery {
    setup_intent_id: String
}


// GET /setup_intent/retrieve
pub async fn retrieve_setup_intent(
    req: HttpRequest,
    query: Query<SetupIntentQuery>,
) -> Result<HttpResponse, Error> {

    let setup_intent_id = query.into_inner().setup_intent_id;

    let res: SetupIntent = AppState::stripeActor(&req)
        .send(SetupIntentMsg::Retrieve(setup_intent_id))
        .await??;

    debug!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

// POST /setup_intent/update
pub async fn update_setup_intent(
    req: HttpRequest,
    query: Query<SetupIntentQuery>,
    json: Json<stripe::SetupIntentUpdateParams>,
) -> Result<HttpResponse, Error> {

    let setup_intent_id = query.into_inner().setup_intent_id;
    let params = json.into_inner();

    let res: SetupIntent = AppState::stripeActor(&req)
        .send(SetupIntentMsg::Update(setup_intent_id, params))
        .await??;

    debug!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

// POST /setup_intent/confirm
pub async fn confirm_setup_intent(
    req: HttpRequest,
    query: Query<SetupIntentQuery>,
    json: Json<stripe::SetupIntentConfirmParams>,
) -> Result<HttpResponse, Error> {

    let setup_intent_id = query.into_inner().setup_intent_id;
    let params = json.into_inner();
    println!("Setup Intent Confirm Params: {:#?}", params);

    let res: stripe::SetupIntent =
        AppState::stripeActor(&req)
        .send(SetupIntentMsg::Confirm(setup_intent_id, params))
        .await??;


    debug!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}

// POST /setup_intent/cancel
pub async fn cancel_setup_intent(
    req: HttpRequest,
    query: Query<SetupIntentQuery>,
    json: Json<stripe::SetupIntentCancelParams>,
) -> Result<HttpResponse, Error> {

    let setup_intent_id = query.into_inner().setup_intent_id;
    let params = json.into_inner();

    let res: SetupIntent = AppState::stripeActor(&req)
        .send(SetupIntentMsg::Cancel(setup_intent_id, params))
        .await??;

    debug!("{:#?}", res);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(res))
}
