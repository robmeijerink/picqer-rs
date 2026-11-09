//! Fulfilment customers, for Picqer Fulfilment.
//!
//! A fulfilment customer is a party whose products a warehouse stores and ships. Picqer has no call to
//! delete one, so one that was made stays.
//!
//! The documentation describes the fields and its examples show the same ones. What a developer
//! account also reports but the documentation never shows, such as `active` and the number of
//! comments, is not read.

use std::fmt;

use reqwest::Method;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use zeroize::Zeroizing;

use crate::client::Client;
use crate::customers::Language;
use crate::error::{Result, ValidationError};
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{
    Date, DateTime, EmailAddress, FulfilmentCustomerId, LocationId, LocationTypeId,
    ShippingProviderProfileId, TemplateId, WarehouseId,
};

bounded_string! {
    /// The name of a fulfilment customer.
    FulfilmentCustomerName, "fulfilment customer name", 1, usize::MAX
}

bounded_string! {
    /// The username to log in to the fulfilment portal. Picqer requires it to be unique.
    FulfilmentUsername, "fulfilment username", 1, usize::MAX
}

/// The password to log in to the fulfilment portal. Picqer never reports it.
///
/// It is wiped from memory on drop and never shown by `Debug`.
#[derive(Clone)]
pub struct FulfilmentPassword(Zeroizing<String>);

impl FulfilmentPassword {
    /// Wraps a password, rejecting an empty one.
    pub fn new(password: impl Into<String>) -> std::result::Result<Self, ValidationError> {
        let password = Zeroizing::new(password.into());
        if password.is_empty() {
            return Err(ValidationError::new(
                "fulfilment password",
                "must not be empty",
            ));
        }
        Ok(Self(password))
    }
}

impl fmt::Debug for FulfilmentPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FulfilmentPassword([redacted])")
    }
}

/// A fulfilment customer.
///
/// A list does not have `allowed_shipping_profiles`, so it is empty there. An empty list for a single
/// customer means all profiles are allowed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct FulfilmentCustomer {
    /// The fulfilment customer.
    pub idfulfilment_customer: FulfilmentCustomerId,
    /// The name.
    pub name: String,
    /// Whether the customer can log in to the fulfilment portal.
    pub can_login: Option<bool>,
    /// The username for the portal.
    pub username: Option<String>,
    /// The email address. Picqer does not check it.
    pub emailaddress: Option<String>,
    /// The language of the portal.
    pub language: Option<Reported<Language>>,
    /// The template that is selected by default for new orders of this customer.
    pub idtemplate: Option<TemplateId>,
    /// When the customer last logged in to the portal.
    pub last_login_at: Option<DateTime>,
    /// The shipping profiles the customer may use.
    #[serde(default)]
    pub allowed_shipping_profiles: Vec<ShippingProviderProfileId>,
    /// The warehouse that is preferred for this customer. Without one the default warehouse is used.
    pub preferred_idwarehouse: Option<WarehouseId>,
}

/// A fulfilment customer to create.
///
/// Picqer requires a language, although its documentation says it is optional. It does not check the
/// email address, a username without a password or a login without a username, takes an unknown
/// shipping profile as none, and answers `201`.
#[derive(Clone)]
pub struct NewFulfilmentCustomer {
    fields: Map<String, Value>,
}

impl NewFulfilmentCustomer {
    /// A fulfilment customer with a name and the language of the portal.
    pub fn new(name: FulfilmentCustomerName, language: Language) -> Self {
        let mut fields = Map::new();
        fields.insert("name".into(), json!(name));
        fields.insert("language".into(), json!(language));
        Self { fields }
    }
}

