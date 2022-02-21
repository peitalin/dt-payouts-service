/// Remote Network Calls with REST
use crate::AppState;
use crate::endpoints::Endpoint;
use crate::models::errors::{
    RpcError,
    ErrJson,
};
use std::str::FromStr;
use actix_web::{
    HttpResponse,
    HttpRequest,
    HttpMessage,
    Error,
    web::Query,
};
use crate::models::{
    Affiliate,
    ConversionType,
};



#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliateConversionBody<'a> {
    click_id: &'a str,
    entity_id: &'a str,
    conversion_type: ConversionType,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliateConversionResponse {
    pub user_id: String
}

pub async fn rpc_convert_click(
    client: &actix_web::client::Client,
    click_id: &str,
    // entity which affiliate converted for
    entity_id: String,
) -> Result<AffiliateConversionResponse, Error> {

    let route = format!("/conversion");
    debug!("requesting PUT endpoint: {}", route);

    let mut response = client.put(Endpoint::Affiliate(&route).as_url())
                    .send_json(&AffiliateConversionBody {
                        click_id: click_id,
                        entity_id: &entity_id,
                        conversion_type: ConversionType::ORDER_CONFIRM,
                    })
                    .await?;

    let bytes = response.body().await?;
    debug!("raw response: {:?}", bytes);

    serde_json::from_slice::<AffiliateConversionResponse>(&bytes)
        .map_err(|e| Error::from(RpcError::Affiliate(errJson!(e))))
}


pub async fn rpc_get_affiliate_profile_by_click_id(
    client: &actix_web::client::Client,
    click_id: &str,
) -> Result<Affiliate, Error> {

    // GET: "/affiliate/click/:clickId",
    let route = format!("/affiliate/click/{}", click_id);
    debug!("requesting GET endpoint: {}", route);

    let mut response = client.get(Endpoint::Affiliate(&route).as_url())
                    .send()
                    .await?;

    let bytes = response.body().await?;
    debug!("raw response: {:?}", bytes);

    serde_json::from_slice::<Affiliate>(&bytes)
        .map_err(|e| Error::from(RpcError::Affiliate(errJson!(e))))
}


pub async fn rpc_get_affiliate_profiles_by_user_ids(
    client: &actix_web::client::Client,
    user_ids: &Vec<String>,
) -> Result<Affiliate, Error> {

    // GET: "/affiliates?userIds=u1,u2,u3",
    let route = format!("/affiliates?userIds={}", user_ids.join(","));
    debug!("requesting GET endpoint: {}", route);

    let mut response = client.get(Endpoint::Affiliate(&route).as_url())
                    .send()
                    .await?;

    let bytes = response.body().await?;
    debug!("raw response: {:?}", bytes);

    serde_json::from_slice::<Affiliate>(&bytes)
        .map_err(|e| Error::from(RpcError::Affiliate(errJson!(e))))
}
