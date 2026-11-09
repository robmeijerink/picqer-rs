//! Customer fields.
//!
//! Picqer documents these as read-only: customer fields are made in the app. They were checked
//! against a developer account with a field of every type. Whatever the documentation does not mark as
//! always present is optional.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::customers::CustomerFieldValue;
use crate::error::ValidationError;
use crate::fields::FieldType;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::CustomerFieldId;

/// An extra data field that a customer can contain.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomerField {
    /// The customer field.
    pub idcustomerfield: CustomerFieldId,
    /// The title of the field.
    pub title: Option<String>,
    /// The type of the field.
    #[serde(rename = "type")]
    pub field_type: Option<Reported<FieldType>>,
    /// The possible values of a radio, select, checkbox or multicheckbox field.
    pub values: Option<Vec<String>>,
    /// Whether the field is required for each customer. Picqer does not enforce this when a
    /// customer is made or changed through the API.
    pub required: Option<bool>,
}

impl CustomerField {
    /// Builds the value of this field for a customer, checked against the field.
    ///
    /// A radio, select or checkbox field only takes one of its `values`: the app allows one for a
    /// checkbox. Picqer stores any text, so this is where a value that is not on the list is caught.
    pub fn value(&self, value: impl Into<String>) -> Result<CustomerFieldValue, ValidationError> {
        let value = value.into();
        crate::fields::check_listed(
            "customer field value",
            &self.field_type,
            &self.values,
            &value,
        )?;
        Ok(CustomerFieldValue::new(self.idcustomerfield, value))
    }

    /// Builds the value of a multicheckbox field from the choices that are ticked.
    ///
    /// Every choice must be one of the `values` of the field. The Picqer app writes several choices
    /// as one text with a semicolon between them, so a choice cannot contain one. No choices at all
    /// remove the value from the customer.
    pub fn choices<I, S>(&self, choices: I) -> Result<CustomerFieldValue, ValidationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let joined = crate::fields::join_choices(
            "customer field choices",
            &self.field_type,
            &self.values,
            choices,
        )?;
        Ok(match joined {
            Some(joined) => CustomerFieldValue::new(self.idcustomerfield, joined),
            None => self.cleared(),
        })
    }

    /// Builds the removal of the value of this field from a customer.
    pub fn cleared(&self) -> CustomerFieldValue {
        CustomerFieldValue::cleared(self.idcustomerfield)
    }
}

pub(crate) fn list(page: Page) -> Call<Vec<CustomerField>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"customerfields"]))))
}

pub(crate) fn get(id: CustomerFieldId) -> Call<CustomerField> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"customerfields", &id]),
    ))
}

macro_rules! customer_fields_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of customer fields.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::customer_fields::CustomerField>> {
            self.client.execute($crate::customer_fields::list(page)) $(.$await)?
        }

        /// Lists all customer fields, following the pages.
        pub $($async)? fn list_all(
            &self,
        ) -> $crate::error::Result<Vec<$crate::customer_fields::CustomerField>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::customer_fields::list(page)) $(.$await)?)
        }

        /// Fetches a customer field.
        pub $($async)? fn get(
            &self,
            id: $crate::types::CustomerFieldId,
        ) -> $crate::error::Result<$crate::customer_fields::CustomerField> {
            self.client.execute($crate::customer_fields::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use customer_fields_methods;

/// Operations on customer fields.
#[derive(Debug, Clone, Copy)]
pub struct CustomerFields<'a> {
    pub(crate) client: &'a Client,
}

impl CustomerFields<'_> {
    customer_fields_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(field_type: &str, values: &[&str]) -> CustomerField {
        serde_json::from_value(serde_json::json!({
            "idcustomerfield": 4, "title": "Kind", "type": field_type, "values": values
        }))
        .unwrap()
    }

    #[test]
    fn a_checkbox_takes_one_listed_value_and_a_multicheckbox_its_choices() {
        let checkbox = field("checkbox", &["active", "hidden"]);
        assert!(checkbox.value("hidden").is_ok());
        assert!(checkbox.value("active;hidden").is_err());
        let multi = field("multicheckbox", &["red", "blue"]);
        assert_eq!(
            serde_json::to_value(multi.choices(["red", "blue"]).unwrap()).unwrap(),
            serde_json::json!({"idcustomerfield": 4, "value": "red;blue"})
        );
        assert_eq!(
            serde_json::to_value(multi.choices([] as [&str; 0]).unwrap()).unwrap(),
            serde_json::json!({"idcustomerfield": 4, "value": null})
        );
        assert!(multi.choices(["green"]).is_err());
    }

    #[test]
    fn a_radio_and_a_select_take_only_listed_values_and_text_takes_any() {
        assert!(field("radio", &["a"]).value("b").is_err());
        assert!(field("select", &["a"]).value("a").is_ok());
        assert!(field("text", &[]).value("anything").is_ok());
        assert!(field("textarea", &[]).value("two\nlines").is_ok());
    }
}