macro_rules! customer_fields {
    () => {
        /// Sets whether the customer can log in to the portal.
        pub fn can_login(mut self, can_login: bool) -> Self {
            self.fields.insert("can_login".into(), json!(can_login));
            self
        }

        /// Sets the username for the portal.
        pub fn username(mut self, username: FulfilmentUsername) -> Self {
            self.fields.insert("username".into(), json!(username));
            self
        }

        /// Sets the password for the portal.
        pub fn password(mut self, password: &FulfilmentPassword) -> Self {
            self.fields
                .insert("password".into(), json!(password.0.as_str()));
            self
        }

        /// Sets the email address.
        pub fn emailaddress(mut self, emailaddress: EmailAddress) -> Self {
            self.fields
                .insert("emailaddress".into(), json!(emailaddress));
            self
        }

        /// Sets the template that is selected by default for new orders.
        pub fn idtemplate(mut self, idtemplate: TemplateId) -> Self {
            self.fields.insert("idtemplate".into(), json!(idtemplate));
            self
        }

        /// Sets the shipping profiles the customer may use. An empty list allows all of them.
        pub fn allowed_shipping_profiles(
            mut self,
            profiles: Vec<ShippingProviderProfileId>,
        ) -> Self {
            self.fields
                .insert("allowed_shipping_profiles".into(), json!(profiles));
            self
        }

        /// Sets the preferred warehouse.
        pub fn preferred_idwarehouse(mut self, warehouse: WarehouseId) -> Self {
            self.fields
                .insert("preferred_idwarehouse".into(), json!(warehouse));
            self
        }
    };
}

impl NewFulfilmentCustomer {
    customer_fields!();

    pub(crate) fn to_value(&self) -> Value {
        Value::Object(self.fields.clone())
    }
}

/// Changes to a fulfilment customer. Only what is set is sent.
#[derive(Clone, Default)]
pub struct FulfilmentCustomerChanges {
    fields: Map<String, Value>,
}

impl FulfilmentCustomerChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the name.
    pub fn name(mut self, name: FulfilmentCustomerName) -> Self {
        self.fields.insert("name".into(), json!(name));
        self
    }

    /// Changes the language of the portal.
    pub fn language(mut self, language: Language) -> Self {
        self.fields.insert("language".into(), json!(language));
        self
    }

    customer_fields!();

    /// Removes the username.
    pub fn without_username(mut self) -> Self {
        self.fields.insert("username".into(), Value::Null);
        self
    }

    /// Removes the email address.
    pub fn without_emailaddress(mut self) -> Self {
        self.fields.insert("emailaddress".into(), Value::Null);
        self
    }

    /// Removes the default template.
    pub fn without_idtemplate(mut self) -> Self {
        self.fields.insert("idtemplate".into(), Value::Null);
        self
    }

    /// Removes the preferred warehouse, so the default warehouse is used.
    pub fn without_preferred_idwarehouse(mut self) -> Self {
        self.fields
            .insert("preferred_idwarehouse".into(), Value::Null);
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        Value::Object(self.fields.clone())
    }
}

fn redacted(fields: &Map<String, Value>) -> Map<String, Value> {
    let mut shown = fields.clone();
    if let Some(password) = shown.get_mut("password") {
        *password = json!("[redacted]");
    }
    shown
}

impl fmt::Debug for NewFulfilmentCustomer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewFulfilmentCustomer")
            .field("fields", &redacted(&self.fields))
            .finish()
    }
}

impl fmt::Debug for FulfilmentCustomerChanges {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FulfilmentCustomerChanges")
            .field("fields", &redacted(&self.fields))
            .finish()
    }
}

/// The period of a report: the days from the first to the last, both included. Without a period
/// Picqer reports the current month.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReportPeriod {
    start: Option<Date>,
    end: Option<Date>,
}

impl ReportPeriod {
    /// The current month.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the report on a day.
    pub fn from(mut self, start: Date) -> Self {
        self.start = Some(start);
        self
    }

    /// Ends the report on a day.
    pub fn until(mut self, end: Date) -> Self {
        self.end = Some(end);
        self
    }

    pub(crate) fn pairs(self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(start) = self.start {
            pairs.push(("start-date", start.to_string()));
        }
        if let Some(end) = self.end {
            pairs.push(("end-date", end.to_string()));
        }
        pairs
    }
}

/// The period and the customer of a report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReportMeta {
    /// The fulfilment customer.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// The first day of the report.
    pub start_date: Option<Date>,
    /// The last day of the report.
    pub end_date: Option<Date>,
}

/// What came in.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReportInbound {
    /// How many receipts there were.
    pub receipts_count: Option<i64>,
    /// How many rows the receipts have.
    pub receipts_rows_count: Option<i64>,
    /// How many products were received.
    pub receipts_products_total_amount: Option<i64>,
}

