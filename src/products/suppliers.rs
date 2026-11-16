use std::collections::BTreeMap;

use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::products::response::ProductSupplier;
use crate::types::{ProductId, ProductSupplierId, SupplierId};

bounded_string! {
    /// The code of a product at a supplier, which Picqer takes up to 255 characters of.
    ///
    /// Picqer stores an empty code as no code, so an empty one is refused here.
    SupplierProductCode, "supplier product code", 1, 255
}

/// A supplier to link to a product, with the purchase details of that supplier.
///
/// The first supplier of a product becomes its default supplier, whatever is asked. A further one
/// only becomes the default with [`NewProductSupplier::make_default`], and the supplier that was
/// the default stays linked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewProductSupplier {
    idsupplier: SupplierId,
    #[serde(skip_serializing_if = "Option::is_none")]
    productcode: Option<SupplierProductCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deliverytime: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minimum_purchase_quantity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    purchase_in_quantities_of: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_default: Option<bool>,
}

impl NewProductSupplier {
    /// Links a supplier. Inactive suppliers can be linked too.
    pub fn new(idsupplier: SupplierId) -> Self {
        Self {
            idsupplier,
            productcode: None,
            deliverytime: None,
            minimum_purchase_quantity: None,
            purchase_in_quantities_of: None,
            is_default: None,
        }
    }

    /// Sets the code of the product at the supplier.
    pub fn productcode(mut self, productcode: SupplierProductCode) -> Self {
        self.productcode = Some(productcode);
        self
    }

    /// Sets the delivery time from the supplier in days.
    pub fn deliverytime(mut self, days: u32) -> Self {
        self.deliverytime = Some(days);
        self
    }

    /// Sets the minimum amount to purchase from the supplier.
    pub fn minimum_purchase_quantity(mut self, quantity: u32) -> Self {
        self.minimum_purchase_quantity = Some(quantity);
        self
    }

    /// Sets the batch size to purchase in.
    pub fn purchase_in_quantities_of(mut self, quantity: u32) -> Self {
        self.purchase_in_quantities_of = Some(quantity);
        self
    }

    /// Makes this the default supplier of the product.
    pub fn make_default(mut self) -> Self {
        self.is_default = Some(true);
        self
    }
}

/// Changes to a supplier of a product. Only what is set is sent, and what is not sent stays as it
/// is.
///
/// The supplier itself cannot be changed: Picqer refuses it with `39`. To replace a supplier, link
/// the new one and remove the old one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProductSupplierChanges {
    fields: BTreeMap<&'static str, Value>,
}

impl ProductSupplierChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the code of the product at the supplier.
    pub fn productcode(mut self, productcode: SupplierProductCode) -> Self {
        self.fields.insert(
            "productcode",
            Value::String(productcode.as_str().to_owned()),
        );
        self
    }

    /// Removes the code of the product at the supplier.
    pub fn without_productcode(mut self) -> Self {
        self.fields.insert("productcode", Value::Null);
        self
    }

    /// Sets the delivery time from the supplier in days.
    pub fn deliverytime(mut self, days: u32) -> Self {
        self.fields.insert("deliverytime", Value::from(days));
        self
    }

    /// Removes the delivery time.
    pub fn without_deliverytime(mut self) -> Self {
        self.fields.insert("deliverytime", Value::Null);
        self
    }

    /// Sets the minimum amount to purchase from the supplier.
    pub fn minimum_purchase_quantity(mut self, quantity: u32) -> Self {
        self.fields
            .insert("minimum_purchase_quantity", Value::from(quantity));
        self
    }

    /// Removes the minimum amount to purchase.
    pub fn without_minimum_purchase_quantity(mut self) -> Self {
        self.fields.insert("minimum_purchase_quantity", Value::Null);
        self
    }

    /// Sets the batch size to purchase in.
    pub fn purchase_in_quantities_of(mut self, quantity: u32) -> Self {
        self.fields
            .insert("purchase_in_quantities_of", Value::from(quantity));
        self
    }

    /// Removes the batch size to purchase in.
    pub fn without_purchase_in_quantities_of(mut self) -> Self {
        self.fields.insert("purchase_in_quantities_of", Value::Null);
        self
    }

    /// Makes this the default supplier of the product, which keeps the supplier that was the
    /// default linked. Picqer has no way to take it away without making another one the default.
    pub fn make_default(mut self) -> Self {
        self.fields.insert("is_default", Value::Bool(true));
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        Value::Object(
            self.fields
                .iter()
                .map(|(name, value)| ((*name).to_owned(), value.clone()))
                .collect(),
        )
    }
}

pub(crate) fn suppliers(id: ProductId) -> Call<Vec<ProductSupplier>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"suppliers"]),
    ))
}

pub(crate) fn supplier(id: ProductId, supplier: ProductSupplierId) -> Call<ProductSupplier> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"suppliers", &supplier]),
    ))
}

pub(crate) fn add_supplier(
    id: ProductId,
    supplier: &NewProductSupplier,
) -> Result<Call<ProductSupplier>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"products", &id, &"suppliers"]))
            .json_body(supplier)?,
    ))
}

pub(crate) fn update_supplier(
    id: ProductId,
    supplier: ProductSupplierId,
    changes: &ProductSupplierChanges,
) -> Result<Call<ProductSupplier>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"products", &id, &"suppliers", &supplier]),
        )
        .json_body(&changes.to_value())?,
    ))
}

pub(crate) fn remove_supplier(id: ProductId, supplier: ProductSupplierId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"products", &id, &"suppliers", &supplier]),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_new_supplier_sends_what_is_set() {
        let supplier = NewProductSupplier::new(SupplierId(4521))
            .productcode(SupplierProductCode::new("SUP-6531-RB").unwrap())
            .deliverytime(5)
            .minimum_purchase_quantity(10)
            .purchase_in_quantities_of(5)
            .make_default();
        assert_eq!(
            serde_json::to_value(&supplier).unwrap(),
            json!({
                "idsupplier": 4521, "productcode": "SUP-6531-RB", "deliverytime": 5,
                "minimum_purchase_quantity": 10, "purchase_in_quantities_of": 5,
                "is_default": true
            })
        );
        assert_eq!(
            serde_json::to_value(NewProductSupplier::new(SupplierId(1))).unwrap(),
            json!({"idsupplier": 1})
        );
    }

    #[test]
    fn changes_send_only_what_is_set_and_null_to_clear() {
        assert_eq!(ProductSupplierChanges::new().to_value(), json!({}));
        let changes = ProductSupplierChanges::new()
            .deliverytime(3)
            .without_productcode()
            .without_deliverytime()
            .without_minimum_purchase_quantity()
            .without_purchase_in_quantities_of()
            .make_default();
        assert_eq!(
            changes.to_value(),
            json!({
                "deliverytime": null, "productcode": null, "minimum_purchase_quantity": null,
                "purchase_in_quantities_of": null, "is_default": true
            })
        );
    }

    #[test]
    fn a_code_is_one_to_255_characters() {
        assert!(SupplierProductCode::new("").is_err());
        assert!(SupplierProductCode::new("a").is_ok());
        assert!(SupplierProductCode::new("a".repeat(255)).is_ok());
        assert!(SupplierProductCode::new("a".repeat(256)).is_err());
    }
}
