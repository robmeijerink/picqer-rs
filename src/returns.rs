//! Returns.
//!
//! A return registers products that a customer sends back, and the products that are sent in
//! exchange. The documentation describes the fields and its examples show a few more, which are read
//! here too. What a developer account also reports but the documentation never shows, such as the
//! number of comments and the value of a return with VAT, is not read.

use std::num::NonZeroU32;

use reqwest::Method;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::client::Client;
use crate::country::CountryCode;
use crate::customers::Language;
use crate::error::{Result, ValidationError};
use crate::http::{Request, segments};
use crate::locations::Location;
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::products::LotNumber;
use crate::reported::Reported;
use crate::types::{
    BackorderId, CustomerId, Date, DateTime, EmailAddress, FulfilmentCustomerId, LocationId, Money,
    NonEmpty, OrderId, PicklistId, ProductId, ReplacementProductId, ReturnId, ReturnLogId,
    ReturnReasonId, ReturnStatusId, ReturnedProductId, TemplateId, UserId, WarehouseId,
};

string_enum! {
    /// What happened to a product that is returned.
    pub enum ReturnedProductStatus {
        /// The product is expected.
        Expected = "expected",
        /// The product is received.
        Received = "received",
        /// The product is received and added to the stock. This cannot be undone.
        AddedToStock = "added_to_stock",
        /// The product is received and cannot be sold.
        Unsalable = "unsalable",
    }
}

string_enum! {
    /// What an entry in the logbook of a return is.
    pub enum ReturnLogType {
        /// The return was made.
        Created = "created",
        /// A comment.
        Comment = "comment",
        /// The status changed.
        StatusChanged = "status_changed",
        /// The status changed to one that completes a return.
        Completed = "completed",
        /// What happened to returned products.
        ReceivedProducts = "received_products",
        /// A picklist was made for the replacement products.
        CreatedPicklist = "created_picklist",
    }
}

/// A status of returns.
///
/// Where a status is embedded in a return or in its logbook, only what the documentation shows
/// there is filled in.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnStatus {
    /// The status.
    pub idreturn_status: ReturnStatusId,
    /// The name of the status.
    pub name: Option<String>,
    /// Whether this is the status a new return gets.
    pub default: Option<bool>,
    /// Whether a return with this status is completed.
    pub completed: Option<bool>,
    /// The color, as six hexadecimal digits without `#`.
    pub color: Option<String>,
    /// When the status was made.
    pub created_at: Option<DateTime>,
    /// When the status was last changed.
    pub updated_at: Option<DateTime>,
    /// When the status was deleted, which does not remove it from the returns that have it.
    pub deleted_at: Option<DateTime>,
}

/// A reason for a return.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnReason {
    /// The reason.
    pub idreturn_reason: ReturnReasonId,
    /// The name of the reason.
    pub name: Option<String>,
    /// Whether this is the default reason.
    pub default: Option<bool>,
    /// When the reason was made.
    pub created_at: Option<DateTime>,
    /// When the reason was last changed.
    pub updated_at: Option<DateTime>,
    /// When the reason was deleted.
    pub deleted_at: Option<DateTime>,
}

/// The product of a line of a return.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnProduct {
    /// The product.
    pub idproduct: ProductId,
    /// Where the image of the product can be downloaded.
    pub image_url: Option<String>,
    /// The product code.
    pub productcode: Option<String>,
    /// The name of the product.
    pub name: Option<String>,
}

/// A product that is returned.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnedProduct {
    /// The line of the return.
    pub idreturn_product: ReturnedProductId,
    /// The return.
    pub idreturn: ReturnId,
    /// The reason.
    pub idreturn_reason: Option<ReturnReasonId>,
    /// The product.
    pub idproduct: ProductId,
    /// The warehouse it was added to the stock of.
    pub idwarehouse: Option<WarehouseId>,
    /// The location it was added to the stock of.
    pub idlocation: Option<LocationId>,
    /// The lot number, when the product tracks lots and was added to the stock.
    pub lot_number: Option<String>,
    /// The expiration date, when the product tracks them and was added to the stock.
    pub expiration_date: Option<Date>,
    /// The price.
    pub price: Option<Money>,
    /// How many are returned.
    pub amount: Option<i64>,
    /// What happened to the product.
    pub status: Option<Reported<ReturnedProductStatus>>,
    /// Whether the line can still be changed or deleted. Not once it is added to the stock.
    pub changeable: Option<bool>,
    /// The product.
    pub product: Option<ReturnProduct>,
    /// The reason.
    pub return_reason: Option<ReturnReason>,
    /// The location it was added to the stock of.
    pub location: Option<Location>,
}

/// A product that is sent in exchange for a returned one.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReplacementProduct {
    /// The line of the return.
    pub idreturn_product_replacement: ReplacementProductId,
    /// The return.
    pub idreturn: ReturnId,
    /// The product.
    pub idproduct: ProductId,
    /// The picklist made for it.
    pub idpicklist: Option<PicklistId>,
    /// The backorder made for it.
    pub idbackorder: Option<BackorderId>,
    /// The price.
    pub price: Option<Money>,
    /// How many are sent.
    pub amount: Option<i64>,
    /// Where the replacement is, such as `concept`. The documentation does not list the values.
    pub status: Option<String>,
    /// Whether the line can still be changed or deleted.
    pub changeable: Option<bool>,
    /// The product.
    pub product: Option<ReturnProduct>,
}