/// What went out.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ReportOutbound {
    /// How many picklists there were.
    pub picklists_count: Option<i64>,
    /// How many rows the picklists have.
    pub picklist_rows_count: Option<i64>,
    /// How many products were on the picklists.
    pub picklist_products_count: Option<i64>,
    /// The shipments. The documentation only shows an empty list, so each is kept as Picqer sends
    /// it.
    #[serde(default)]
    pub shipments: Vec<Value>,
    /// How many shipments there were.
    pub shipments_count: Option<i64>,
    /// How many countries they went to.
    pub shipment_country_count: Option<i64>,
    /// How many shipping profiles were used.
    pub shipment_profile_count: Option<i64>,
}

/// A location that holds stock of the customer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LocationInUse {
    /// The location.
    pub idlocation: LocationId,
    /// The warehouse.
    pub idwarehouse: Option<WarehouseId>,
    /// The name of the warehouse.
    pub warehouse_name: Option<String>,
    /// The name of the location.
    pub stock_location: Option<String>,
    /// How many products are there.
    pub unique_products: Option<i64>,
    /// How many items are there. Picqer sends it as text.
    #[serde(default, deserialize_with = "crate::serde_util::opt_text")]
    pub total_items: Option<String>,
}

/// A type of location that holds stock of the customer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LocationTypeInUse {
    /// The warehouse.
    pub idwarehouse: Option<WarehouseId>,
    /// The name of the warehouse.
    pub warehouse_name: Option<String>,
    /// The location type.
    pub idlocation_type: Option<LocationTypeId>,
    /// The name of the type.
    pub name: Option<String>,
    /// How many locations of this type are in use.
    pub unique_locations: Option<i64>,
}

/// What is stored.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReportStorage {
    /// How many locations hold stock.
    pub stock_locations_in_use_count: Option<i64>,
    /// The locations that hold stock.
    #[serde(default)]
    pub stock_locations_in_use: Vec<LocationInUse>,
    /// The location types that hold stock.
    #[serde(default)]
    pub stock_location_types_in_use: Vec<LocationTypeInUse>,
    /// How many products there are.
    pub products_total: Option<i64>,
    /// How much stock there is.
    pub stock_total: Option<i64>,
}

/// What was returned.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReportReturns {
    /// How many returns there were.
    pub returns_count: Option<i64>,
    /// How many rows the returns have.
    pub return_rows_count: Option<i64>,
    /// How many products were returned.
    pub return_products_count: Option<i64>,
}

/// The statistics of a fulfilment customer over a period.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct FulfilmentReport {
    /// The customer and the period.
    pub meta_data: Option<ReportMeta>,
    /// What came in.
    pub inbound: Option<ReportInbound>,
    /// What went out.
    pub outbound: Option<ReportOutbound>,
    /// What is stored.
    pub storage_usage: Option<ReportStorage>,
    /// What was returned.
    pub returns: Option<ReportReturns>,
}

pub(crate) fn list(page: Page) -> Call<Vec<FulfilmentCustomer>> {
    Call::new(page.apply(Request::new(
        Method::GET,
        segments(&[&"fulfilment", &"customers"]),
    )))
}

pub(crate) fn get(id: FulfilmentCustomerId) -> Call<FulfilmentCustomer> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"fulfilment", &"customers", &id]),
    ))
}

pub(crate) fn create(customer: &NewFulfilmentCustomer) -> Result<Call<FulfilmentCustomer>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"fulfilment", &"customers"]))
            .json_body(&customer.to_value())?,
    ))
}

pub(crate) fn update(
    id: FulfilmentCustomerId,
    changes: &FulfilmentCustomerChanges,
) -> Result<Call<FulfilmentCustomer>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"fulfilment", &"customers", &id]))
            .json_body(&changes.to_value())?,
    ))
}

pub(crate) fn report(id: FulfilmentCustomerId, period: ReportPeriod) -> Call<FulfilmentReport> {
    let mut request = Request::new(
        Method::GET,
        segments(&[&"fulfilment", &"customers", &id, &"report"]),
    );
    for (name, value) in period.pairs() {
        request = request.query(name, value);
    }
    Call::new(request)
}

