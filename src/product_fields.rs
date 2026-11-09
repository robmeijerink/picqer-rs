//! Product fields.
//!
//! Picqer documents these as read-only: product fields are made in the app. They were checked
//! against a developer account with a field of every type. Whatever the documentation does not mark
//! as always present is optional.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::error::ValidationError;
use crate::fields::FieldType;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::products::{ProductFieldEntry, ProductFieldValue};
use crate::reported::Reported;
use crate::types::ProductFieldId;

use crate::fields::CHOICE_SEPARATOR;

impl ProductFieldEntry {
    /// The choices of a multicheckbox value, which Picqer writes with a semicolon between them.
    pub fn choices(&self) -> Vec<&str> {
        self.value
            .as_deref()
            .unwrap_or_default()
            .split(CHOICE_SEPARATOR)
            .filter(|choice| !choice.is_empty())
            .collect()
    }
}

/// An extra data field that a product can contain.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ProductField {
    /// The product field.
    pub idproductfield: ProductFieldId,
    /// The title of the field.
    pub title: Option<String>,
    /// The type of the field.
    #[serde(rename = "type")]
    pub field_type: Option<Reported<FieldType>>,
    /// The possible values of a radio, select, checkbox or multicheckbox field.
    pub values: Option<Vec<String>>,
    /// Whether the field is required for each product.
    pub required: Option<bool>,
    /// Whether the field is shown on the picklist.
    pub visible_picklist: Option<bool>,
    /// Whether the field is shown on the invoice.
    pub visible_invoice: Option<bool>,
    /// Whether the field is shown on the shipping list.
    pub visible_shippinglist: Option<bool>,
    /// Whether the field is shown on the portal.
    pub visible_portal: Option<bool>,
    /// Whether the field is shown on the purchase order.
    pub visible_purchase_order: Option<bool>,
}

impl ProductField {
    /// Builds the value of this field for a product, checked against the field.
    ///
    /// A radio, select or checkbox field only takes one of its `values`: the app allows one for a
    /// checkbox. Picqer does not check that and stores any text, so this is where a value that is
    /// not on the list is caught. Other types take any text. Use [`choices`](Self::choices) for a
    /// multicheckbox field.
    pub fn value(&self, value: impl Into<String>) -> Result<ProductFieldValue, ValidationError> {
        let value = value.into();
        crate::fields::check_listed(
            "product field value",
            &self.field_type,
            &self.values,
            &value,
        )?;
        Ok(ProductFieldValue::new(self.idproductfield, value))
    }

    /// Builds the value of a multicheckbox field from the choices that are ticked.
    ///
    /// Every choice must be one of the `values` of the field. Picqer writes several choices as one
    /// text with a semicolon between them, so a choice cannot contain one. No choices at all
    /// removes the value.
    pub fn choices<I, S>(&self, choices: I) -> Result<ProductFieldValue, ValidationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Ok(
            match crate::fields::join_choices(
                "product field choices",
                &self.field_type,
                &self.values,
                choices,
            )? {
                Some(joined) => ProductFieldValue::new(self.idproductfield, joined),
                None => self.cleared(),
            },
        )
    }

    /// Builds the removal of the value of this field from a product.
    pub fn cleared(&self) -> ProductFieldValue {
        ProductFieldValue::cleared(self.idproductfield)
    }
}

pub(crate) fn list(page: Page) -> Call<Vec<ProductField>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"productfields"]))))
}

pub(crate) fn get(id: ProductFieldId) -> Call<ProductField> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"productfields", &id]),
    ))
}

macro_rules! product_fields_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of product fields.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::product_fields::ProductField>> {
            self.client.execute($crate::product_fields::list(page)) $(.$await)?
        }

        /// Lists all product fields, following the pages.
        pub $($async)? fn list_all(
            &self,
        ) -> $crate::error::Result<Vec<$crate::product_fields::ProductField>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::product_fields::list(page)) $(.$await)?)
        }

        /// Fetches a product field.
        pub $($async)? fn get(
            &self,
            id: $crate::types::ProductFieldId,
        ) -> $crate::error::Result<$crate::product_fields::ProductField> {
            self.client.execute($crate::product_fields::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use product_fields_methods;

/// Operations on product fields.
#[derive(Debug, Clone, Copy)]
pub struct ProductFields<'a> {
    pub(crate) client: &'a Client,
}

impl ProductFields<'_> {
    product_fields_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(field_type: &str, values: &[&str]) -> ProductField {
        serde_json::from_value(serde_json::json!({
            "idproductfield": 6243, "title": "T", "type": field_type, "values": values
        }))
        .unwrap()
    }

    #[test]
    fn a_radio_field_takes_only_its_values() {
        let radio = field("radio", &["none", "few", "many"]);
        assert!(radio.value("few").is_ok());
        assert!(radio.value("nope").is_err());
        assert!(radio.value("").is_err());
    }

    #[test]
    fn a_text_field_takes_any_text() {
        let text = field("text", &[]);
        assert!(text.value("anything").is_ok());
        assert!(text.value("").is_ok());
    }

    #[test]
    fn a_select_field_takes_only_its_values() {
        let select = field("select", &["none", "low", "high"]);
        assert!(select.value("low").is_ok());
        assert!(select.value("extreme").is_err());
    }

    #[test]
    fn a_checkbox_takes_one_listed_value_and_a_multicheckbox_any_text() {
        let checkbox = field("checkbox", &["Active", "Once"]);
        assert!(checkbox.value("Active").is_ok());
        assert!(checkbox.value("Active;Once").is_err());
        assert!(field("multicheckbox", &["a"]).value("a;b").is_ok());
    }

    #[test]
    fn a_type_that_is_not_known_takes_any_text() {
        assert!(field("barcode", &["a"]).value("b").is_ok());
    }

    fn multicheckbox() -> ProductField {
        field("multicheckbox", &["netherlands", "belgium", "luxembourg"])
    }

    #[test]
    fn choices_are_written_the_way_the_app_writes_them() {
        let value = multicheckbox()
            .choices(["netherlands", "belgium", "luxembourg"])
            .unwrap();
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            serde_json::json!({"idproductfield": 6243, "value": "netherlands;belgium;luxembourg"})
        );
    }

    #[test]
    fn choices_must_be_listed_and_are_not_repeated() {
        assert!(multicheckbox().choices(["netherlands", "germany"]).is_err());
        let value = multicheckbox().choices(["belgium", "belgium"]).unwrap();
        assert_eq!(
            serde_json::to_value(value).unwrap()["value"],
            serde_json::json!("belgium")
        );
    }

    #[test]
    fn no_choices_remove_the_value() {
        let none: [&str; 0] = [];
        assert_eq!(
            serde_json::to_value(multicheckbox().choices(none).unwrap()).unwrap(),
            serde_json::json!({"idproductfield": 6243, "value": null})
        );
    }

    #[test]
    fn only_a_multicheckbox_takes_choices() {
        assert!(field("radio", &["a"]).choices(["a"]).is_err());
        assert!(field("checkbox", &["a"]).choices(["a"]).is_err());
    }

    #[test]
    fn a_choice_cannot_contain_the_separator() {
        assert!(field("multicheckbox", &["a;b"]).choices(["a;b"]).is_err());
    }

    #[test]
    fn a_value_can_be_removed() {
        assert_eq!(
            serde_json::to_value(field("text", &[]).cleared()).unwrap(),
            serde_json::json!({"idproductfield": 6243, "value": null})
        );
    }
}