/// The customer of a return, as a single return shows it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnCustomer {
    /// The customer.
    pub idcustomer: CustomerId,
    /// The number of the customer.
    #[serde(default, deserialize_with = "crate::serde_util::opt_text")]
    pub customerid: Option<String>,
    /// The name of the customer.
    pub name: Option<String>,
    /// The contact person.
    pub contactname: Option<String>,
}

/// The order of a return, as a single return shows it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnOrder {
    /// The order.
    pub idorder: OrderId,
    /// The number of the order.
    pub orderid: Option<String>,
    /// Your own reference.
    pub reference: Option<String>,
    /// When the order was made.
    pub created_at: Option<DateTime>,
}

/// The amounts and values of a return.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnTotals {
    /// How many products are returned.
    pub returned_products_count: Option<i64>,
    /// What they are worth.
    pub returned_products_value: Option<Money>,
    /// How many replacement products are sent.
    pub replacement_products_count: Option<i64>,
    /// What they are worth.
    pub replacement_products_value: Option<Money>,
    /// How many days after the order was shipped the return was made.
    pub days_after_order_shipped: Option<i64>,
}

/// A return.
///
/// A single return comes with its status, customer, order, products and totals. A list has only
/// the fields of the return itself.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Return {
    /// The return.
    pub idreturn: ReturnId,
    /// The status.
    pub idreturn_status: ReturnStatusId,
    /// The customer.
    pub idcustomer: Option<CustomerId>,
    /// The order.
    pub idorder: Option<OrderId>,
    /// The template.
    pub idtemplate: Option<TemplateId>,
    /// The number of the return, such as `R2026-1024`. The documentation calls it an integer.
    pub returnid: Option<String>,
    /// The name to return from.
    pub name: Option<String>,
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
    /// The region.
    pub region: Option<String>,
    /// The country. Picqer does not check it, so any text can come back.
    pub country: Option<Reported<CountryCode>>,
    /// The address as one text.
    pub full_address: Option<String>,
    /// The phone number.
    pub telephone: Option<String>,
    /// The email address. Picqer does not check it.
    pub emailaddress: Option<String>,
    /// The language. Picqer does not check it, so any text can come back.
    pub language: Option<Reported<Language>>,
    /// Your own reference.
    pub reference: Option<String>,
    /// The tracking code of the parcel that is sent back.
    pub tracking_code: Option<String>,
    /// When the return was received.
    pub received_at: Option<DateTime>,
    /// When the return was completed.
    pub completed_at: Option<DateTime>,
    /// The user that completed the return.
    pub completed_by_iduser: Option<UserId>,
    /// The fulfilment customer, with Picqer Fulfilment.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// When the return was made.
    pub created_at: Option<DateTime>,
    /// When the return was last changed.
    pub updated_at: Option<DateTime>,
    /// The status. Only a single return has it.
    pub return_status: Option<ReturnStatus>,
    /// The customer. Only a single return has it.
    pub customer: Option<ReturnCustomer>,
    /// The order. Only a single return has it.
    pub order: Option<ReturnOrder>,
    /// The returned products. Only a single return has them.
    #[serde(default)]
    pub returned_products: Vec<ReturnedProduct>,
    /// The replacement products. Only a single return has them.
    #[serde(default)]
    pub replacement_products: Vec<ReplacementProduct>,
    /// The totals. Only a single return has them.
    pub totals: Option<ReturnTotals>,
}

/// The user of an entry in the logbook.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnLogUser {
    /// The user.
    pub iduser: UserId,
    /// The name of the user.
    pub name: Option<String>,
}

/// An entry in the logbook of a return: a change or a comment.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReturnLog {
    /// The entry.
    pub idreturn_log: ReturnLogId,
    /// The status the return had.
    pub idreturn_status: Option<ReturnStatusId>,
    /// The user that made the entry.
    pub iduser: Option<UserId>,
    /// What the entry is.
    #[serde(rename = "type")]
    pub log_type: Option<Reported<ReturnLogType>>,
    /// The comment or what happened.
    pub message: Option<String>,
    /// Whether the customer got an email about it.
    pub notified_customer: Option<bool>,
    /// When the entry was made.
    pub created_at: Option<DateTime>,
    /// When the entry was last changed.
    pub updated_at: Option<DateTime>,
    /// The status the return had, with its name and color.
    pub return_status: Option<ReturnStatus>,
    /// The user that made the entry.
    pub user: Option<ReturnLogUser>,
}

/// Which returns to list by status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnStatusFilter {
    /// The returns with a status that is of the processing kind.
    Processing,
    /// The returns with a status that completes a return.
    Completed,
    /// The returns with this status.
    Status(ReturnStatusId),
}

/// Filters for listing returns.
///
/// Picqer lists every return for a status filter it does not know, which cannot be sent here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReturnQuery {
    search: Option<String>,
    status: Option<ReturnStatusFilter>,
    emailaddress: Option<String>,
    idcustomer: Option<CustomerId>,
    created_after: Option<DateTime>,
    updated_after: Option<DateTime>,
}

