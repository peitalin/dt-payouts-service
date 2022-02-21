use super::ids::{
    // AlipayAccountId, BankAccountId, CardId, CouponId,
    CustomerId, PaymentMethodId, PaymentSourceId,
};
use super::params::{Deleted, Expand, Expandable, List, Metadata, Object, RangeQuery, Timestamp};
use super::{
    Address,
    Currency, CustomField,
    TaxId,
    PaymentMethod, PaymentSource, PaymentSourceParams,
    Scheduled, Shipping, ShippingParams, Subscription,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;



/// The resource representing a Stripe "Customer".
/// For more details see [https://stripe.com/docs/api/customers/object](https://stripe.com/docs/api/customers/object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Customer {
    /// Unique identifier for the object.
    pub id: CustomerId,

    pub balance: i32,

    /// Current balance, if any, being stored on the customer's account.
    ///
    /// If negative, the customer has credit to apply to the next invoice.
    /// If positive, the customer has an amount owed that will be added to the next invoice.
    /// The balance does not refer to any unpaid invoices; it solely takes into account amounts that have yet to be successfully applied to any invoice.
    /// This balance is only taken into account as invoices are finalized.
    /// Note that the balance does not include unpaid invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_balance: Option<u64>,

    /// The customer's address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,

    /// Time at which the object was created.
    ///
    /// Measured in seconds since the Unix epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<Timestamp>,

    /// Three-letter [ISO code for the currency](https://stripe.com/docs/currencies) the customer can be charged in for recurring billing purposes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<Currency>,

    /// ID of the default payment source for the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_source: Option<Expandable<PaymentSource>>,

    // Always true for a deleted object
    #[serde(default)]
    pub deleted: bool,

    /// When the customer's latest invoice is billed by charging automatically, delinquent is true if the invoice's latest charge is failed.
    ///
    /// When the customer's latest invoice is billed by sending an invoice, delinquent is true if the invoice is not paid by its due date.
    #[serde(default)]
    pub delinquent: bool,

    /// An arbitrary string attached to the object.
    ///
    /// Often useful for displaying to users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Describes the current discount active on the customer, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<String>,

    /// The customer's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// The prefix for the customer used to generate unique invoice numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_prefix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_settings: Option<InvoiceSettingCustomerSetting>,

    /// Has the value `true` if the object exists in live mode or the value `false` if the object exists in test mode.
    #[serde(default)]
    pub livemode: bool,

    /// Set of key-value pairs that you can attach to an object.
    ///
    /// This can be useful for storing additional information about the object in a structured format.
    #[serde(default)]
    pub metadata: Metadata,

    /// The customer's full name or business name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The customer's phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// The customer's preferred locales (languages), ordered by preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_locales: Option<Vec<String>>,

    /// Mailing and shipping address for the customer.
    ///
    /// Appears on invoices emailed to this customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<Shipping>,

    /// The customer's payment sources, if any.
    #[serde(default)]
    pub sources: List<PaymentSource>,

    /// The customer's current subscriptions, if any.
    #[serde(default)]
    pub subscriptions: List<Subscription>,

    /// Describes the customer's tax exemption status.
    ///
    /// One of `none`, `exempt`, or `reverse`.
    /// When set to `reverse`, invoice and receipt PDFs include the text **"Reverse charge"**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exempt: Option<CustomerTaxExempt>,

    /// The customer's tax IDs.
    #[serde(default)]
    pub tax_ids: List<TaxId>,

    /// The customer's tax information.
    ///
    /// Appears on invoices emailed to this customer.
    /// This field has been deprecated and will be removed in a future API version, for further information view the [migration guide](https://stripe.com/docs/billing/migration/taxes#moving-from-taxinfo-to-customer-tax-ids).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_info: Option<TaxInfo>,

    /// Describes the status of looking up the tax ID provided in `tax_info`.
    ///
    /// This field has been deprecated and will be removed in a future API version, for further information view the [migration guide](https://stripe.com/docs/billing/migration/taxes#moving-from-taxinfo-to-customer-tax-ids).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_info_verification: Option<TaxInfoVerification>,
}



