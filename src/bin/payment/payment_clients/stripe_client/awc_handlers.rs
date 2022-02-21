use actix::prelude::{ ResponseActFuture, WrapFuture };
use std::sync::Arc;
use serde::de::DeserializeOwned;
use core::fmt::Debug;

use crate::models::{ StripeError, ErrJson };
use crate::payment_clients::stripe_client::StripeResponse;
use actix_web::Error;
use crate::payment_clients::stripe_client::STRIPE_ENDPOINT_URL;



// #[macro_export]
// macro_rules! deserialize_response {
//     ( $e:expr ) => {
//         debug!("awc response: {:?}", $e);
//         serde_json::from_slice::<T>(
//             &$e.body_bytes()
//             .await
//             .map_err(|e| StripeError::DeserializationError(errJson!(e)))?
//         ).map_err(|e| StripeError::DeserializationError(errJson!(e)))
//     };
// }


/////////////////////////////////////
/// Actix Client
/// http client using actix
/////////////////////////////////////

pub async fn awc_get<T: DeserializeOwned>(
    client: Arc<actix_web::client::Client>,
    route: &str,
) -> StripeResponse<T> {

    let request_url = format!("{}{}", STRIPE_ENDPOINT_URL, route);
    debug!("GET endpoint: {}", &request_url);

    let mut response = client
        .get(request_url)
        .send()
        .await
        .map_err(|e| StripeError::NetworkError(errJson!(e)))?;

    let bytes: actix_web::web::Bytes = response.body()
        .await
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))?;

    debug!("Response: {:?}", &bytes);

    serde_json::from_slice::<T>(&bytes)
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))
}


pub async fn awc_get_query(
    client: Arc<actix_web::client::Client>,
    route: &str,
    query: serde_json::Value,
) -> StripeResponse<serde_json::Value> {

    /// Serialize the form content using `serde_qs` instead of `serde_urlencoded`
    /// See https://github.com/seanmonstar/reqwest/issues/274
    let query_str = serde_qs::to_string(&query)
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))?;

    let route_with_query = format!("{}?{}", route, query_str);

    let request_url = format!("{}{}", STRIPE_ENDPOINT_URL, route_with_query);
    debug!("GET endpoint: {}", &request_url);

    let mut response = client
        .get(request_url)
        .send()
        .await
        .map_err(|e| StripeError::NetworkError(errJson!(e)))?;

    let bytes = response.body()
        .await
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))?;

    debug!("Response: {:?}", &bytes);

    serde_json::from_slice(&bytes)
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))
}


pub async fn awc_post<T: DeserializeOwned>(
    client: Arc<actix_web::client::Client>,
    route: &str,
) -> StripeResponse<T> {

    let request_url = format!("{}{}", STRIPE_ENDPOINT_URL, route);
    debug!("POST endpoint: {}", &request_url);

    let mut response = client
        .post(request_url)
        .send()
        .await
        .map_err(|e| StripeError::NetworkError(errJson!(e)))?;

    let bytes = response.body()
        .await
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))?;

    debug!("Response: {:?}", &bytes);

    serde_json::from_slice::<T>(&bytes)
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))
}


pub async fn awc_post_body<T, B>(
    client: Arc<actix_web::client::Client>,
    route: &str,
    body: B
) -> StripeResponse<T>
where
    T: DeserializeOwned + Send + 'static,
    B: serde::Serialize + Debug
{

    let request_url = format!("{}{}", STRIPE_ENDPOINT_URL, route);
    debug!("POST endpoint: {}", &request_url);
    debug!("POST body:\n{:?}", &body);

    /// stripe uses "application/x-www-form-urlencoded" content-type,
    ///
    /// NOTE: using .send_form(&body) will automatically urlencode the json body
    /// using the serde_urlencoded lib, but it doesnt handle arrays: ?param[]=card
    /// serde_sq encodes arrays as: ?param[0]=card,
    /// https://github.com/actix/actix-web/issues/1329
    let form_body = serde_qs::to_string(&body)
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))?;

    debug!("encoding as application/x-www-form-urlencoded form:\n{:?}", &form_body);

    let mut response = client
        .post(request_url)
        // .send_form(&body)
        .send_body(&form_body)
        .await
        .map_err(|e| StripeError::NetworkError(errJson!(e)))?;

    let bytes = response.body()
        .await
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))?;

    debug!("Response: {:?}", &bytes);

    serde_json::from_slice::<T>(&bytes)
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))
}


pub async fn awc_delete<T: DeserializeOwned>(
    client: Arc<actix_web::client::Client>,
    route: &str,
) -> StripeResponse<T> {

    let request_url = format!("{}{}", STRIPE_ENDPOINT_URL, route);
    debug!("DELETE endpoint: {}", &request_url);

    let mut response = client
        .delete(request_url)
        .send()
        .await
        .map_err(|e| StripeError::NetworkError(errJson!(e)))?;

    let bytes = response.body()
        .await
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))?;

    debug!("Response: {:?}", &bytes);

    serde_json::from_slice::<T>(&bytes)
        .map_err(|e| StripeError::DeserializationError(errJson!(e)))
}