impl ReturnQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Searches the number, the reference, the name of the customer and its contact person.
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    /// Lists the returns with a status.
    pub fn status(mut self, status: ReturnStatusFilter) -> Self {
        self.status = Some(status);
        self
    }

    /// Lists the returns with this email address.
    pub fn emailaddress(mut self, emailaddress: impl Into<String>) -> Self {
        self.emailaddress = Some(emailaddress.into());
        self
    }

    /// Lists the returns of a customer.
    pub fn idcustomer(mut self, idcustomer: CustomerId) -> Self {
        self.idcustomer = Some(idcustomer);
        self
    }

    /// Lists the returns made after this moment.
    pub fn created_after(mut self, moment: DateTime) -> Self {
        self.created_after = Some(moment);
        self
    }

    /// Lists the returns changed after this moment.
    pub fn updated_after(mut self, moment: DateTime) -> Self {
        self.updated_after = Some(moment);
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(search) = &self.search {
            pairs.push(("search", search.clone()));
        }
        if let Some(status) = self.status {
            pairs.push((
                "status",
                match status {
                    ReturnStatusFilter::Processing => "processing".to_owned(),
                    ReturnStatusFilter::Completed => "completed".to_owned(),
                    ReturnStatusFilter::Status(id) => id.to_string(),
                },
            ));
        }
        if let Some(email) = &self.emailaddress {
            pairs.push(("emailaddress", email.clone()));
        }
        if let Some(id) = self.idcustomer {
            pairs.push(("idcustomer", id.to_string()));
        }
        if let Some(moment) = self.created_after {
            pairs.push(("created_after", moment.to_string()));
        }
        if let Some(moment) = self.updated_after {
            pairs.push(("updated_after", moment.to_string()));
        }
        pairs
    }
}

macro_rules! return_fields {
    () => {
        /// Sets the name to return from.
        pub fn name(mut self, name: impl Into<String>) -> Self {
            self.fields.insert("name".into(), json!(name.into()));
            self
        }

        /// Sets the contact person.
        pub fn contactname(mut self, contactname: impl Into<String>) -> Self {
            self.fields
                .insert("contactname".into(), json!(contactname.into()));
            self
        }

        /// Sets the street and number.
        pub fn address(mut self, address: impl Into<String>) -> Self {
            self.fields.insert("address".into(), json!(address.into()));
            self
        }

        /// Sets the second address line.
        pub fn address2(mut self, address2: impl Into<String>) -> Self {
            self.fields
                .insert("address2".into(), json!(address2.into()));
            self
        }

        /// Sets the postal code.
        pub fn zipcode(mut self, zipcode: impl Into<String>) -> Self {
            self.fields.insert("zipcode".into(), json!(zipcode.into()));
            self
        }

        /// Sets the city.
        pub fn city(mut self, city: impl Into<String>) -> Self {
            self.fields.insert("city".into(), json!(city.into()));
            self
        }

        /// Sets the region.
        pub fn region(mut self, region: impl Into<String>) -> Self {
            self.fields.insert("region".into(), json!(region.into()));
            self
        }

        /// Sets the country.
        pub fn country(mut self, country: CountryCode) -> Self {
            self.fields.insert("country".into(), json!(country));
            self
        }

        /// Sets the phone number.
        pub fn telephone(mut self, telephone: impl Into<String>) -> Self {
            self.fields
                .insert("telephone".into(), json!(telephone.into()));
            self
        }

        /// Sets the email address.
        pub fn emailaddress(mut self, emailaddress: EmailAddress) -> Self {
            self.fields
                .insert("emailaddress".into(), json!(emailaddress));
            self
        }

        /// Sets the language.
        pub fn language(mut self, language: Language) -> Self {
            self.fields.insert("language".into(), json!(language));
            self
        }

        /// Sets your own reference.
        pub fn reference(mut self, reference: impl Into<String>) -> Self {
            self.fields
                .insert("reference".into(), json!(reference.into()));
            self
        }

        /// Sets the tracking code of the parcel that is sent back.
        pub fn tracking_code(mut self, tracking_code: impl Into<String>) -> Self {
            self.fields
                .insert("tracking_code".into(), json!(tracking_code.into()));
            self
        }

        /// Sets the template.
        pub fn idtemplate(mut self, idtemplate: TemplateId) -> Self {
            self.fields.insert("idtemplate".into(), json!(idtemplate));
            self
        }

        /// Sets the customer.
        pub fn idcustomer(mut self, idcustomer: CustomerId) -> Self {
            self.fields.insert("idcustomer".into(), json!(idcustomer));
            self
        }

        /// Sets the order.
        pub fn idorder(mut self, idorder: OrderId) -> Self {
            self.fields.insert("idorder".into(), json!(idorder));
            self
        }

        /// Sets the fulfilment customer, with Picqer Fulfilment.
        pub fn idfulfilment_customer(mut self, customer: FulfilmentCustomerId) -> Self {
            self.fields
                .insert("idfulfilment_customer".into(), json!(customer));
            self
        }
    };
}