impl Object for Customer {
    type Id = CustomerId;
    fn id(&self) -> Self::Id {
        self.id.clone()
    }
    fn object(&self) -> &'static str {
        "customer"
    }
}



/// The parameters for `Customer::create`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerCreateParams {
    /// An integer amount in %s that represents the account balance for your customer.
    ///
    /// Account balances only affect invoices.
    /// A negative amount represents a credit that decreases the amount due on an invoice; a positive amount increases the amount due on an invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_balance: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<u64>,

    /// The customer's address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,

    #[serde(skip_serializing_if = "Option::is_none")]
    // pub coupon: Option<CouponId>,
    pub coupon: Option<String>,

    /// An arbitrary string that you can attach to a customer object.
    ///
    /// It is displayed alongside the customer in the dashboard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Customer's email address.
    ///
    /// It's displayed alongside the customer in your dashboard and can be useful for searching and tracking.
    /// This may be up to *512 characters*.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    // /// Specifies which fields in the response should be expanded.
    // #[serde(skip_serializing_if = "Expand::is_empty")]
    // pub expand: Vec<String>,

    /// The prefix for the customer used to generate unique invoice numbers.
    ///
    /// Must be 3–12 uppercase letters or numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_prefix: Option<String>,

    /// Default invoice settings for this customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_settings: Option<CustomerInvoiceSettings>,

    /// A set of key-value pairs that you can attach to a customer object.
    ///
    /// It can be useful for storing additional information about the customer in a structured format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    /// The customer's full name or business name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<PaymentMethodId>,

    /// The customer's phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Customer's preferred languages, ordered by preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_locales: Option<Vec<String>>,

    /// The customer's shipping information.
    ///
    /// Appears on invoices emailed to this customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<ShippingParams>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// The customer's tax exemption.
    ///
    /// One of `none`, `exempt`, or `reverse`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exempt: Option<CustomerTaxExemptFilter>,

    /// The customer's tax IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id_data: Option<Vec<TaxIdData>>,

    /// The customer's tax information.
    ///
    /// Appears on invoices emailed to this customer.
    /// This parameter has been deprecated and will be removed in a future API version, for further information view the [migration guide](https://stripe.com/docs/billing/migration/taxes#moving-from-taxinfo-to-customer-tax-ids).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_info: Option<TaxInfoParams>,
}

impl CustomerCreateParams {
    pub fn new() -> Self {
        CustomerCreateParams {
            account_balance: Default::default(),
            balance: Default::default(),
            address: Default::default(),
            coupon: Default::default(),
            description: Default::default(),
            email: Default::default(),
            // expand: Default::default(),
            invoice_prefix: Default::default(),
            invoice_settings: Default::default(),
            metadata: Default::default(),
            name: Default::default(),
            payment_method: Default::default(),
            phone: Default::default(),
            preferred_locales: Default::default(),
            shipping: Default::default(),
            source: Default::default(),
            tax_exempt: Default::default(),
            tax_id_data: Default::default(),
            tax_info: Default::default(),
        }
    }
}
impl Default for CustomerCreateParams {
    fn default() -> Self {
        CustomerCreateParams {
            account_balance: Default::default(),
            balance: Default::default(),
            address: Default::default(),
            coupon: Default::default(),
            description: Default::default(),
            email: Default::default(),
            // expand: Default::default(),
            invoice_prefix: Default::default(),
            invoice_settings: Default::default(),
            metadata: Default::default(),
            name: Default::default(),
            payment_method: Default::default(),
            phone: Default::default(),
            preferred_locales: Default::default(),
            shipping: Default::default(),
            source: Default::default(),
            tax_exempt: Default::default(),
            tax_id_data: Default::default(),
            tax_info: Default::default(),
        }
    }
}


/// The parameters for `Customer::list`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<RangeQuery<Timestamp>>,

    /// A filter on the list based on the customer's `email` field.
    ///
    /// The value must be a string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// A cursor for use in pagination.
    ///
    /// `ending_before` is an object ID that defines your place in the list.
    /// For instance, if you make a list request and receive 100 objects, starting with `obj_bar`, your subsequent call can include `ending_before=obj_bar` in order to fetch the previous page of the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending_before: Option<CustomerId>,

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
    pub starting_after: Option<CustomerId>,
}

