// Internal Imports
use actix_web::{
    http::StatusCode,
    error::ResponseError,
    HttpResponse,
};
use futures::{future, future::Future};
use failure::Error;


#[derive(Debug, Clone, Serialize, Deserialize, Fail)]
pub struct ErrJson {
    pub file: String,
    pub message: String,
}
impl std::fmt::Display for ErrJson {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "{:?}",
            serde_json::to_string(&self).unwrap_or(self.message.clone())
        )
    }
}
impl ErrJson {
    pub fn new(message: &str) -> Self {
        Self {
            file: format!("{}:{}", file!(), line!()),
            message: String::from(message),
        }
    }
}


#[derive(Debug, Serialize, Fail)]
pub enum StripeError {
    #[fail(display = "{}", _0)]
    PaymentIntent(ErrJson),
    #[fail(display = "{}", _0)]
    SetupIntent(ErrJson),
    #[fail(display = "{}", _0)]
    PaymentMethod(ErrJson),
    #[fail(display = "{}", _0)]
    Customer(ErrJson),
    #[fail(display = "{}", _0)]
    Refund(ErrJson),
    #[fail(display = "{}", _0)]
    IdPrefix(ErrJson),
    #[fail(display = "{}", _0)]
    NetworkError(ErrJson),
    #[fail(display = "{}", _0)]
    DeserializationError(ErrJson),
}