/// A product to return, when a return is made or later.
///
/// Picqer takes an amount of zero, a fraction (as its whole part), a negative price and a missing
/// price (as zero), which cannot be sent here except for the price.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewReturnedProduct {
    idreturn_reason: ReturnReasonId,
    idproduct: ProductId,
    amount: NonZeroU32,
    price: Option<Money>,
}

impl NewReturnedProduct {
    /// Returns an amount of a product for a reason.
    pub fn new(idreturn_reason: ReturnReasonId, idproduct: ProductId, amount: NonZeroU32) -> Self {
        Self {
            idreturn_reason,
            idproduct,
            amount,
            price: None,
        }
    }

    /// Sets the price. Picqer takes it as zero without one.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    pub(crate) fn to_value(self) -> Value {
        let mut body = Map::new();
        body.insert("idreturn_reason".into(), json!(self.idreturn_reason));
        body.insert("idproduct".into(), json!(self.idproduct));
        body.insert("amount".into(), json!(self.amount));
        if let Some(price) = self.price {
            body.insert("price".into(), json!(price));
        }
        Value::Object(body)
    }
}

/// A product to send in exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewReplacementProduct {
    idproduct: ProductId,
    amount: NonZeroU32,
    price: Option<Money>,
}

impl NewReplacementProduct {
    /// Sends an amount of a product.
    pub fn new(idproduct: ProductId, amount: NonZeroU32) -> Self {
        Self {
            idproduct,
            amount,
            price: None,
        }
    }

    /// Sets the price. Picqer takes it as zero without one.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    pub(crate) fn to_value(self) -> Value {
        let mut body = Map::new();
        body.insert("idproduct".into(), json!(self.idproduct));
        body.insert("amount".into(), json!(self.amount));
        if let Some(price) = self.price {
            body.insert("price".into(), json!(price));
        }
        Value::Object(body)
    }
}

/// A return to create, with its status.
///
/// The products can be given here or added later. Picqer makes the return before it checks the
/// products, so a product that is refused leaves an empty return behind and the call fails. Picqer
/// takes the default template when none is given, although its documentation says it is required,
/// and it does not check the country, the email address or the language, which are checked here.
#[derive(Debug, Clone, PartialEq)]
pub struct NewReturn {
    fields: Map<String, Value>,
    returned_products: Vec<NewReturnedProduct>,
    replacement_products: Vec<NewReplacementProduct>,
}

impl NewReturn {
    /// A return with a status, such as the default one from [`ReturnStatuses::list`].
    pub fn new(idreturn_status: ReturnStatusId) -> Self {
        let mut fields = Map::new();
        fields.insert("idreturn_status".into(), json!(idreturn_status));
        Self {
            fields,
            returned_products: Vec::new(),
            replacement_products: Vec::new(),
        }
    }

    return_fields!();

    /// Adds a product to return.
    pub fn returned_product(mut self, product: NewReturnedProduct) -> Self {
        self.returned_products.push(product);
        self
    }

    /// Adds a product to send in exchange.
    pub fn replacement_product(mut self, product: NewReplacementProduct) -> Self {
        self.replacement_products.push(product);
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut body = self.fields.clone();
        if !self.returned_products.is_empty() {
            body.insert(
                "returned_products".into(),
                Value::Array(
                    self.returned_products
                        .iter()
                        .map(|product| product.to_value())
                        .collect(),
                ),
            );
        }
        if !self.replacement_products.is_empty() {
            body.insert(
                "replacement_products".into(),
                Value::Array(
                    self.replacement_products
                        .iter()
                        .map(|product| product.to_value())
                        .collect(),
                ),
            );
        }
        Value::Object(body)
    }
}

/// Changes to a return. Only what is set is sent.
///
/// Only the fields of the return itself can be changed here: products have their own calls.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReturnChanges {
    fields: Map<String, Value>,
}

impl ReturnChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    return_fields!();

    /// Changes the status. A status that completes a return also sets who completed it and when.
    pub fn idreturn_status(mut self, idreturn_status: ReturnStatusId) -> Self {
        self.fields
            .insert("idreturn_status".into(), json!(idreturn_status));
        self
    }

    /// Sets when the return was received.
    pub fn received_at(mut self, moment: DateTime) -> Self {
        self.fields.insert("received_at".into(), json!(moment));
        self
    }

    /// Removes the name.
    pub fn without_name(mut self) -> Self {
        self.fields.insert("name".into(), Value::Null);
        self
    }

    /// Removes the reference.
    pub fn without_reference(mut self) -> Self {
        self.fields.insert("reference".into(), Value::Null);
        self
    }

    /// Removes the tracking code.
    pub fn without_tracking_code(mut self) -> Self {
        self.fields.insert("tracking_code".into(), Value::Null);
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        Value::Object(self.fields.clone())
    }
}

/// Changes to a returned product that is still changeable. Only what is set is sent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReturnedProductChanges {
    price: Option<Money>,
    amount: Option<NonZeroU32>,
    idreturn_reason: Option<ReturnReasonId>,
}

