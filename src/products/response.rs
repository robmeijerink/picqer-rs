use std::collections::BTreeMap;

use serde::Deserialize;
use url::Url;

use crate::country::CountryCode;
use crate::reported::Reported;
use crate::stock::StockLevel;
use crate::tags::Tag;
use crate::types::{
    Centimeters, DateTime, FulfilmentCustomerId, Grams, Money, ProductFieldId, ProductId,
    SupplierId, VatGroupId,
};

use crate::pricelists::PricelistPrice;

string_enum! {
    /// How Picqer treats the stock of a product.
    pub enum ProductType {
        /// A product with stock. This is the default.
        Normal = "normal",
        /// A product whose stock is not monitored.
        UnlimitedStock = "unlimited_stock",
        /// A composition of other products that has no stock of its own.
        VirtualComposition = "virtual_composition",
        /// A composition of other products that has stock of its own.
        CompositionWithStock = "composition_with_stock",
    }
}

string_enum! {
    /// The ABC classification of a product, based on how often it is picked.
    pub enum AbcClassification {
        /// Picked the most.
        A = "A",
        /// Picked moderately.
        B = "B",
        /// Picked the least.
        C = "C",
    }
}

/// A supplier of a product, with the terms of that supplier.
///
/// The attribute table and the examples of the documentation do not list this, but the API reports
/// it. A product without a supplier still has one entry, for the default supplier, and that entry
/// has no supplier.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ProductSupplier {
    /// The supplier, which is missing in the entry of a product that has none.
    pub idsupplier: Option<SupplierId>,
    /// The code of the product at the supplier.
    pub productcode: Option<String>,
    /// The delivery time in days.
    pub deliverytime: Option<i64>,
    /// The minimum amount to purchase.
    pub minimum_purchase_quantity: Option<i64>,
    /// The batch size to purchase in.
    pub purchase_in_quantities_of: Option<i64>,
    /// Whether this is the default supplier of the product.
    pub is_default: Option<bool>,
}

/// The value of a product field on a product.
///
/// Picqer only reports the fields that have a value. The value is always text: a number that was
/// sent is reported as its text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ProductFieldEntry {
    /// The product field.
    pub idproductfield: ProductFieldId,
    /// The title of the field.
    pub title: Option<String>,
    /// The value.
    pub value: Option<String>,
}

/// A product in the catalogue.
///
/// `created`, `updated`, `tags`, `images` and `stock` are present in every response. What the
/// attribute table calls optional is optional here, even where the API always fills it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Product {
    /// The product.
    pub idproduct: ProductId,
    /// The VAT group of the product.
    pub idvatgroup: VatGroupId,
    /// The name of the product.
    pub name: String,
    /// The price, excluding VAT.
    pub price: Money,
    /// The code of the product, which is unique.
    pub productcode: String,
    /// When the product was created.
    pub created: DateTime,
    /// When the product was last changed.
    pub updated: DateTime,
    /// The tags of the product, by title.
    #[serde(deserialize_with = "crate::serde_util::tag_map")]
    pub tags: BTreeMap<String, Tag>,
    /// The urls of the images of the product.
    pub images: Vec<Url>,
    /// The stock of the product per warehouse.
    pub stock: Vec<StockLevel>,
    /// The prices of the product in pricelists.
    #[serde(default)]
    pub pricelists: Vec<PricelistPrice>,
    /// The values of the product fields that have one.
    #[serde(default)]
    pub productfields: Vec<ProductFieldEntry>,
    /// The default supplier.
    pub idsupplier: Option<SupplierId>,
    /// The fixed stock price, excluding VAT.
    pub fixedstockprice: Option<Money>,
    /// The code of the product at the supplier.
    pub productcode_supplier: Option<String>,
    /// The delivery time from the supplier in days.
    pub deliverytime: Option<i64>,
    /// The description of the product.
    pub description: Option<String>,
    /// The barcode, usually GS1, EAN13 or UPC.
    pub barcode: Option<String>,
    /// How Picqer treats the stock of the product.
    #[serde(rename = "type")]
    pub product_type: Option<Reported<ProductType>>,
    /// Deprecated in favor of the type.
    pub unlimitedstock: Option<bool>,
    /// The weight.
    pub weight: Option<Grams>,
    /// The length. Not available for virtual compositions.
    pub length: Option<Centimeters>,
    /// The width. Not available for virtual compositions.
    pub width: Option<Centimeters>,
    /// The height. Not available for virtual compositions.
    pub height: Option<Centimeters>,
    /// The minimum amount to purchase from the supplier.
    pub minimum_purchase_quantity: Option<i64>,
    /// The batch size to purchase in.
    pub purchase_in_quantities_of: Option<i64>,
    /// The HS code for customs.
    pub hs_code: Option<String>,
    /// The country of origin for customs.
    ///
    /// The API does not check this, so it can be a code this crate does not know.
    pub country_of_origin: Option<Reported<CountryCode>>,
    /// The Manufacturer Identification Code for customs.
    pub mid_code: Option<String>,
    /// Whether the product is active.
    pub active: Option<bool>,
    /// Whether a lot number is required when stock is registered.
    pub track_lot_numbers: Option<bool>,
    /// Whether an expiration date is required when stock is registered.
    pub track_expiration_dates: Option<bool>,
    /// The fulfilment customer the product belongs to, with Picqer Fulfilment.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// The average amount picked per day in the last 28 days.
    ///
    /// The documentation shows both a number and a text for this field, so both are read.
    #[serde(default, deserialize_with = "crate::serde_util::opt_number_or_text")]
    pub analysis_pick_amount_per_day: Option<f64>,
    /// The classification of the product, updated once every week.
    pub analysis_abc_classification: Option<Reported<AbcClassification>>,
    /// Whether the product is assembled.
    pub assembled: Option<bool>,
    /// The number of comments on the product.
    pub comment_count: Option<i64>,
    /// Whether the product is shown on the portal.
    pub show_on_portal: Option<bool>,
    /// The suppliers of the product.
    pub suppliers: Option<Vec<ProductSupplier>>,
}
