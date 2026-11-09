use serde::Serialize;

use crate::country::CountryCode;
use crate::pricelists::PricelistPrice;
use crate::products::response::ProductType;
use crate::types::{
    Centimeters, FulfilmentCustomerId, Grams, MidCode, Money, ProductCode, ProductFieldId,
    ProductName, SupplierId, VatGroupId,
};

/// The fields of a product that are optional both when creating and when changing one.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
struct Details {
    #[serde(skip_serializing_if = "Option::is_none")]
    fixedstockprice: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idsupplier: Option<SupplierId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    productcode_supplier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deliverytime: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    barcode: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    product_type: Option<ProductType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weight: Option<Grams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<Centimeters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<Centimeters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<Centimeters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minimum_purchase_quantity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    purchase_in_quantities_of: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hs_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country_of_origin: Option<CountryCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mid_code: Option<MidCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    track_lot_numbers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    track_expiration_dates: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idfulfilment_customer: Option<FulfilmentCustomerId>,
}

macro_rules! product_setters {
    () => {
        detail_setters! {
            /// Sets the fixed stock price, excluding VAT.
            fixedstockprice: Money,
            /// Sets the default supplier.
            idsupplier: SupplierId,
            /// Sets the code of the product at the supplier.
            productcode_supplier: String,
            /// Sets the delivery time from the supplier in days.
            deliverytime: u32,
            /// Sets the description.
            description: String,
            /// Sets the barcode, usually GS1, EAN13 or UPC.
            barcode: String,
            /// Sets how Picqer treats the stock of the product.
            product_type: ProductType,
            /// Sets the weight.
            weight: Grams,
            /// Sets the length.
            length: Centimeters,
            /// Sets the width.
            width: Centimeters,
            /// Sets the height.
            height: Centimeters,
            /// Sets the minimum amount to purchase from the supplier.
            minimum_purchase_quantity: u32,
            /// Sets the batch size to purchase in.
            purchase_in_quantities_of: u32,
            /// Sets the HS code for customs.
            hs_code: String,
            /// Sets the country of origin for customs.
            country_of_origin: CountryCode,
            /// Sets the Manufacturer Identification Code for customs.
            mid_code: MidCode,
            /// Sets whether a lot number is required when stock is registered. Only available when
            /// traceability is enabled for the account.
            track_lot_numbers: bool,
            /// Sets whether an expiration date is required when stock is registered. Only
            /// available when traceability is enabled for the account.
            track_expiration_dates: bool,
            /// Sets the fulfilment customer, with Picqer Fulfilment.
            idfulfilment_customer: FulfilmentCustomerId,
        }
    };
}

/// A product to create.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewProduct {
    idvatgroup: VatGroupId,
    productcode: ProductCode,
    name: ProductName,
    price: Money,
    #[serde(flatten)]
    details: Details,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pricelists: Vec<PricelistPrice>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    productfields: Vec<ProductFieldValue>,
}

impl NewProduct {
    /// A product with the fields Picqer requires.
    pub fn new(
        idvatgroup: VatGroupId,
        productcode: ProductCode,
        name: ProductName,
        price: Money,
    ) -> Self {
        Self {
            idvatgroup,
            productcode,
            name,
            price,
            details: Details::default(),
            pricelists: Vec::new(),
            productfields: Vec::new(),
        }
    }

    /// Sets the price of the product in pricelists.
    pub fn pricelists(mut self, pricelists: Vec<PricelistPrice>) -> Self {
        self.pricelists = pricelists;
        self
    }

    /// Sets the values of product fields.
    pub fn productfields(mut self, productfields: Vec<ProductFieldValue>) -> Self {
        self.productfields = productfields;
        self
    }

    product_setters!();
}

/// The value of a product field on a product, to set or to clear.
///
/// Picqer reports the value as text, also when a number was sent. Picqer does not check a value
/// against the values of a radio, select, checkbox or multicheckbox field. [`ProductField::value`]
/// checks a radio, select or checkbox value and [`ProductField::choices`] builds a multicheckbox
/// value, so build one from the field when you can.
///
/// [`ProductField::value`]: crate::product_fields::ProductField::value
/// [`ProductField::choices`]: crate::product_fields::ProductField::choices
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductFieldValue {
    idproductfield: ProductFieldId,
    value: Option<String>,
}

impl ProductFieldValue {
    /// Sets a product field to a value, without checking it against the field.
    pub fn new(idproductfield: ProductFieldId, value: impl Into<String>) -> Self {
        Self {
            idproductfield,
            value: Some(value.into()),
        }
    }

    /// Removes the value of a product field from the product.
    pub fn cleared(idproductfield: ProductFieldId) -> Self {
        Self {
            idproductfield,
            value: None,
        }
    }
}

/// Changes to a product. Only what is set is sent.
///
/// The active state is changed with `activate` and `inactivate`, not here.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ProductChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    idvatgroup: Option<VatGroupId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    productcode: Option<ProductCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<ProductName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pricelists: Option<Vec<PricelistPrice>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    productfields: Option<Vec<ProductFieldValue>>,
    #[serde(flatten)]
    details: Details,
}

impl ProductChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves the product to another VAT group.
    pub fn idvatgroup(mut self, idvatgroup: VatGroupId) -> Self {
        self.idvatgroup = Some(idvatgroup);
        self
    }

    /// Changes the code of the product, which must stay unique.
    pub fn productcode(mut self, productcode: ProductCode) -> Self {
        self.productcode = Some(productcode);
        self
    }

    /// Changes the name.
    pub fn name(mut self, name: ProductName) -> Self {
        self.name = Some(name);
        self
    }

    /// Changes the price, excluding VAT.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    /// Sets the price of the product in pricelists.
    pub fn pricelists(mut self, pricelists: Vec<PricelistPrice>) -> Self {
        self.pricelists = Some(pricelists);
        self
    }

    /// Sets the values of product fields.
    pub fn productfields(mut self, productfields: Vec<ProductFieldValue>) -> Self {
        self.productfields = Some(productfields);
        self
    }

    product_setters!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn product() -> NewProduct {
        NewProduct::new(
            VatGroupId(7),
            ProductCode::new("A-1").unwrap(),
            ProductName::new("A product").unwrap(),
            Money::from_euros(12).unwrap(),
        )
    }

    #[test]
    fn a_new_product_sends_the_required_fields_only() {
        assert_eq!(
            serde_json::to_value(product()).unwrap(),
            serde_json::json!({
                "idvatgroup": 7, "productcode": "A-1", "name": "A product", "price": 12.0
            })
        );
    }

    #[test]
    fn optional_fields_are_sent_under_their_wire_names() {
        let value = serde_json::to_value(
            product()
                .product_type(ProductType::UnlimitedStock)
                .weight(Grams::new(250))
                .country_of_origin(CountryCode::NL)
                .deliverytime(3),
        )
        .unwrap();
        assert_eq!(value["type"], "unlimited_stock");
        assert_eq!(value["weight"], 250);
        assert_eq!(value["country_of_origin"], "NL");
        assert_eq!(value["deliverytime"], 3);
    }

    #[test]
    fn a_change_sends_only_what_is_set() {
        assert_eq!(serde_json::to_string(&ProductChanges::new()).unwrap(), "{}");
        let changes = ProductChanges::new()
            .name(ProductName::new("New").unwrap())
            .barcode("123".to_owned());
        assert_eq!(
            serde_json::to_value(changes).unwrap(),
            serde_json::json!({"name": "New", "barcode": "123"})
        );
    }
}