impl ReturnedProductChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the price.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    /// Changes the amount.
    pub fn amount(mut self, amount: NonZeroU32) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Changes the reason.
    pub fn idreturn_reason(mut self, reason: ReturnReasonId) -> Self {
        self.idreturn_reason = Some(reason);
        self
    }

    pub(crate) fn to_value(self) -> Value {
        let mut body = Map::new();
        if let Some(price) = self.price {
            body.insert("price".into(), json!(price));
        }
        if let Some(amount) = self.amount {
            body.insert("amount".into(), json!(amount));
        }
        if let Some(reason) = self.idreturn_reason {
            body.insert("idreturn_reason".into(), json!(reason));
        }
        Value::Object(body)
    }
}

/// Changes to a replacement product that is still changeable. Only what is set is sent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReplacementProductChanges {
    price: Option<Money>,
    amount: Option<NonZeroU32>,
}

impl ReplacementProductChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the price.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    /// Changes the amount.
    pub fn amount(mut self, amount: NonZeroU32) -> Self {
        self.amount = Some(amount);
        self
    }

    pub(crate) fn to_value(self) -> Value {
        let mut body = Map::new();
        if let Some(price) = self.price {
            body.insert("price".into(), json!(price));
        }
        if let Some(amount) = self.amount {
            body.insert("amount".into(), json!(amount));
        }
        Value::Object(body)
    }
}

/// An entry to add to the logbook of a return: a comment, a change of status or both.
///
/// Picqer answers an entry with neither as if it was made, and makes none, so one needs a comment
/// or a status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewReturnLog {
    status: Option<ReturnStatusId>,
    message: Option<String>,
    notify_customer: bool,
}

impl NewReturnLog {
    /// A comment.
    pub fn comment(message: impl Into<String>) -> Self {
        Self {
            status: None,
            message: Some(message.into()),
            notify_customer: false,
        }
    }

    /// A change of status. A status that completes a return also sets who completed it and when.
    pub fn status(status: ReturnStatusId) -> Self {
        Self {
            status: Some(status),
            message: None,
            notify_customer: false,
        }
    }

    /// Adds a comment to a change of status.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Adds a change of status to a comment.
    pub fn with_status(mut self, status: ReturnStatusId) -> Self {
        self.status = Some(status);
        self
    }

    /// Sends the customer an email about it, when the return has an email address.
    pub fn notify_customer(mut self) -> Self {
        self.notify_customer = true;
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut body = Map::new();
        if let Some(status) = self.status {
            body.insert("idreturn_status".into(), json!(status));
        }
        if let Some(message) = &self.message {
            body.insert("message".into(), json!(message));
        }
        if self.notify_customer {
            body.insert("notify_customer".into(), json!(true));
        }
        Value::Object(body)
    }
}

/// What to register for a returned product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedProduct {
    idreturn_product: ReturnedProductId,
    status: ReturnedProductStatus,
    idlocation: Option<LocationId>,
    lot_number: Option<LotNumber>,
    expiration_date: Option<Date>,
}

impl ReceivedProduct {
    /// Registers what happened to a returned product.
    pub fn new(idreturn_product: ReturnedProductId, status: ReturnedProductStatus) -> Self {
        Self {
            idreturn_product,
            status,
            idlocation: None,
            lot_number: None,
            expiration_date: None,
        }
    }

    /// Puts the stock on a location, for a product that is added to the stock. Without one it goes
    /// to no specific location.
    pub fn idlocation(mut self, idlocation: LocationId) -> Self {
        self.idlocation = Some(idlocation);
        self
    }

    /// Records the lot number, for a product that tracks lots and is added to the stock.
    pub fn lot_number(mut self, lot_number: LotNumber) -> Self {
        self.lot_number = Some(lot_number);
        self
    }

    /// Records the expiration date, for a product that tracks them and is added to the stock.
    pub fn expiration_date(mut self, date: Date) -> Self {
        self.expiration_date = Some(date);
        self
    }

    fn to_value(&self) -> Value {
        let mut body = Map::new();
        body.insert("idreturn_product".into(), json!(self.idreturn_product));
        body.insert("status".into(), json!(self.status));
        if let Some(location) = self.idlocation {
            body.insert("idlocation".into(), json!(location));
        }
        if let Some(lot) = &self.lot_number {
            body.insert("lot_number".into(), json!(lot));
        }
        if let Some(date) = self.expiration_date {
            body.insert("expiration_date".into(), json!(date));
        }
        Value::Object(body)
    }
}

/// What happened to the products of a return.
///
/// A product that is added to the stock cannot be changed back, and a return that has one cannot be
/// deleted anymore. The other statuses can be changed into each other. A product that tracks lots
/// or expiration dates needs a location and every value it tracks when it is added to the stock, and
/// Picqer answers `422` with the errors of each line in `context.errors` otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveReturn {
    idwarehouse: Option<WarehouseId>,
    products: NonEmpty<ReceivedProduct>,
}

impl ReceiveReturn {
    /// Registers what happened to some products.
    pub fn new(products: NonEmpty<ReceivedProduct>) -> Self {
        Self {
            idwarehouse: None,
            products,
        }
    }

    /// Sets the warehouse that products are added to the stock of. This is required when one is.
    pub fn idwarehouse(mut self, idwarehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(idwarehouse);
        self
    }

