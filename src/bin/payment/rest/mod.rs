
pub mod stripe;
pub mod paypal;

pub mod affiliate_commissions;
pub mod affiliates;
pub mod create_confirm_payment;
pub mod transactions;
pub mod refunds;
pub mod payment_methods;
pub mod payout_methods;
pub mod payout_items;
pub mod payout_splits;
pub mod payouts;
pub mod health;

pub use affiliate_commissions::*;
pub use affiliates::*;
pub use create_confirm_payment::*;
pub use transactions::*;
pub use refunds::*;
pub use payment_methods::*;
pub use payout_methods::*;
pub use payout_items::*;
pub use payout_splits::*;
pub use payouts::*;
pub use health::*;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaymentProcessor {
    Paypal,
    Stripe,
}
impl PaymentProcessor {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentProcessor::Stripe => "Stripe",
            PaymentProcessor::Paypal => "Paypal",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Stripe" | "stripe" => PaymentProcessor::Stripe,
            "Paypal" | "paypal" | "PayPal" => PaymentProcessor::Paypal,
            _ => PaymentProcessor::Stripe,
        }
    }
}

//////////////////
/// Misc Handlers
//////////////////

pub fn test_handler(
    req: actix_web::HttpRequest
) -> actix_web::HttpResponse {

    debug!("req: {:?}", req);
    debug!("headers: {:?}", req.headers());
    debug!("uri: {:?}", req.uri());

    actix_web::HttpResponse::Ok()
        .content_type("application_json")
        .json(json!({
            "status": "OK",
            "message": "Test response for gm-payment service"
        }))
}

pub fn handle_404(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    // Or use Namedfile to server 404.html pages
    actix_web::HttpResponse::NotFound()
        .json(
            json!({
                "status": 404,
                "reason": "Endpoint not found.",
                "path": format!("{:?}", req.path()),
                "query_string": format!("{:?}", req.query_string()),
                "headers": format!("{:?}", req.headers()),
                "connection_info": format!("{:?}", req.connection_info()),
            })
        )
}



