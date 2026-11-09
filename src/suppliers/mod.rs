//! Suppliers.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::country::CountryCode;
use crate::customers::Language;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{EmailAddress, SupplierId};

bounded_string! {
    /// The name of a supplier, which Picqer requires to be unique.
    SupplierName, "supplier name", 1, usize::MAX
}

/// A supplier of products.
///
/// Picqer also reports `active`, `idtemplate`, `minimum_order_total`, `full_address` and
/// `comment_count`, which the documentation does not describe. Those are not read here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Supplier {
    /// The supplier.
    pub idsupplier: SupplierId,
    /// The name.
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
    /// The country. Picqer keeps the code as it was given, so it can be one this crate does not
    /// know.
    pub country: Option<Reported<CountryCode>>,
    /// The phone number.
    pub telephone: Option<String>,
    /// The email address.
    pub emailaddress: Option<String>,
    /// Remarks.
    pub remarks: Option<String>,
    /// The language used for what is sent to the supplier.
    pub language: Option<Reported<Language>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
struct Details {
    #[serde(skip_serializing_if = "Option::is_none")]
    contactname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    zipcode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<CountryCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    telephone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emailaddress: Option<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remarks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<Language>,
}

macro_rules! supplier_setters {
    () => {
        detail_setters! {
            /// Sets the contact person.
            contactname: String,
            /// Sets the street and number.
            address: String,
            /// Sets the second address line.
            address2: String,
            /// Sets the postal code.
            zipcode: String,
            /// Sets the city.
            city: String,
            /// Sets the region, state or province.
            region: String,
            /// Sets the country. Picqer does not check it, this crate does.
            country: CountryCode,
            /// Sets the phone number.
            telephone: String,
            /// Sets the email address.
            emailaddress: EmailAddress,
            /// Sets remarks.
            remarks: String,
            /// Sets the language. Picqer replaces a language it does not offer with `en`, so only
            /// the two it offers can be sent.
            language: Language,
        }
    };
}

/// A supplier to create.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewSupplier {
    name: SupplierName,
    #[serde(flatten)]
    details: Details,
}

impl NewSupplier {
    /// A supplier with the field Picqer requires.
    pub fn new(name: SupplierName) -> Self {
        Self {
            name,
            details: Details::default(),
        }
    }

    supplier_setters!();
}

/// Changes to a supplier. Only what is set is sent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SupplierChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<SupplierName>,
    #[serde(flatten)]
    details: Details,
}

impl SupplierChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the name, which must stay unique.
    pub fn name(mut self, name: SupplierName) -> Self {
        self.name = Some(name);
        self
    }

    supplier_setters!();
}

pub(crate) fn list(page: Page) -> Call<Vec<Supplier>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"suppliers"]))))
}

pub(crate) fn get(id: SupplierId) -> Call<Supplier> {
    Call::new(Request::new(Method::GET, segments(&[&"suppliers", &id])))
}

pub(crate) fn create(supplier: &NewSupplier) -> Result<Call<Supplier>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"suppliers"])).json_body(supplier)?,
    ))
}

pub(crate) fn update(id: SupplierId, changes: &SupplierChanges) -> Result<Call<Supplier>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"suppliers", &id])).json_body(changes)?,
    ))
}

macro_rules! suppliers_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of suppliers.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::suppliers::Supplier>> {
            self.client.execute($crate::suppliers::list(page)) $(.$await)?
        }

        /// Lists all suppliers, following the pages.
        pub $($async)? fn list_all(&self) -> $crate::error::Result<Vec<$crate::suppliers::Supplier>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::suppliers::list(page)) $(.$await)?)
        }

        /// Fetches a supplier.
        pub $($async)? fn get(
            &self,
            id: $crate::types::SupplierId,
        ) -> $crate::error::Result<$crate::suppliers::Supplier> {
            self.client.execute($crate::suppliers::get(id)) $(.$await)?
        }

        /// Creates a supplier. Picqer cannot delete a supplier.
        pub $($async)? fn create(
            &self,
            supplier: &$crate::suppliers::NewSupplier,
        ) -> $crate::error::Result<$crate::suppliers::Supplier> {
            self.client.execute($crate::suppliers::create(supplier)?) $(.$await)?
        }

        /// Changes a supplier.
        pub $($async)? fn update(
            &self,
            id: $crate::types::SupplierId,
            changes: &$crate::suppliers::SupplierChanges,
        ) -> $crate::error::Result<$crate::suppliers::Supplier> {
            self.client.execute($crate::suppliers::update(id, changes)?) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use suppliers_methods;

/// Operations on suppliers.
#[derive(Debug, Clone, Copy)]
pub struct Suppliers<'a> {
    pub(crate) client: &'a Client,
}

impl Suppliers<'_> {
    suppliers_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_supplier_sends_the_name_and_what_was_set() {
        let supplier = NewSupplier::new(SupplierName::new("Example").unwrap())
            .country(CountryCode::BE)
            .language(Language::En)
            .city("Stad".to_owned());
        assert_eq!(
            serde_json::to_value(supplier).unwrap(),
            serde_json::json!({
                "name": "Example", "city": "Stad", "country": "BE", "language": "en"
            })
        );
    }

    #[test]
    fn changes_send_only_what_is_set() {
        assert_eq!(
            serde_json::to_string(&SupplierChanges::new()).unwrap(),
            "{}"
        );
        assert!(SupplierName::new("").is_err());
    }
}