    pub(crate) fn to_value(&self) -> Result<Value> {
        let adds_stock = self
            .products
            .iter()
            .any(|product| product.status == ReturnedProductStatus::AddedToStock);
        if adds_stock && self.idwarehouse.is_none() {
            return Err(ValidationError::new(
                "return warehouse",
                "a warehouse is required to add products to the stock",
            )
            .into());
        }
        let mut body = Map::new();
        if let Some(warehouse) = self.idwarehouse {
            body.insert("idwarehouse".into(), json!(warehouse));
        }
        body.insert(
            "return_products".into(),
            Value::Array(
                self.products
                    .iter()
                    .map(ReceivedProduct::to_value)
                    .collect(),
            ),
        );
        Ok(Value::Object(body))
    }
}

pub(crate) fn list(query: &ReturnQuery, page: Page) -> Call<Vec<Return>> {
    let mut request = Request::new(Method::GET, segments(&[&"returns"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: ReturnId) -> Call<Return> {
    Call::new(Request::new(Method::GET, segments(&[&"returns", &id])))
}

pub(crate) fn create(new: &NewReturn) -> Result<Call<Return>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"returns"])).json_body(&new.to_value())?,
    ))
}

pub(crate) fn update(id: ReturnId, changes: &ReturnChanges) -> Result<Call<Return>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"returns", &id])).json_body(&changes.to_value())?,
    ))
}

pub(crate) fn delete(id: ReturnId) -> NoContent {
    NoContent::new(Request::new(Method::DELETE, segments(&[&"returns", &id])))
}

pub(crate) fn returned_products(id: ReturnId) -> Call<Vec<ReturnedProduct>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"returns", &id, &"returned_products"]),
    ))
}

pub(crate) fn add_returned_product(
    id: ReturnId,
    product: NewReturnedProduct,
) -> Result<Call<ReturnedProduct>> {
    Ok(Call::new(
        Request::new(
            Method::POST,
            segments(&[&"returns", &id, &"returned_products"]),
        )
        .json_body(&product.to_value())?,
    ))
}

pub(crate) fn update_returned_product(
    id: ReturnId,
    line: ReturnedProductId,
    changes: ReturnedProductChanges,
) -> Result<Call<ReturnedProduct>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"returns", &id, &"returned_products", &line]),
        )
        .json_body(&changes.to_value())?,
    ))
}

pub(crate) fn remove_returned_product(id: ReturnId, line: ReturnedProductId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"returns", &id, &"returned_products", &line]),
    ))
}

pub(crate) fn replacement_products(id: ReturnId) -> Call<Vec<ReplacementProduct>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"returns", &id, &"replacement_products"]),
    ))
}

pub(crate) fn add_replacement_product(
    id: ReturnId,
    product: NewReplacementProduct,
) -> Result<Call<ReplacementProduct>> {
    Ok(Call::new(
        Request::new(
            Method::POST,
            segments(&[&"returns", &id, &"replacement_products"]),
        )
        .json_body(&product.to_value())?,
    ))
}

pub(crate) fn update_replacement_product(
    id: ReturnId,
    line: ReplacementProductId,
    changes: ReplacementProductChanges,
) -> Result<Call<ReplacementProduct>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"returns", &id, &"replacement_products", &line]),
        )
        .json_body(&changes.to_value())?,
    ))
}

pub(crate) fn remove_replacement_product(id: ReturnId, line: ReplacementProductId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"returns", &id, &"replacement_products", &line]),
    ))
}

pub(crate) fn logs(id: ReturnId) -> Call<Vec<ReturnLog>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"returns", &id, &"logs"]),
    ))
}

pub(crate) fn add_log(id: ReturnId, log: &NewReturnLog) -> Result<Call<ReturnLog>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"returns", &id, &"logs"]))
            .json_body(&log.to_value())?,
    ))
}

pub(crate) fn receive(id: ReturnId, received: &ReceiveReturn) -> Result<Call<Return>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"returns", &id, &"receive"]))
            .json_body(&received.to_value()?)?,
    ))
}

pub(crate) fn statuses() -> Call<Vec<ReturnStatus>> {
    Call::new(Request::new(Method::GET, segments(&[&"return_statuses"])))
}

pub(crate) fn reasons() -> Call<Vec<ReturnReason>> {
    Call::new(Request::new(Method::GET, segments(&[&"return_reasons"])))
}

