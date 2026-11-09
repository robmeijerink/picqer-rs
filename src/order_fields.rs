//! Order fields.
//!
//! Picqer documents these as read-only: order fields are made in the app. They were checked against
//! a developer account with a field of every type. Whatever the documentation does not mark as
//! always present is optional.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::ValidationError;
use crate::fields::FieldType;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::OrderFieldId;

/// An extra data field that an order can contain.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct OrderField {
    /// The order field.
    pub idorderfield: OrderFieldId,
    /// The title of the field.
    pub title: Option<String>,
    /// The type of the field.
    #[serde(rename = "type")]
    pub field_type: Option<Reported<FieldType>>,
    /// The possible values of a radio, select, checkbox or multicheckbox field.
    pub values: Option<Vec<String>>,
    /// Whether the field is required for each order. Picqer does not enforce this when an order is
    /// made or processed through the API.
    pub required: Option<bool>,
    /// Whether the field shows on the picklist.
    pub visible_picklist: Option<bool>,
    /// Whether the field shows in the portal.
    pub visible_portal: Option<bool>,
    /// Whether the field can only be reached through the API.
    pub only_accessible_via_api: Option<bool>,
}

/// The value of an order field on an order, to set.
///
/// Picqer reports the value as text, also when a number was sent. It does not check the value of a
/// radio, select, checkbox or multicheckbox field against the values of the field, and stores any
/// text. An empty text removes the value from the order. [`OrderField::value`] checks that a radio,
/// select or checkbox value is a listed one, and [`OrderField::choices`] builds a multicheckbox
/// value, so build one from the field when you can.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OrderFieldValue {
    idorderfield: OrderFieldId,
    value: String,
}

impl OrderFieldValue {
    /// Sets an order field to a value, without checking it against the field.
    pub fn new(idorderfield: OrderFieldId, value: impl Into<String>) -> Self {
        Self {
            idorderfield,
            value: value.into(),
        }
    }

    pub(crate) fn body(&self) -> serde_json::Value {
        serde_json::json!({"value": self.value})
    }

    pub(crate) fn field(&self) -> OrderFieldId {
        self.idorderfield
    }
}

impl OrderField {
    /// Builds the value of this field for an order, checked against the field.
    ///
    /// A radio, select or checkbox field only takes one of its `values`: the app allows one for a
    /// checkbox. Picqer stores any text, so this is where a value that is not on the list is caught.
    pub fn value(&self, value: impl Into<String>) -> Result<OrderFieldValue, ValidationError> {
        let value = value.into();
        crate::fields::check_listed("order field value", &self.field_type, &self.values, &value)?;
        Ok(OrderFieldValue::new(self.idorderfield, value))
    }

    /// Builds the value of a multicheckbox field from the choices that are ticked.
    ///
    /// Every choice must be one of the `values` of the field. The Picqer app writes several choices
    /// as one text with a semicolon between them, so a choice cannot contain one. No choices at all
    /// give an empty text, which removes the value from the order.
    pub fn choices<I, S>(&self, choices: I) -> Result<OrderFieldValue, ValidationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let joined = crate::fields::join_choices(
            "order field choices",
            &self.field_type,
            &self.values,
            choices,
        )?;
        Ok(OrderFieldValue::new(
            self.idorderfield,
            joined.unwrap_or_default(),
        ))
    }
}

pub(crate) fn list(page: Page) -> Call<Vec<OrderField>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"orderfields"]))))
}

pub(crate) fn get(id: OrderFieldId) -> Call<OrderField> {
    Call::new(Request::new(Method::GET, segments(&[&"orderfields", &id])))
}

macro_rules! order_fields_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of order fields.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::order_fields::OrderField>> {
            self.client.execute($crate::order_fields::list(page)) $(.$await)?
        }

        /// Lists all order fields, following the pages.
        pub $($async)? fn list_all(
            &self,
        ) -> $crate::error::Result<Vec<$crate::order_fields::OrderField>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::order_fields::list(page)) $(.$await)?)
        }

        /// Fetches an order field.
        pub $($async)? fn get(
            &self,
            id: $crate::types::OrderFieldId,
        ) -> $crate::error::Result<$crate::order_fields::OrderField> {
            self.client.execute($crate::order_fields::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use order_fields_methods;

/// Operations on order fields.
#[derive(Debug, Clone, Copy)]
pub struct OrderFields<'a> {
    pub(crate) client: &'a Client,
}

impl OrderFields<'_> {
    order_fields_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(field_type: &str, values: &[&str]) -> OrderField {
        serde_json::from_value(serde_json::json!({
            "idorderfield": 4, "title": "Kind", "type": field_type, "values": values
        }))
        .unwrap()
    }

    #[test]
    fn a_select_field_only_takes_a_listed_value() {
        let select = field("select", &["a", "b"]);
        assert!(select.value("a").is_ok());
        assert!(select.value("c").is_err());
        assert!(field("text", &[]).value("anything").is_ok());
    }

    #[test]
    fn a_checkbox_takes_one_listed_value_and_a_multicheckbox_its_choices() {
        let checkbox = field("checkbox", &["active", "hidden"]);
        assert!(checkbox.value("active").is_ok());
        assert!(checkbox.value("active;hidden").is_err());
        let multi = field("multicheckbox", &["red", "blue"]);
        assert_eq!(
            serde_json::to_value(multi.choices(["red", "blue"]).unwrap()).unwrap(),
            serde_json::json!({"idorderfield": 4, "value": "red;blue"})
        );
        assert_eq!(
            serde_json::to_value(multi.choices([] as [&str; 0]).unwrap()).unwrap(),
            serde_json::json!({"idorderfield": 4, "value": ""})
        );
        assert!(multi.choices(["green"]).is_err());
        assert!(checkbox.choices(["active"]).is_err());
    }

    #[test]
    fn a_value_is_sent_as_text_with_its_field() {
        let value = OrderFieldValue::new(OrderFieldId::from(4), "x");
        assert_eq!(
            serde_json::to_value(&value).unwrap(),
            serde_json::json!({"idorderfield": 4, "value": "x"})
        );
        assert_eq!(value.body(), serde_json::json!({"value": "x"}));
    }
}
