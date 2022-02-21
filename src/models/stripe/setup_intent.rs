
use serde_derive::{Deserialize, Serialize};
use super::ids::{CustomerId};
use super::params::{Expand, Expandable, List, Metadata, Object, RangeQuery, Timestamp};
use super::{
    Account, Charge, Currency, Customer, Invoice, PaymentMethod, PaymentSource,
    Shipping,
    Card,
    TransferData,
};


/// The resource representing a Stripe "SetupIntent".
/// For more details see [https://stripe.com/docs/api/setup_intents/object](https://stripe.com/docs/api/setup_intents/object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SetupIntent {
    /// Unique identifier for the object.
    pub id: String,
    pub object: String,
    pub application: Option<String>,
    pub cancellation_reason: Option<String>,
    pub client_secret: Option<String>,
    pub created: Option<Timestamp>,
    pub customer: Option<String>,
    pub description: Option<String>,
    pub last_setup_error: Option<String>,
    /// Has the value `true` if the object exists in live mode or the value `false` if the object exists in test mode.
    pub livemode: bool,
    #[serde(default)]
    pub metadata: Option<Metadata>,
    pub next_action: Option<String>,
    // pub on_behalf_of: Option<Account>,
    pub on_behalf_of: Option<String>,
    pub payment_method: Option<String>,
    // pub payment_method_options: Option<Card>,
    pub payment_method_options: Option<SetupIntentCard>,
    pub payment_method_types: Option<Vec<String>>,
    pub status: String,
    pub usage: String,
}


/// The set of parameters that can be used when creating a setup_intent object.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SetupIntentCreateParams {

    /// Set to true to attempt to confirm this SetupIntent immediately.
    /// This parameter defaults to false. If the payment method attached
    /// is a card, a return_url may be provided in case additional
    /// authentication is required.
    pub confirm: Option<bool>,

    /// ID of the Customer this SetupIntent belongs to, if one exists.
    /// If present, payment methods used with this SetupIntent can
    /// only be attached to this Customer, and payment methods attached
    /// to other Customers cannot be used with this SetupIntent.
    pub customer: Option<String>,

    /// An arbitrary string attached to the object.
    /// Often useful for displaying to users. This will be unset
    /// if you POST an empty value.
    pub description: Option<String>,

    pub metadata: Option<Metadata>,

    /// The Stripe account ID for which this SetupIntent is created.
    pub on_behalf_of: Option<String>,

    /// The Stripe account ID for which this SetupIntent is created.
    pub payment_method: Option<String>,

    /// Payment-method-specific configuration for this SetupIntent.
    pub payment_method_options: Option<Card>,

    /// The list of payment method types that this SetupIntent is allowed
    /// to set up. If this is not provided, defaults to [“card”].
    /// Valid payment method types include: card and card_present.
    pub payment_method_types: Option<Vec<String>>,

    /// return url
    pub return_url: Option<String>,

    /// Indicates how the payment method is intended to be used in the future.
    /// Use on_session if you intend to only reuse the payment method when
    /// the customer is in your checkout flow. Use off_session if
    /// your customer may or may not be in your checkout flow.
    /// If not provided, this value defaults to off_session.
    pub usage: Option<String>,
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SetupIntentUpdateParams {
    /// ID of the SetupIntent to retrieve.
    pub intent: String,

    /// ID of the Customer this SetupIntent belongs to, if one exists.
    pub customer: Option<String>,

    /// An arbitrary string attached to the object.
    pub description: Option<String>,

    pub metadata: Option<Metadata>,

    /// The Stripe account ID for which this SetupIntent is created.
    pub payment_method: Option<String>,

    /// The list of payment method types that this SetupIntent is allowed
    /// to set up. If this is not provided, defaults to [“card”].
    pub payment_method_types: Option<Vec<String>>,
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SetupIntentConfirmParams {
    /// ID of the SetupIntent to retrieve.
    pub intent: String,

    /// The Stripe account ID for which this SetupIntent is created.
    pub payment_method: Option<String>,

    /// Payment-method-specific configuration for this SetupIntent.
    pub payment_method_options: Option<SetupIntentCard>,

    /// return url
    pub return_url: Option<String>,
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SetupIntentCancelParams {
    /// ID of the SetupIntent to retrieve.
    pub intent: String,

    /// Reason for canceling this SetupIntent. Possible values
    /// are abandoned, requested_by_customer, or duplicate
    pub cancellation_reason: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SetupIntentCard {
    pub card: Request3dSecure
    // pub card: Card
}
/// if you wish to request 3D Secure based on logic from your own fraud engine,
/// provide this option. Permitted values include: automatic or any.
/// If not provided, defaults to automatic. Read our guide on manually
/// requesting 3D Secure for more information on how this configuration
/// interacts with Radar and our SCA Engine.

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Request3dSecure {
    pub request_three_d_secure: Option<String>,
}