macro_rules! returns_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of returns, newest first.
        pub $($async)? fn list(
            &self,
            query: &$crate::returns::ReturnQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::returns::Return>> {
            self.client.execute($crate::returns::list(query, page)) $(.$await)?
        }

        /// Lists all returns that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::returns::ReturnQuery,
        ) -> $crate::error::Result<Vec<$crate::returns::Return>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::returns::list(query, page)) $(.$await)?)
        }

        /// Fetches a return, with its status, customer, order, products and totals.
        pub $($async)? fn get(
            &self,
            id: $crate::types::ReturnId,
        ) -> $crate::error::Result<$crate::returns::Return> {
            self.client.execute($crate::returns::get(id)) $(.$await)?
        }

        /// Creates a return. The answer does not yet show the replacement products that were
        /// given: fetch the return to see them.
        pub $($async)? fn create(
            &self,
            new: &$crate::returns::NewReturn,
        ) -> $crate::error::Result<$crate::returns::Return> {
            self.client.execute($crate::returns::create(new)?) $(.$await)?
        }

        /// Changes a return and returns it.
        pub $($async)? fn update(
            &self,
            id: $crate::types::ReturnId,
            changes: &$crate::returns::ReturnChanges,
        ) -> $crate::error::Result<$crate::returns::Return> {
            self.client.execute($crate::returns::update(id, changes)?) $(.$await)?
        }

        /// Deletes a return. Picqer refuses one with a picklist or a backorder, and one with a
        /// product that was added to the stock.
        pub $($async)? fn delete(&self, id: $crate::types::ReturnId) -> $crate::error::Result<()> {
            self.client.execute($crate::returns::delete(id)) $(.$await)?
        }

        /// Lists the returned products of a return.
        pub $($async)? fn returned_products(
            &self,
            id: $crate::types::ReturnId,
        ) -> $crate::error::Result<Vec<$crate::returns::ReturnedProduct>> {
            self.client.execute($crate::returns::returned_products(id)) $(.$await)?
        }

        /// Adds a returned product to a return.
        pub $($async)? fn add_returned_product(
            &self,
            id: $crate::types::ReturnId,
            product: $crate::returns::NewReturnedProduct,
        ) -> $crate::error::Result<$crate::returns::ReturnedProduct> {
            self.client.execute($crate::returns::add_returned_product(id, product)?) $(.$await)?
        }

        /// Changes a returned product that is still changeable.
        pub $($async)? fn update_returned_product(
            &self,
            id: $crate::types::ReturnId,
            line: $crate::types::ReturnedProductId,
            changes: $crate::returns::ReturnedProductChanges,
        ) -> $crate::error::Result<$crate::returns::ReturnedProduct> {
            self.client.execute($crate::returns::update_returned_product(id, line, changes)?) $(.$await)?
        }

        /// Deletes a returned product that is still changeable.
        pub $($async)? fn remove_returned_product(
            &self,
            id: $crate::types::ReturnId,
            line: $crate::types::ReturnedProductId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::returns::remove_returned_product(id, line)) $(.$await)?
        }

        /// Lists the replacement products of a return.
        pub $($async)? fn replacement_products(
            &self,
            id: $crate::types::ReturnId,
        ) -> $crate::error::Result<Vec<$crate::returns::ReplacementProduct>> {
            self.client.execute($crate::returns::replacement_products(id)) $(.$await)?
        }

        /// Adds a replacement product to a return.
        pub $($async)? fn add_replacement_product(
            &self,
            id: $crate::types::ReturnId,
            product: $crate::returns::NewReplacementProduct,
        ) -> $crate::error::Result<$crate::returns::ReplacementProduct> {
            self.client.execute($crate::returns::add_replacement_product(id, product)?) $(.$await)?
        }

        /// Changes a replacement product that is still changeable.
        pub $($async)? fn update_replacement_product(
            &self,
            id: $crate::types::ReturnId,
            line: $crate::types::ReplacementProductId,
            changes: $crate::returns::ReplacementProductChanges,
        ) -> $crate::error::Result<$crate::returns::ReplacementProduct> {
            self.client.execute($crate::returns::update_replacement_product(id, line, changes)?) $(.$await)?
        }

        /// Deletes a replacement product that is still changeable.
        pub $($async)? fn remove_replacement_product(
            &self,
            id: $crate::types::ReturnId,
            line: $crate::types::ReplacementProductId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::returns::remove_replacement_product(id, line)) $(.$await)?
        }

        /// Lists the logbook of a return, newest first.
        pub $($async)? fn logs(
            &self,
            id: $crate::types::ReturnId,
        ) -> $crate::error::Result<Vec<$crate::returns::ReturnLog>> {
            self.client.execute($crate::returns::logs(id)) $(.$await)?
        }

        /// Adds a comment, a change of status or both to the logbook of a return.
        pub $($async)? fn add_log(
            &self,
            id: $crate::types::ReturnId,
            log: &$crate::returns::NewReturnLog,
        ) -> $crate::error::Result<$crate::returns::ReturnLog> {
            self.client.execute($crate::returns::add_log(id, log)?) $(.$await)?
        }

        /// Registers what happened to returned products, and returns the return.
        pub $($async)? fn receive(
            &self,
            id: $crate::types::ReturnId,
            received: &$crate::returns::ReceiveReturn,
        ) -> $crate::error::Result<$crate::returns::Return> {
            self.client.execute($crate::returns::receive(id, received)?) $(.$await)?
        }
    };
}

macro_rules! return_statuses_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists the statuses of returns.
        pub $($async)? fn list(
            &self,
        ) -> $crate::error::Result<Vec<$crate::returns::ReturnStatus>> {
            self.client.execute($crate::returns::statuses()) $(.$await)?
        }
    };
}

macro_rules! return_reasons_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists the reasons for returns.
        pub $($async)? fn list(
            &self,
        ) -> $crate::error::Result<Vec<$crate::returns::ReturnReason>> {
            self.client.execute($crate::returns::reasons()) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use {return_reasons_methods, return_statuses_methods, returns_methods};

/// Operations on returns.
#[derive(Debug, Clone, Copy)]
pub struct Returns<'a> {
    pub(crate) client: &'a Client,
}