macro_rules! fulfilment_customers_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of fulfilment customers.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::fulfilment::FulfilmentCustomer>> {
            self.client.execute($crate::fulfilment::list(page)) $(.$await)?
        }

        /// Lists all fulfilment customers, following the pages.
        pub $($async)? fn list_all(
            &self,
        ) -> $crate::error::Result<Vec<$crate::fulfilment::FulfilmentCustomer>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::fulfilment::list(page)) $(.$await)?)
        }

        /// Fetches a fulfilment customer, with the shipping profiles it may use.
        pub $($async)? fn get(
            &self,
            id: $crate::types::FulfilmentCustomerId,
        ) -> $crate::error::Result<$crate::fulfilment::FulfilmentCustomer> {
            self.client.execute($crate::fulfilment::get(id)) $(.$await)?
        }

        /// Creates a fulfilment customer. Picqer has no call to delete one.
        pub $($async)? fn create(
            &self,
            customer: &$crate::fulfilment::NewFulfilmentCustomer,
        ) -> $crate::error::Result<$crate::fulfilment::FulfilmentCustomer> {
            self.client.execute($crate::fulfilment::create(customer)?) $(.$await)?
        }

        /// Changes a fulfilment customer and returns it.
        pub $($async)? fn update(
            &self,
            id: $crate::types::FulfilmentCustomerId,
            changes: &$crate::fulfilment::FulfilmentCustomerChanges,
        ) -> $crate::error::Result<$crate::fulfilment::FulfilmentCustomer> {
            self.client.execute($crate::fulfilment::update(id, changes)?) $(.$await)?
        }

        /// Fetches the statistics of a fulfilment customer over a period.
        pub $($async)? fn report(
            &self,
            id: $crate::types::FulfilmentCustomerId,
            period: $crate::fulfilment::ReportPeriod,
        ) -> $crate::error::Result<$crate::fulfilment::FulfilmentReport> {
            self.client.execute($crate::fulfilment::report(id, period)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use fulfilment_customers_methods;

/// Operations on fulfilment customers.
#[derive(Debug, Clone, Copy)]
pub struct FulfilmentCustomers<'a> {
    pub(crate) client: &'a Client,
}

impl FulfilmentCustomers<'_> {
    fulfilment_customers_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_customer_needs_a_name_and_a_language() {
        let new = NewFulfilmentCustomer::new(
            FulfilmentCustomerName::new("Fulfilling Partners").unwrap(),
            Language::Nl,
        );
        assert_eq!(
            new.to_value(),
            json!({"name": "Fulfilling Partners", "language": "nl"})
        );
        assert!(FulfilmentCustomerName::new("").is_err());
    }

    #[test]
    fn a_login_is_sent_but_the_password_is_not_shown() {
        let password = FulfilmentPassword::new("ue73hjdf893").unwrap();
        let new = NewFulfilmentCustomer::new(
            FulfilmentCustomerName::new("Partners").unwrap(),
            Language::En,
        )
        .can_login(true)
        .username(FulfilmentUsername::new("partners").unwrap())
        .password(&password)
        .allowed_shipping_profiles(vec![ShippingProviderProfileId(22)])
        .preferred_idwarehouse(WarehouseId(1));
        assert_eq!(
            new.to_value(),
            json!({
                "name": "Partners", "language": "en", "can_login": true, "username": "partners",
                "password": "ue73hjdf893", "allowed_shipping_profiles": [22],
                "preferred_idwarehouse": 1
            })
        );
        assert!(!format!("{password:?}").contains("ue73hjdf893"));
        assert!(!format!("{new:?}").contains("ue73hjdf893"));
        assert!(FulfilmentPassword::new("").is_err());
    }

    #[test]
    fn changes_can_remove_what_may_be_empty() {
        assert_eq!(FulfilmentCustomerChanges::new().to_value(), json!({}));
        let changes = FulfilmentCustomerChanges::new()
            .without_username()
            .without_emailaddress()
            .without_idtemplate()
            .without_preferred_idwarehouse()
            .allowed_shipping_profiles(vec![]);
        assert_eq!(
            changes.to_value(),
            json!({
                "username": null, "emailaddress": null, "idtemplate": null,
                "preferred_idwarehouse": null, "allowed_shipping_profiles": []
            })
        );
    }

    #[test]
    fn a_period_is_sent_as_two_dates() {
        assert!(ReportPeriod::new().pairs().is_empty());
        let period = ReportPeriod::new()
            .from("2026-11-01".parse().unwrap())
            .until("2026-11-09".parse().unwrap());
        assert_eq!(
            period.pairs(),
            vec![
                ("start-date", "2026-11-01".to_owned()),
                ("end-date", "2026-11-09".to_owned())
            ]
        );
    }
}
