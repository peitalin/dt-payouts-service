/// Remote Network Calls with REST
use crate::AppState;
use crate::endpoints::Endpoint;
use crate::models::errors::{
    RpcError,
    ErrJson,
};
use crate::models::{
    CartRpc,
    Transaction,
    OrderDb,
    UserId,
    UpdateUserProfile,
    UpdateStripeCustomerId,
    UserPublic,
    AuthInfo,
    AddRemovePaymentMethodResponse,
    AuthError,
};
use futures::{
    future,
    future::Either,
    Future,
};
use std::str::FromStr;
use actix_web::{
    HttpResponse,
    HttpRequest,
    HttpMessage,
    Error,
    web::Query,
    http,
};


pub async fn rpc_setup_intents_for_user(
    client: &actix_web::client::Client,
    update_user_profile: UpdateUserProfile
) -> Result<String, Error> {

    // POST /auth/profile/update
    let route = format!("/auth/profile/update");
    debug!("requesting endpoint: {}", route);

    let mut response = client.post(Endpoint::User(&route).as_url())
                    .send_json(&update_user_profile)
                    .await?;

    let bytes = response.body().await?;

    std::str::from_utf8(&bytes)
        .map(|s| String::from(s))
        .map_err(|e| Error::from(RpcError::User(errJson!(e))))

}


pub async fn rpc_update_stripe_customer_id_for_user(
    client: &actix_web::client::Client,
    update_stripe_customer_id: UpdateStripeCustomerId
) -> Result<String, Error> {

    // POST /auth/profile/update
    let route = String::from("/auth/profile/update");
    debug!("requesting endpoint: {}", route);

    let mut response = client.post(Endpoint::User(&route).as_url())
                    .send_json(&update_stripe_customer_id)
                    .await?;

    let bytes = response.body().await?;

    std::str::from_utf8(&bytes)
        .map(String::from)
        .map_err(|e| Error::from(RpcError::User(errJson!(e))))

}


pub async fn rpc_get_user_profile(
    client: &actix_web::client::Client,
    user_id: &str,
) -> Result<UserPublic, Error> {

    // POST /user/get?user_id=xxxxxxxxx
    let route = format!("/user/get?user_id={}", user_id);
    debug!("requesting endpoint: {}", route);

    let mut response = client.get(Endpoint::User(&route).as_url())
                    .send()
                    .await?;

    let bytes = response.body().await?;

    serde_json::from_slice::<UserPublic>(&bytes)
        .map_err(|e| Error::from(RpcError::User(errJson!(e))))

}

pub async fn rpc_get_user_profiles_by_store_ids(
    client: &actix_web::client::Client,
    store_ids: &Vec<String>,
) -> Result<Vec<UserPublic>, Error> {

    // POST /users/by/storeIds/
    let route = format!("/users/by/storeIds");
    debug!("requesting endpoint: {}", route);

    let mut response = client.post(Endpoint::User(&route).as_url())
                    .send_json(&json!({
                        "storeIds": store_ids
                    }))
                    .await?;

    let bytes = response.body().await?;
    // debug!("gm-user response: {:?}", bytes);
    serde_json::from_slice::<Vec<UserPublic>>(&bytes)
        .map_err(|e| Error::from(RpcError::User(errJson!(e))))

}


pub async fn rpc_get_auth_info(
    client: &actix_web::client::Client,
    req: &HttpRequest,
) -> Result<AuthInfo, Error> {

    if let Some(gm_auth_cookie) = req.cookie("gm-auth") {

        let mut response = client.get(Endpoint::User("/auth/id").as_url())
                        .cookie(gm_auth_cookie)
                        .send()
                        .await?;

        let bytes = response.body().await?;

        serde_json::from_slice::<AuthInfo>(&bytes)
            .map_err(|e| Error::from(RpcError::User(errJson!(e))))

    } else {

        Err(RpcError::User(errJson!("No gm-auth cookie!")))
                .map_err(Error::from)
    }

}

pub async fn rpc_save_payment_method(
    client: &actix_web::client::Client,
    req: &HttpRequest,
    payment_method_id: String,
    customer_id: String
) -> Result<AddRemovePaymentMethodResponse, Error> {

    // POST /users/by/storeIds/
    let route = format!("/auth/profile/addPaymentMethod");
    debug!("requesting endpoint: {}", route);
    let gm_cookie = req.cookie("gm-auth")
        .ok_or(Error::from(AuthError::NotWorthyEnough(errJson!(
            format!("missing gm-auth cookie when calling: {:?}", route)
        ))))?;
    // debug!("gm-auth cookie: {:?}", gm_cookie);

    let mut response = client.post(Endpoint::User(&route).as_url())
                    .cookie(gm_cookie)
                    .send_json(&json!({
                        "payment_method_id": payment_method_id,
                        "customer_id": customer_id
                    }))
                    .await?;

    debug!("gm-user response: {:?}", response);
    let bytes = response.body().await?;

    serde_json::from_slice::<AddRemovePaymentMethodResponse>(&bytes)
        .map_err(|e| Error::from(RpcError::User(errJson!(e))))

}