impl Returns<'_> {
    returns_methods!(async; await);
}

/// Operations on the statuses of returns.
#[derive(Debug, Clone, Copy)]
pub struct ReturnStatuses<'a> {
    pub(crate) client: &'a Client,
}

impl ReturnStatuses<'_> {
    return_statuses_methods!(async; await);
}

/// Operations on the reasons for returns.
#[derive(Debug, Clone, Copy)]
pub struct ReturnReasons<'a> {
    pub(crate) client: &'a Client,
}

impl ReturnReasons<'_> {
    return_reasons_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_return_sends_its_status_fields_and_products() {
        let new = NewReturn::new(ReturnStatusId(1))
            .name("John Hendriks")
            .reference("Order 23992")
            .country(CountryCode::NL)
            .returned_product(
                NewReturnedProduct::new(
                    ReturnReasonId(2),
                    ProductId(3),
                    NonZeroU32::new(1).unwrap(),
                )
                .price(Money::from_cents(1295).unwrap()),
            )
            .replacement_product(NewReplacementProduct::new(
                ProductId(4),
                NonZeroU32::new(2).unwrap(),
            ));
        assert_eq!(
            new.to_value(),
            json!({
                "idreturn_status": 1, "name": "John Hendriks", "reference": "Order 23992",
                "country": "NL",
                "returned_products": [
                    {"idreturn_reason": 2, "idproduct": 3, "amount": 1, "price": 12.95}
                ],
                "replacement_products": [{"idproduct": 4, "amount": 2}]
            })
        );
        assert_eq!(
            NewReturn::new(ReturnStatusId(1)).to_value(),
            json!({"idreturn_status": 1})
        );
    }

    #[test]
    fn changes_can_remove_what_may_be_empty() {
        assert_eq!(ReturnChanges::new().to_value(), json!({}));
        let changes = ReturnChanges::new()
            .without_name()
            .without_reference()
            .without_tracking_code()
            .idreturn_status(ReturnStatusId(9));
        assert_eq!(
            changes.to_value(),
            json!({"name": null, "reference": null, "tracking_code": null, "idreturn_status": 9})
        );
    }

    #[test]
    fn a_log_needs_a_comment_or_a_status() {
        assert_eq!(
            NewReturnLog::comment("label sent")
                .notify_customer()
                .to_value(),
            json!({"message": "label sent", "notify_customer": true})
        );
        assert_eq!(
            NewReturnLog::status(ReturnStatusId(5))
                .message("done")
                .to_value(),
            json!({"idreturn_status": 5, "message": "done"})
        );
    }

    #[test]
    fn adding_to_the_stock_needs_a_warehouse() {
        let line = ReceivedProduct::new(ReturnedProductId(21), ReturnedProductStatus::AddedToStock)
            .idlocation(LocationId(6516))
            .lot_number(LotNumber::new("LOT-1").unwrap())
            .expiration_date("2027-01-15".parse().unwrap());
        let products = NonEmpty::new(vec![
            line,
            ReceivedProduct::new(ReturnedProductId(22), ReturnedProductStatus::Received),
        ])
        .unwrap();
        let received = ReceiveReturn::new(products.clone());
        assert!(received.to_value().is_err());
        assert_eq!(
            received.idwarehouse(WarehouseId(1)).to_value().unwrap(),
            json!({
                "idwarehouse": 1,
                "return_products": [
                    {
                        "idreturn_product": 21, "status": "added_to_stock", "idlocation": 6516,
                        "lot_number": "LOT-1", "expiration_date": "2027-01-15"
                    },
                    {"idreturn_product": 22, "status": "received"}
                ]
            })
        );
        let without_stock = NonEmpty::new(vec![ReceivedProduct::new(
            ReturnedProductId(22),
            ReturnedProductStatus::Unsalable,
        )])
        .unwrap();
        assert_eq!(
            ReceiveReturn::new(without_stock).to_value().unwrap(),
            json!({"return_products": [{"idreturn_product": 22, "status": "unsalable"}]})
        );
    }

    #[test]
    fn filters_become_query_pairs() {
        assert!(ReturnQuery::new().pairs().is_empty());
        let moment: DateTime = "2026-11-09 16:00:00".parse().unwrap();
        let query = ReturnQuery::new()
            .search("R2026")
            .status(ReturnStatusFilter::Status(ReturnStatusId(17650)))
            .emailaddress("a@example.com")
            .idcustomer(CustomerId(5))
            .created_after(moment)
            .updated_after(moment);
        assert_eq!(
            query.pairs(),
            vec![
                ("search", "R2026".to_owned()),
                ("status", "17650".to_owned()),
                ("emailaddress", "a@example.com".to_owned()),
                ("idcustomer", "5".to_owned()),
                ("created_after", "2026-11-09 16:00:00".to_owned()),
                ("updated_after", "2026-11-09 16:00:00".to_owned()),
            ]
        );
        assert_eq!(
            ReturnQuery::new()
                .status(ReturnStatusFilter::Processing)
                .pairs(),
            vec![("status", "processing".to_owned())]
        );
    }
}