impl CustomerListParams {
    pub fn new() -> Self {
        CustomerListParams {
            created: Default::default(),
            email: Default::default(),
            ending_before: Default::default(),
            // expand: Default::default(),
            limit: Default::default(),
            starting_after: Default::default(),
        }
    }
}


/// The parameters for `Customer::update`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerUpdateParams {
    /// An integer amount in %s that represents the account balance for your customer.
    ///
    /// Account balances only affect invoices.
    /// A negative amount represents a credit that decreases the amount due on an invoice; a positive amount increases the amount due on an invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_balance: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<u64>,

    /// The customer's address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,

    #[serde(skip_serializing_if = "Option::is_none")]
    // pub coupon: Option<CouponId>,
    pub coupon: Option<String>,

    /// ID of Alipay account to make the customer's new default for invoice payments.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub default_alipay_account: Option<AlipayAccountId>,
    pub default_alipay_account: Option<String>,

    /// ID of bank account to make the customer's new default for invoice payments.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub default_bank_account: Option<BankAccountId>,
    pub default_bank_account: Option<String>,

    /// ID of card to make the customer's new default for invoice payments.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub default_card: Option<CardId>,
    pub default_card: Option<String>,

    /// Provide the ID of a payment source already attached to this customer to make it this customer's default payment source.
    ///
    /// If you want to add a new payment source and make it the default, see the [source](https://stripe.com/docs/api/customers/update#update_customer-source) property.
    #[serde(skip_serializing_if = "Option::is_none")]
    // pub default_source: Option<PaymentSourceId>,
    pub default_source: Option<String>,

    /// An arbitrary string that you can attach to a customer object.
    ///
    /// It is displayed alongside the customer in the dashboard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Customer's email address.
    ///
    /// It's displayed alongside the customer in your dashboard and can be useful for searching and tracking.
    /// This may be up to *512 characters*.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    // /// Specifies which fields in the response should be expanded.
    // #[serde(skip_serializing_if = "Expand::is_empty")]
    // pub expand: Vec<String>,

    /// The prefix for the customer used to generate unique invoice numbers.
    ///
    /// Must be 3–12 uppercase letters or numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_prefix: Option<String>,

    /// Default invoice settings for this customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_settings: Option<CustomerInvoiceSettings>,

    /// A set of key-value pairs that you can attach to a customer object.
    ///
    /// It can be useful for storing additional information about the customer in a structured format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    /// The customer's full name or business name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The customer's phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Customer's preferred languages, ordered by preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_locales: Option<Vec<String>>,

    /// The customer's shipping information.
    ///
    /// Appears on invoices emailed to this customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<ShippingParams>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// The customer's tax exemption.
    ///
    /// One of `none`, `exempt`, or `reverse`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exempt: Option<CustomerTaxExemptFilter>,

    /// The customer's tax information.
    ///
    /// Appears on invoices emailed to this customer.
    /// This parameter has been deprecated and will be removed in a future API version, for further information view the [migration guide](https://stripe.com/docs/billing/migration/taxes#moving-from-taxinfo-to-customer-tax-ids).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_info: Option<TaxInfoParams>,

    /// Unix timestamp representing the end of the trial period the customer will get before being charged for the first time.
    ///
    /// This will always overwrite any trials that might apply via a subscribed plan.
    /// If set, trial_end will override the default trial period of the plan the customer is being subscribed to.
    /// The special value `now` can be provided to end the customer's trial immediately.
    /// Can be at most two years from `billing_cycle_anchor`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trial_end: Option<Scheduled>,
}