impl ResponseError for StripeError {
    // error_response => json
    fn error_response(&self) -> HttpResponse {
       match self {
            StripeError::PaymentIntent(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            StripeError::SetupIntent(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            StripeError::PaymentMethod(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            StripeError::Customer(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            StripeError::Refund(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            StripeError::IdPrefix(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            StripeError::NetworkError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            StripeError::DeserializationError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
       }
    }
}

impl From<serde_json::Error> for StripeError {
    fn from(e: serde_json::Error) -> Self {
        StripeError::DeserializationError(errJson!(e))
    }
}


#[derive(Debug, Serialize, Fail)]
pub enum PaypalError {
    #[fail(display = "{}", _0)]
    BadInput(ErrJson),
    #[fail(display = "{}", _0)]
    NetworkError(ErrJson),
    #[fail(display = "{}", _0)]
    InternalError(ErrJson),
    #[fail(display = "{}", _0)]
    DeserializationError(ErrJson),
    #[fail(display = "{}", _0)]
    InsufficientFunds(ErrJson),
    #[fail(display = "{}", _0)]
    ValidationError(ErrJson),
    #[fail(display = "{}", _0)]
    DuplicateTransaction(ErrJson),
}

impl From<std::str::Utf8Error> for PaypalError {
    fn from(e: std::str::Utf8Error) -> Self {
        PaypalError::DeserializationError(errJson!(e))
    }
}

impl ResponseError for PaypalError {
    // error_response => json
    fn error_response(&self) -> HttpResponse {
       match self {
            PaypalError::BadInput(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::BAD_REQUEST)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            PaypalError::NetworkError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            PaypalError::InternalError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            PaypalError::DeserializationError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            PaypalError::InsufficientFunds(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            PaypalError::ValidationError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            PaypalError::DuplicateTransaction(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::BAD_REQUEST)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
       }
    }
}

#[derive(Debug, Serialize, Fail)]
pub enum BraintreeError {
    #[fail(display = "{}", _0)]
    InvalidCredentials(ErrJson),
    #[fail(display = "{}", _0)]
    InternalError(ErrJson),
}

impl ResponseError for BraintreeError {
    fn error_response(&self) -> HttpResponse {
       match self {
            BraintreeError::InvalidCredentials(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::BAD_REQUEST)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            BraintreeError::InternalError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
       }
    }
}



#[derive(Debug, Serialize, Deserialize, Fail)]
pub enum DbError {
    #[fail(display = "{}", _0)]
    TransactionWriteError(ErrJson),
    #[fail(display = "{}", _0)]
    TransactionReadError(ErrJson),
    #[fail(display = "{}", _0)]
    PayoutItemWriteError(ErrJson),
    #[fail(display = "{}", _0)]
    PayoutItemReadError(ErrJson),
    #[fail(display = "{}", _0)]
    PayoutWriteError(ErrJson),
    #[fail(display = "{}", _0)]
    PayoutReadError(ErrJson),
    #[fail(display = "{}", _0)]
    RefundWriteError(ErrJson),
    #[fail(display = "{}", _0)]
    RefundReadError(ErrJson),
    #[fail(display = "{}", _0)]
    PaymentMethodWriteError(ErrJson),
    #[fail(display = "{}", _0)]
    PaymentMethodDeleteError(ErrJson),
    #[fail(display = "{}", _0)]
    PaymentMethodReadError(ErrJson),
    #[fail(display = "{}", _0)]
    PaymentMethodAddressWriteError(ErrJson),
    #[fail(display = "{}", _0)]
    PaymentMethodAddressReadError(ErrJson),
    #[fail(display = "{}", _0)]
    PayoutMethodWriteError(ErrJson),
    #[fail(display = "{}", _0)]
    PayoutMethodReadError(ErrJson),
    #[fail(display = "{}", _0)]
    PayoutSplitWriteError(ErrJson),
    #[fail(display = "{}", _0)]
    PayoutSplitReadError(ErrJson),
    #[fail(display = "{}", _0)]
    PoolError(ErrJson),
}

impl ResponseError for DbError {
    fn error_response(&self) -> HttpResponse {
       match self {
            DbError::TransactionWriteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::TransactionReadError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PayoutItemWriteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PayoutItemReadError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PayoutWriteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PayoutReadError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::RefundWriteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::RefundReadError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PaymentMethodWriteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PaymentMethodDeleteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PaymentMethodReadError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PaymentMethodAddressWriteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PaymentMethodAddressReadError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PayoutMethodWriteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PayoutMethodReadError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PayoutSplitWriteError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PayoutSplitReadError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            DbError::PoolError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
       }
    }
}


#[derive(Debug, Fail, Serialize, Deserialize)]
pub enum RpcError {
    #[fail(display = "{}", _0)]
    Shopping(ErrJson),
    #[fail(display = "{}", _0)]
    Customer(ErrJson),
    #[fail(display = "{}", _0)]
    User(ErrJson),
    #[fail(display = "{}", _0)]
    Affiliate(ErrJson),
}

impl ResponseError for RpcError {
    fn error_response(&self) -> HttpResponse {
       match self {
            RpcError::Shopping(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            RpcError::Customer(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            RpcError::User(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            }
            RpcError::Affiliate(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            }
       }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PaginateError {
    b64DecodeError(ErrJson),
    InvalidCursor(ErrJson),
}

impl ResponseError for PaginateError {
    fn error_response(&self) -> HttpResponse {
       match self {
            PaginateError::b64DecodeError(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
            PaginateError::InvalidCursor(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            }
       }
    }
}

impl From<base64::DecodeError> for PaginateError {
    fn from(e: base64::DecodeError) -> Self {
        match e {
            base64::DecodeError::InvalidByte(_u, _r) =>
                PaginateError::b64DecodeError(ErrJson::new("err decoding b64 cursor")),
            base64::DecodeError::InvalidLength =>
                PaginateError::b64DecodeError(ErrJson::new("err decoding b64 cursor")),
            base64::DecodeError::InvalidLastSymbol(_u, _r) =>
                PaginateError::b64DecodeError(ErrJson::new("err decoding b64 cursor")),
        }
    }
}

impl From<std::str::Utf8Error> for PaginateError {
    fn from(_e: std::str::Utf8Error) -> Self {
        PaginateError::b64DecodeError(ErrJson::new("err decoding Utf8 b64 cursor"))
    }
}

impl std::fmt::Display for PaginateError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str(&format!("PaginateError: {:?}", self))
    }
}

impl std::error::Error for PaginateError {
    fn description(&self) -> &str {
        "Error decoding b64 cursor for pagination!"
    }
}

#[derive(Debug, Serialize, Deserialize, Fail)]
pub enum AuthError {
    #[fail(display = "{}", _0)]
    NotWorthyEnough(ErrJson),
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
       match self {
            AuthError::NotWorthyEnough(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
       }
    }
}

#[derive(Debug, Serialize, Deserialize, Fail)]
pub enum MonthError {
    #[fail(display = "{}", _0)]
    ImpossibruMonth(ErrJson),
}

impl ResponseError for MonthError {
    fn error_response(&self) -> HttpResponse {
       match self {
            MonthError::ImpossibruMonth(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
       }
    }
}


#[derive(Debug, Serialize, Deserialize, Fail)]
pub enum AffiliateError {
    #[fail(display = "{}", _0)]
    CreateSellerAffiliateManually(ErrJson),
}

impl ResponseError for AffiliateError {
    fn error_response(&self) -> HttpResponse {
       match self {
            AffiliateError::CreateSellerAffiliateManually(ejson) => {
                warn!("{}: {}", ejson.file, ejson.message);
                HttpResponse::build(StatusCode::BAD_REQUEST)
                    .json(json!({ "file": ejson.file, "message": ejson.message }))
            },
       }
    }
}