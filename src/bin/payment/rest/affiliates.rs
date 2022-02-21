use actix_web::{
    HttpRequest,
    HttpResponse,
    HttpMessage,
    web, web::Query, web::Json,
    Error,
};
use crate::db;
use crate::db::GetPool;
use crate::models::{
    ErrJson,
    CLICK_COOKIE_NAME,
};
use crate::rpc::AffiliateConversionResponse;
use crate::rpc::{
    rpc_get_affiliate_profile_by_click_id,
    rpc_convert_click,
};
use crate::{AppState};



pub async fn convert_buyer_affiliate(
    reqref: &HttpRequest,
    order_id: String
) -> Option<String> {
    // let CLICK_COOKIE_NAME = "gm-affiliate-click"
    match reqref.cookie(CLICK_COOKIE_NAME) {
        None => None,
        Some(buyer_click_cookie) => {

            // gm-affiliate-click=click_c80fcea4-0195-45ff-9c1c-910fca47e769
            let click_id: &str = buyer_click_cookie.value();

            debug!("Converting cookie: {:?}", CLICK_COOKIE_NAME);
            // convert click and retrieve associated userId from affiliate-service
            let maybe_buyer_affiliate: Result<AffiliateConversionResponse, Error> =
                rpc_convert_click(
                    AppState::httpClient(reqref),
                    &click_id,
                    order_id
                ).await;

            // return userId of person who referred this purchase
            match maybe_buyer_affiliate {
                Err(e) => {
                    debug!("Unable to convert click: {:?}", e);
                    None
                },
                Ok(res) => Some(res.user_id),
            }
        }
    }
}
