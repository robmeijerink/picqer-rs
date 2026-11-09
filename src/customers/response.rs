use std::collections::BTreeMap;

use serde::Deserialize;

use crate::country::CountryCode;
use crate::reported::Reported;
use crate::tags::Tag;
use crate::types::{
    CustomerAddressId, CustomerFieldId, CustomerId, FulfilmentCustomerId, Percentage, TemplateId,
};

string_enum! {
    /// The language of a customer, used for what is sent to them.
    ///
    /// These are the two languages Picqer accepted on the account this was checked against. It
    /// refused `de`, `fr` and others.
    pub enum Language {
        /// Dutch.
        Nl = "nl",
        /// English.
        En = "en",
    }
}

/// An address of a customer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomerAddress {
    /// The address.
    pub idcustomer_address: CustomerAddressId,
    /// The name on the address.
    pub name: String,
    /// The contact person.
    pub contactname: Option<String>,
    /// The street and number.
    pub address: Option<String>,
    /// The second address line.
    pub address2: Option<String>,
    /// The postal code.
    pub zipcode: Option<String>,
    /// The city.
    pub city: Option<String>,
    /// The region, state or province.
    pub region: Option<String>,
    /// The country.
    ///
    /// Picqer keeps the code as it was given, so it can be one this crate does not know, such as a
    /// lower case one.
    pub country: Option<Reported<CountryCode>>,
    /// Whether this is the address used for invoices on a new order.
    pub defaultinvoice: Option<bool>,
    /// Whether this is the address used for delivery on a new order.
    pub defaultdelivery: Option<bool>,
}

/// The value of a customer field on a customer.
///
/// Picqer only reports the fields that have a value, and only when a single customer is fetched. The
/// value is always text: a number that was sent is reported as its text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomerFieldEntry {
    /// The customer field.
    pub idcustomerfield: CustomerFieldId,
    /// The title of the field.
    pub title: Option<String>,
    /// The value.
    pub value: Option<String>,
}

/// A customer.
///
/// Picqer reports more fields than its documentation lists, such as `full_address` and
/// `partialdelivery`. Those are not read here.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Customer {
    /// The customer.
    pub idcustomer: CustomerId,
    /// The name of the company, or of the person for a consumer.
    pub name: String,
    /// Your own number for the customer. The documentation says this is an integer, but Picqer
    /// reports and accepts text such as `ABC-1`.
    pub customerid: Option<String>,
    /// The contact person.
    pub contactname: Option<String>,
    /// The phone number.
    pub telephone: Option<String>,
    /// The email address.
    pub emailaddress: Option<String>,
    /// The discount the customer gets by default, in percent.
    pub discount: Option<Percentage>,
    /// The VAT number.
    pub vatnumber: Option<String>,
    /// Whether the customer pays VAT.
    pub calculatevat: Option<bool>,
    /// Internal remarks.
    pub remarks: Option<String>,
    /// Remarks that are added to every new order.
    pub default_order_remarks: Option<String>,
    /// The language of the customer.
    pub language: Option<Reported<Language>>,
    /// Whether orders of the customer are split automatically.
    pub auto_split: Option<bool>,
    /// The template used for the customer.
    pub idtemplate: Option<TemplateId>,
    /// The fulfilment customer, with Picqer Fulfilment.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// The addresses of the customer.
    #[serde(default)]
    pub addresses: Vec<CustomerAddress>,
    /// The values of the customer fields that have one. Only a single customer has them.
    #[serde(default)]
    pub customerfields: Vec<CustomerFieldEntry>,
    /// The tags of the customer, by title. The documentation shows a list, but Picqer sends an
    /// object.
    #[serde(default, deserialize_with = "crate::serde_util::tag_map")]
    pub tags: BTreeMap<String, Tag>,
}
