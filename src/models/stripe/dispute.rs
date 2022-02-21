// use super::ids::DisputeId;
use super::params::{Expand, Expandable, List, Metadata, Object, RangeQuery, Timestamp};
use super::{BalanceTransaction, Charge, Currency};
use serde_derive::{Deserialize, Serialize};

/// The resource representing a Stripe "Dispute".
///
/// For more details see [https://stripe.com/docs/api/disputes/object](https://stripe.com/docs/api/disputes/object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dispute {
    /// Unique identifier for the object.
    pub id: String,

    /// Disputed amount.
    ///
    /// Usually the amount of the charge, but can differ (usually because of currency fluctuation or because only part of the order is disputed).
    pub amount: i64,

    /// List of zero, one, or two balance transactions that show funds withdrawn and reinstated to your Stripe account as a result of this dispute.
    pub balance_transactions: Vec<BalanceTransaction>,

    /// ID of the charge that was disputed.
    pub charge: Expandable<Charge>,

    /// Time at which the object was created.
    ///
    /// Measured in seconds since the Unix epoch.
    pub created: Timestamp,

    /// Three-letter [ISO currency code](https://www.iso.org/iso-4217-currency-codes.html), in lowercase.
    ///
    /// Must be a [supported currency](https://stripe.com/docs/currencies).
    pub currency: Currency,

    pub evidence: DisputeEvidence,

    pub evidence_details: DisputeEvidenceDetails,

    /// If true, it is still possible to refund the disputed payment.
    ///
    /// Once the payment has been fully refunded, no further funds will be withdrawn from your Stripe account as a result of this dispute.
    pub is_charge_refundable: bool,

    /// Has the value `true` if the object exists in live mode or the value `false` if the object exists in test mode.
    pub livemode: bool,

    /// Set of key-value pairs that you can attach to an object.
    ///
    /// This can be useful for storing additional information about the object in a structured format.
    pub metadata: Metadata,

    /// Reason given by cardholder for dispute.
    ///
    /// Possible values are `bank_cannot_process`, `check_returned`, `credit_not_processed`, `customer_initiated`, `debit_not_authorized`, `duplicate`, `fraudulent`, `general`, `incorrect_account_details`, `insufficient_funds`, `product_not_received`, `product_unacceptable`, `subscription_canceled`, or `unrecognized`.
    /// Read more about [dispute reasons](https://stripe.com/docs/disputes/categories).
    pub reason: String,

    /// Current status of dispute.
    ///
    /// Possible values are `warning_needs_response`, `warning_under_review`, `warning_closed`, `needs_response`, `under_review`, `charge_refunded`, `won`, or `lost`.
    pub status: DisputeStatus,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DisputeEvidence {
    /// Any server or activity logs showing proof that the customer accessed or downloaded the purchased digital product.
    ///
    /// This information should include IP addresses, corresponding timestamps, and any detailed recorded activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_activity_log: Option<String>,

    /// The billing address provided by the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_address: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) Your subscription cancellation policy, as shown to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub cancellation_policy: Option<Expandable<File>>,
    pub cancellation_policy: Option<String>,

    /// An explanation of how and when the customer was shown your refund policy prior to purchase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation_policy_disclosure: Option<String>,

    /// A justification for why the customer's subscription was not canceled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation_rebuttal: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) Any communication with the customer that you feel is relevant to your case.
    ///
    /// Examples include emails proving that the customer received the product or service, or demonstrating their use of or satisfaction with the product or service.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub customer_communication: Option<Expandable<File>>,
    pub customer_communication: Option<String>,

    /// The email address of the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_email_address: Option<String>,

    /// The name of the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_name: Option<String>,

    /// The IP address that the customer used when making the purchase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_purchase_ip: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) A relevant document or contract showing the customer's signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub customer_signature: Option<Expandable<File>>,
    pub customer_signature: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) Documentation for the prior charge that can uniquely identify the charge, such as a receipt, shipping label, work order, etc.
    ///
    /// This document should be paired with a similar document from the disputed payment that proves the two payments are separate.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub duplicate_charge_documentation: Option<Expandable<File>>,
    pub duplicate_charge_documentation: Option<String>,

    /// An explanation of the difference between the disputed charge versus the prior charge that appears to be a duplicate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_charge_explanation: Option<String>,

    /// The Stripe ID for the prior charge which appears to be a duplicate of the disputed charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_charge_id: Option<String>,

    /// A description of the product or service that was sold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_description: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) Any receipt or message sent to the customer notifying them of the charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub receipt: Option<Expandable<File>>,
    pub receipt: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) Your refund policy, as shown to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub refund_policy: Option<Expandable<File>>,
    pub refund_policy: Option<String>,

    /// Documentation demonstrating that the customer was shown your refund policy prior to purchase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_policy_disclosure: Option<String>,

    /// A justification for why the customer is not entitled to a refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_refusal_explanation: Option<String>,

    /// The date on which the customer received or began receiving the purchased service, in a clear human-readable format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_date: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) Documentation showing proof that a service was provided to the customer.
    ///
    /// This could include a copy of a signed contract, work order, or other form of written agreement.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub service_documentation: Option<Expandable<File>>,
    pub service_documentation: Option<String>,

    /// The address to which a physical product was shipped.
    ///
    /// You should try to include as complete address information as possible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<String>,

    /// The delivery service that shipped a physical product, such as Fedex, UPS, USPS, etc.
    ///
    /// If multiple carriers were used for this purchase, please separate them with commas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_carrier: Option<String>,

    /// The date on which a physical product began its route to the shipping address, in a clear human-readable format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_date: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) Documentation showing proof that a product was shipped to the customer at the same address the customer provided to you.
    ///
    /// This could include a copy of the shipment receipt, shipping label, etc.
    /// It should show the customer's full shipping address, if possible.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub shipping_documentation: Option<Expandable<File>>,
    pub shipping_documentation: Option<String>,

    /// The tracking number for a physical product, obtained from the delivery service.
    ///
    /// If multiple tracking numbers were generated for this purchase, please separate them with commas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_tracking_number: Option<String>,

    /// (ID of a [file upload](https://stripe.com/docs/guides/file-upload)) Any additional evidence or statements.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub uncategorized_file: Option<Expandable<File>>,
    pub uncategorized_file: Option<String>,

    /// Any additional evidence or statements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncategorized_text: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DisputeEvidenceDetails {
    /// Date by which evidence must be submitted in order to successfully challenge dispute.
    ///
    /// Will be null if the customer's bank or credit card company doesn't allow a response for this particular dispute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_by: Option<Timestamp>,

    /// Whether evidence has been staged for this dispute.
    pub has_evidence: bool,

    /// Whether the last evidence submission was submitted past the due date.
    ///
    /// Defaults to `false` if no evidence submissions have occurred.
    /// If `true`, then delivery of the latest evidence is *not* guaranteed.
    pub past_due: bool,

    /// The number of times evidence has been submitted.
    ///
    /// Typically, you may only submit evidence once.
    pub submission_count: u64,
}

