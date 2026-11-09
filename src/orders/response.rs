use std::collections::BTreeMap;

use serde::Deserialize;

use crate::country::CountryCode;
use crate::customers::Language;
use crate::picklists::Picklist;
use crate::reported::Reported;
use crate::tags::Tag;
use crate::types::{
    CustomerId, Date, DateTime, FulfilmentCustomerId, Money, OrderFieldId, OrderId, OrderProductId,
    Percentage, PricelistId, ProductId, ShippingProviderProfileId, TemplateId, VatGroupId,
    WarehouseId,
};

string_enum! {
    /// Where an order is in its life.
    pub enum OrderStatus {
        /// Being prepared. A concept can still be changed.
        Concept = "concept",
        /// Placed, but waiting before it is processed.
        Expected = "expected",
        /// Being fulfilled. It has picklists or backorders.
        Processing = "processing",
        /// Put on hold.
        Paused = "paused",
        /// Fulfilled.
        Completed = "completed",
        /// Cancelled.
        Cancelled = "cancelled",
    }
}

/// A line of an order.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct OrderProduct {
    /// The line.
    pub idorder_product: OrderProductId,
    /// The product.
    pub idproduct: ProductId,
    /// How many were ordered.
    pub amount: i64,
    /// The code of the product.
    pub productcode: Option<String>,
    /// The name of the product.
    pub name: Option<String>,
    /// Remarks about the line.
    pub remarks: Option<String>,
    /// The price per piece, excluding VAT.
    pub price: Option<Money>,
    /// The VAT group of the line.
    pub idvatgroup: Option<VatGroupId>,
    /// How many were cancelled.
    pub amount_cancelled: Option<i64>,
    /// The weight in grams.
    pub weight: Option<i64>,
    /// The line this line is a part of, when the product is a composition.
    pub partof_idorder_product: Option<OrderProductId>,
    /// Whether the product has parts.
    pub has_parts: Option<bool>,
}

/// The value of an order field on an order.
///
/// Picqer only reports the fields that have a value. The value is always text: a number that was
/// sent is reported as its text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct OrderFieldEntry {
    /// The order field.
    pub idorderfield: OrderFieldId,
    /// The title of the field.
    pub title: Option<String>,
    /// The value.
    #[serde(default, deserialize_with = "crate::serde_util::opt_text")]
    pub value: Option<String>,
}

/// How far a line of an order has come.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct OrderLineStatus {
    /// The product.
    pub idproduct: ProductId,
    /// The code of the product.
    pub productcode: Option<String>,
    /// The name of the product.
    pub name: Option<String>,
    /// How many were ordered.
    pub amount: Option<i64>,
    /// How many were delivered.
    pub delivered: Option<i64>,
    /// How many are on backorder.
    pub backorder: Option<i64>,
    /// How many are waiting to be picked.
    pub picklist: Option<i64>,
    /// How many are allocated.
    pub allocated: Option<i64>,
}

/// An order.
///
/// Picqer also sends `pickup_point_data` and `comment_count`. Those are not read here.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Order {
    /// The order.
    pub idorder: OrderId,
    /// The number of the order, such as `O2026-1599`. The documentation says this is an integer,
    /// but Picqer reports text.
    pub orderid: String,
    /// Where the order is in its life.
    pub status: Reported<OrderStatus>,
    /// Whether the order was placed through the portal. The answer to creating an order leaves it
    /// out.
    pub via_portal: Option<bool>,
    /// The lines of the order.
    pub products: Vec<OrderProduct>,
    /// The customer.
    pub idcustomer: Option<CustomerId>,
    /// The template used for the order.
    pub idtemplate: Option<TemplateId>,
    /// The shipping provider profile.
    pub idshippingprovider_profile: Option<ShippingProviderProfileId>,
    /// The name to deliver to.
    pub deliveryname: Option<String>,
    /// The contact person to deliver to.
    pub deliverycontactname: Option<String>,
    /// The street and number to deliver to.
    pub deliveryaddress: Option<String>,
    /// The second address line to deliver to.
    pub deliveryaddress2: Option<String>,
    /// The postal code to deliver to.
    pub deliveryzipcode: Option<String>,
    /// The city to deliver to.
    pub deliverycity: Option<String>,
    /// The region to deliver to.
    pub deliveryregion: Option<String>,
    /// The country to deliver to.
    pub deliverycountry: Option<Reported<CountryCode>>,
    /// The whole delivery address as text.
    pub full_delivery_address: Option<String>,
    /// The name to invoice.
    pub invoicename: Option<String>,
    /// The contact person to invoice.
    pub invoicecontactname: Option<String>,
    /// The street and number to invoice.
    pub invoiceaddress: Option<String>,
    /// The second address line to invoice.
    pub invoiceaddress2: Option<String>,
    /// The postal code to invoice.
    pub invoicezipcode: Option<String>,
    /// The city to invoice.
    pub invoicecity: Option<String>,
    /// The region to invoice.
    pub invoiceregion: Option<String>,
    /// The country to invoice.
    pub invoicecountry: Option<Reported<CountryCode>>,
    /// The whole invoice address as text.
    pub full_invoice_address: Option<String>,
    /// The phone number.
    pub telephone: Option<String>,
    /// The email address.
    pub emailaddress: Option<String>,
    /// Your own reference for the order.
    pub reference: Option<String>,
    /// Remarks of the customer.
    pub customer_remarks: Option<String>,
    /// Whether the order may be delivered in parts.
    pub partialdelivery: Option<bool>,
    /// Whether orders of the customer are split automatically.
    pub auto_split: Option<bool>,
    /// The discount, in percent.
    pub discount: Option<Percentage>,
    /// Whether VAT is calculated.
    pub calculatevat: Option<bool>,
    /// Whether the order was invoiced. The answer to creating an order leaves it out.
    pub invoiced: Option<bool>,
    /// Why the order is paused, when it is.
    pub paused_reason: Option<String>,
    /// The fulfilment customer, with Picqer Fulfilment.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// The preferred delivery date.
    pub preferred_delivery_date: Option<Date>,
    /// The language of the order.
    pub language: Option<Reported<Language>>,
    /// A page where the customer can follow the order.
    pub public_status_page: Option<String>,
    /// When the order was created.
    pub created: Option<DateTime>,
    /// When the order was last changed.
    pub updated: Option<DateTime>,
    /// The warehouses the order can be delivered from.
    #[serde(default)]
    pub warehouses: Vec<WarehouseId>,
    /// The tags of the order, by title.
    #[serde(default, deserialize_with = "crate::serde_util::tag_map")]
    pub tags: BTreeMap<String, Tag>,
    /// The values of the order fields that have one.
    #[serde(default)]
    pub orderfields: Vec<OrderFieldEntry>,
    /// The pricelists the order can use.
    #[serde(default)]
    pub pricelists: Vec<PricelistId>,
    /// The picklists of the order, without their lines.
    #[serde(default)]
    pub picklists: Vec<Picklist>,
}