impl CustomerUpdateParams {
    pub fn new() -> Self {
        CustomerUpdateParams {
            account_balance: Default::default(),
            balance: Default::default(),
            address: Default::default(),
            coupon: Default::default(),
            default_alipay_account: Default::default(),
            default_bank_account: Default::default(),
            default_card: Default::default(),
            default_source: Default::default(),
            description: Default::default(),
            email: Default::default(),
            // expand: Default::default(),
            invoice_prefix: Default::default(),
            invoice_settings: Default::default(),
            metadata: Default::default(),
            name: Default::default(),
            phone: Default::default(),
            preferred_locales: Default::default(),
            shipping: Default::default(),
            source: Default::default(),
            tax_exempt: Default::default(),
            tax_info: Default::default(),
            trial_end: Default::default(),
        }
    }
}

impl Default for CustomerUpdateParams {
    fn default() -> Self {
        CustomerUpdateParams {
            account_balance: Default::default(),
            balance: Default::default(),
            address: Default::default(),
            coupon: Default::default(),
            default_alipay_account: Default::default(),
            default_bank_account: Default::default(),
            default_card: Default::default(),
            default_source: Default::default(),
            description: Default::default(),
            email: Default::default(),
            // expand: Default::default(),
            invoice_prefix: Default::default(),
            invoice_settings: Default::default(),
            metadata: Default::default(),
            name: Default::default(),
            phone: Default::default(),
            preferred_locales: Default::default(),
            shipping: Default::default(),
            source: Default::default(),
            tax_exempt: Default::default(),
            tax_info: Default::default(),
            trial_end: Default::default(),
        }
    }
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AddressParams {
    pub line1: String,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub state: Option<String>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvoiceSettingCustomerSetting {
    /// Default custom fields to be displayed on invoices for this customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<InvoiceSettingCustomField>>,

    /// ID of the default payment method used for subscriptions and invoices for the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_payment_method: Option<Expandable<PaymentMethod>>,

    /// Default footer to be displayed on invoices for this customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomerInvoiceSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<CustomField>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_payment_method: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvoiceSettingCustomField {
    /// The name of the custom field.
    pub name: String,

    /// The value of the custom field.
    pub value: String,
}


#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TaxInfo {
    /// The customer's tax ID number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,

    /// The type of ID number.
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaxInfoVerification {
    /// The state of verification for this customer.
    ///
    /// Possible values are `unverified`, `pending`, or `verified`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The official name associated with the tax ID returned from the external provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_name: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TaxIdData {
    #[serde(rename = "type")]
    pub type_: Option<TaxIdDataType>,

    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaxInfoParams {
    pub tax_id: String,

    #[serde(rename = "type")]
    pub type_: TaxInfoType,
}

/// An enum representing the possible values of an `Customer`'s `tax_exempt` field.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CustomerTaxExempt {
    Exempt,
    None,
    Reverse,
}

impl CustomerTaxExempt {
    pub fn as_str(&self) -> &'static str {
        match self {
            CustomerTaxExempt::Exempt => "exempt",
            CustomerTaxExempt::None => "none",
            CustomerTaxExempt::Reverse => "reverse",
        }
    }
}

impl AsRef<str> for CustomerTaxExempt {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for CustomerTaxExempt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

/// An enum representing the possible values of an `CustomerCreateParams`'s `tax_exempt` field.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CustomerTaxExemptFilter {
    Exempt,
    None,
    Reverse,
}

impl CustomerTaxExemptFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            CustomerTaxExemptFilter::Exempt => "exempt",
            CustomerTaxExemptFilter::None => "none",
            CustomerTaxExemptFilter::Reverse => "reverse",
        }
    }
}

impl AsRef<str> for CustomerTaxExemptFilter {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for CustomerTaxExemptFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

/// An enum representing the possible values of an `TaxIdData`'s `type` field.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaxIdDataType {
    AuAbn,
    EuVat,
    NzGst,
}

impl TaxIdDataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaxIdDataType::AuAbn => "au_abn",
            TaxIdDataType::EuVat => "eu_vat",
            TaxIdDataType::NzGst => "nz_gst",
        }
    }
}

impl AsRef<str> for TaxIdDataType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for TaxIdDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

/// An enum representing the possible values of an `TaxInfoParams`'s `type` field.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaxInfoType {
    Vat,
}

impl TaxInfoType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaxInfoType::Vat => "vat",
        }
    }
}

impl AsRef<str> for TaxInfoType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for TaxInfoType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}
