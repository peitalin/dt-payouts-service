use super::errors::ErrorCode;
use super::ids::{ChargeId, CustomerId};
use super::params::{Expand, Expandable, List, Metadata, Object, RangeQuery, Timestamp};
use super::{
    Account, BalanceTransaction, BillingDetails,
    Currency, Customer, Dispute, Invoice,
    PaymentMethodDetails, PaymentSource, PaymentSourceParams, Refund, Shipping,
    //// non-imported
    // Application,
    // ApplicationFee,
    // Order,
    // Review,
    // Transfer,
};
use serde_derive::{Deserialize, Serialize};

/// The resource representing a Stripe "Charge".
///
/// For more details see [https://stripe.com/docs/api/charges/object](https://stripe.com/docs/api/charges/object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Charge {
    /// Unique identifier for the object.
    pub id: ChargeId,

    /// A positive integer representing how much to charge in the [smallest currency unit](https://stripe.com/docs/currencies#zero-decimal) (e.g., 100 cents to charge $1.00 or 100 to charge ¥100, a zero-decimal currency).
    ///
    /// The minimum amount is $0.50 US or [equivalent in charge currency](https://support.stripe.com/questions/what-is-the-minimum-amount-i-can-charge-with-stripe).
    pub amount: i64,

    /// Amount in %s refunded (can be less than the amount attribute on the charge if a partial refund was issued).
    pub amount_refunded: i64,

    /// ID of the Connect application that created the charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub application: Option<Application>,
    pub application: Option<String>,

    /// The application fee (if any) for the charge.
    ///
    /// [See the Connect documentation](https://stripe.com/docs/connect/direct-charges#collecting-fees) for details.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub application_fee: Option<ApplicationFee>,
    pub application_fee: Option<String>,

    /// The amount of the application fee (if any) for the charge.
    ///
    /// [See the Connect documentation](https://stripe.com/docs/connect/direct-charges#collecting-fees) for details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_fee_amount: Option<i64>,

    /// ID of the balance transaction that describes the impact of this charge on your account balance (not including refunds or disputes).
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub balance_transaction: Option<BalanceTransaction>,
    pub balance_transaction: Option<String>,
    //e.g "txn_1ExEnSKqy1M9WH1DD5a4ojHp"

    pub billing_details: BillingDetails,

    /// If the charge was created without capturing, this Boolean represents whether it is still uncaptured or has since been captured.
    pub captured: bool,

    /// Time at which the object was created.
    ///
    /// Measured in seconds since the Unix epoch.
    pub created: Timestamp,

    /// Three-letter [ISO currency code](https://www.iso.org/iso-4217-currency-codes.html), in lowercase.
    ///
    /// Must be a [supported currency](https://stripe.com/docs/currencies).
    pub currency: Currency,

    /// ID of the customer this charge is for if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub customer: Option<Customer>,
    pub customer: Option<String>,

    /// An arbitrary string attached to the object.
    ///
    /// Often useful for displaying to users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Details about the dispute if the charge has been disputed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispute: Option<Dispute>,

    /// Error code explaining reason for charge failure if available (see [the errors section](https://stripe.com/docs/api#errors) for a list of codes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<ErrorCode>,

    /// Message to user further explaining reason for charge failure if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,

    /// Information on fraud assessments for the charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fraud_details: Option<FraudDetails>,

    /// ID of the invoice this charge is for if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<Invoice>,

    /// Has the value `true` if the object exists in live mode or the value `false` if the object exists in test mode.
    pub livemode: bool,

    /// Set of key-value pairs that you can attach to an object.
    ///
    /// This can be useful for storing additional information about the object in a structured format.
    pub metadata: Metadata,

    /// The account (if any) the charge was made on behalf of without triggering an automatic transfer.
    ///
    /// See the [Connect documentation](https://stripe.com/docs/connect/charges-transfers) for details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<Account>,

    /// ID of the order this charge is for if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub order: Option<Order>,
    pub order: Option<String>,

    /// Details about whether the payment was accepted, and why.
    ///
    /// See [understanding declines](https://stripe.com/docs/declines) for details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<ChargeOutcome>,

    /// `true` if the charge succeeded, or was successfully authorized for later capture.
    pub paid: bool,

    /// ID of the PaymentIntent associated with this charge, if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_intent: Option<String>,

    /// ID of the payment method used in this charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<String>,

    /// Details about the payment method at the time of the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_details: Option<PaymentMethodDetails>,

    /// This is the email address that the receipt for this charge was sent to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_email: Option<String>,

    /// This is the transaction number that appears on email receipts sent for this charge.
    ///
    /// This attribute will be `null` until a receipt has been sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_number: Option<String>,

    /// This is the URL to view the receipt for this charge.
    ///
    /// The receipt is kept up-to-date to the latest state of the charge, including any refunds.
    /// If the charge is for an Invoice, the receipt will be stylized as an Invoice receipt.
    pub receipt_url: String,

    /// Whether the charge has been fully refunded.
    ///
    /// If the charge is only partially refunded, this attribute will still be false.
    pub refunded: bool,

    /// A list of refunds that have been applied to the charge.
    pub refunds: List<Refund>,

    /// ID of the review associated with this charge if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub review: Option<Review>,
    pub review: Option<String>,

    /// Shipping information for the charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<Shipping>,

    /// For most Stripe users, the source of every charge is a credit or debit card.
    ///
    /// This hash is then the [card object](#card_object) describing that card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PaymentSource>,

    /// The transfer ID which created this charge.
    ///
    /// Only present if the charge came from another Stripe account.
    /// [See the Connect documentation](https://stripe.com/docs/connect/destination-charges) for details.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub source_transfer: Option<Transfer>,
    pub source_transfer: Option<String>,

    /// Extra information about a charge.
    ///
    /// This will appear on your customer's credit card statement.
    /// It must contain at least one letter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement_descriptor: Option<String>,

    /// The status of the payment is either `succeeded`, `pending`, or `failed`.
    pub status: String,

    /// ID of the transfer to the `destination` account (only applicable if the charge was created using the `destination` parameter).
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub transfer: Option<Transfer>,
    pub transfer: Option<String>,

    /// An optional dictionary including the account to automatically transfer to as part of a destination charge.
    ///
    /// [See the Connect documentation](https://stripe.com/docs/connect/destination-charges) for details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_data: Option<TransferData>,

    /// A string that identifies this transaction as part of a group.
    ///
    /// See the [Connect documentation](https://stripe.com/docs/connect/charges-transfers#grouping-transactions) for details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_group: Option<String>,
}


impl Object for Charge {
    type Id = ChargeId;
    fn id(&self) -> Self::Id {
        self.id.clone()
    }
    fn object(&self) -> &'static str {
        "charge"
    }
}

/// The resource representing a Stripe charge object outcome.
///
/// For more details see [https://stripe.com/docs/api#charge_object-outcome](https://stripe.com/docs/api#charge_object-outcome)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChargeOutcome {
    #[serde(rename = "type")]
    pub outcome_type: String,
    pub network_status: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub risk_level: Option<String>,
    #[serde(default)]
    pub seller_message: Option<String>,
    #[serde(default)]
    pub rule: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FraudDetails {
    pub user_report: Option<String>,
    #[serde(skip_serializing)]
    pub stripe_report: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransferData {
    /// The amount transferred to the destination account, if specified.
    ///
    /// By default, the entire charge amount is transferred to the destination account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,

    /// ID of an existing, connected Stripe account to transfer funds to if `transfer_data` was specified in the charge request.
    pub destination: Account,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DestinationParams {
    pub account: String,
    pub amount: u64,
}