/// The parameters for `Dispute::list`.
#[derive(Clone, Debug, Serialize)]
pub struct ListDisputes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<RangeQuery<Timestamp>>,

    /// A cursor for use in pagination.
    ///
    /// `ending_before` is an object ID that defines your place in the list.
    /// For instance, if you make a list request and receive 100 objects, starting with `obj_bar`, your subsequent call can include `ending_before=obj_bar` in order to fetch the previous page of the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub ending_before: Option<DisputeId>,
    pub ending_before: Option<String>,

    // /// Specifies which fields in the response should be expanded.
    // #[serde(skip_serializing_if = "Expand::is_empty")]
    // pub expand: Vec<String>,

    /// A limit on the number of objects to be returned.
    ///
    /// Limit can range between 1 and 100, and the default is 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// A cursor for use in pagination.
    ///
    /// `starting_after` is an object ID that defines your place in the list.
    /// For instance, if you make a list request and receive 100 objects, ending with `obj_foo`, your subsequent call can include `starting_after=obj_foo` in order to fetch the next page of the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub starting_after: Option<DisputeId>,
    pub starting_after: Option<String>,
}

impl ListDisputes {
    pub fn new() -> Self {
        ListDisputes {
            created: Default::default(),
            ending_before: Default::default(),
            // expand: Default::default(),
            limit: Default::default(),
            starting_after: Default::default(),
        }
    }
}

/// An enum representing the possible values of an `Dispute`'s `status` field.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DisputeStatus {
    ChargeRefunded,
    Lost,
    NeedsResponse,
    UnderReview,
    WarningClosed,
    WarningNeedsResponse,
    WarningUnderReview,
    Won,
}

impl DisputeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DisputeStatus::ChargeRefunded => "charge_refunded",
            DisputeStatus::Lost => "lost",
            DisputeStatus::NeedsResponse => "needs_response",
            DisputeStatus::UnderReview => "under_review",
            DisputeStatus::WarningClosed => "warning_closed",
            DisputeStatus::WarningNeedsResponse => "warning_needs_response",
            DisputeStatus::WarningUnderReview => "warning_under_review",
            DisputeStatus::Won => "won",
        }
    }
}

impl AsRef<str> for DisputeStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for DisputeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}
