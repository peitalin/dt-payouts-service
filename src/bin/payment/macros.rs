
use actix_web::Error;
use crate::models::StripeError;
use crate::models::ErrJson;

#[macro_export]
macro_rules! errJson {
    ( $e:expr ) => {
        ErrJson {
            file: format!("{}:{}", file!(), line!()),
            message: $e.to_string(),
        }
    };
}

/// "expand" fields for Stripe
#[macro_export]
macro_rules! expand_as_ref {
    ( $expand:expr ) => {
        $expand.iter()
            .map(String::as_ref)
            .collect::<Vec<&str>>()
    };
}