use std::collections::BTreeMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Deserializer};

use crate::country::CountryCode;
use crate::reported::Reported;
use crate::tags::Tag;
use crate::types::{
    CustomerId, Date, DateTime, FulfilmentCustomerId, LocationId, Money, OrderId, OrderProductId,
    PackagingId, PicklistBatchId, PicklistId, PicklistProductId, PicklistProductLocationId,
    ProductId, ShippingProviderProfileId, TemplateId, UserId, VatGroupId, WarehouseId,
};

string_enum! {
    /// Where a picklist is in its life.
    pub enum PicklistStatus {
        /// Ready to be picked.
        New = "new",
        /// Put on hold.
        Paused = "paused",
        /// Picked and finished.
        Closed = "closed",
        /// Hidden until a moment in the future.
        Snoozed = "snoozed",
        /// Cancelled.
        Cancelled = "cancelled",
    }
}

/// How much of a picklist line is picked from one location.
///
/// The pick locations of a line are worked out when the picklist is made and can change when stock
/// is moved. Their amounts do not have to add up to the amount of the line, because some stock has
/// no specific location.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PickLocation {
    /// This part of the line. Picqer is enabling this field gradually.
    pub idpicklist_product_location: Option<PicklistProductLocationId>,
    /// The location, or nothing for stock that has no specific location.
    pub idlocation: Option<LocationId>,
    /// The name of the location. Stock without a specific location has none, although the
    /// documentation says a name is required.
    pub name: Option<String>,
    /// How many are to be picked there.
    pub amount: i64,
    /// How many are picked there. Picqer is enabling this field gradually.
    pub amount_picked: Option<i64>,
    /// The lot number, when traceability is enabled.
    pub lot_number: Option<String>,
    /// The expiration date of the lot, when traceability is enabled.
    pub expiration_date: Option<Date>,
}

/// A line of a picklist.
///
/// Picqer also sends legacy duplicates of some fields, `stock_location` and `has_parts`. Those are
/// not read here.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct PicklistProduct {
    /// The line. The documentation does not list it as an attribute, but its endpoints use it.
    pub idpicklist_product: PicklistProductId,
    /// The product.
    pub idproduct: ProductId,
    /// The line of the order this line comes from.
    pub idorder_product: Option<OrderProductId>,
    /// The replacement of a return this line comes from.
    pub idreturn_product_replacement: Option<i64>,
    /// The VAT group.
    pub idvatgroup: VatGroupId,
    /// The code of the product.
    pub productcode: String,
    /// The name of the product.
    pub name: String,
    /// Remarks about the line.
    pub remarks: Option<String>,
    /// How many are to be picked.
    pub amount: i64,
    /// How many are picked.
    pub amount_picked: i64,
    /// The price per piece, excluding VAT.
    pub price: Money,
    /// The weight in grams.
    pub weight: Option<i64>,
    /// Where the line is picked.
    pub pick_locations: Vec<PickLocation>,
}

/// A picklist: what has to be collected in the warehouse for an order or a return.
///
/// The picklists that Picqer embeds in an order have no lines, so `products` is empty there.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Picklist {
    /// The picklist.
    pub idpicklist: PicklistId,
    /// The number of the picklist, such as `P2026-1370`.
    pub picklistid: String,
    /// The warehouse.
    pub idwarehouse: WarehouseId,
    /// The template.
    pub idtemplate: TemplateId,
    /// The name to deliver to.
    pub deliveryname: String,
    /// The country to deliver to.
    pub deliverycountry: Reported<CountryCode>,
    /// Whether the picklist was invoiced.
    pub invoiced: bool,
    /// Whether the picklist is urgent.
    pub urgent: bool,
    /// Where the picklist is in its life.
    pub status: Reported<PicklistStatus>,
    /// How many products are to be picked.
    pub totalproducts: i64,
    /// How many products are picked.
    pub totalpicked: i64,
    /// The weight in grams.
    pub weight: i64,
    /// When the picklist was created.
    pub created: DateTime,
    /// When the picklist was last changed.
    pub updated: DateTime,
    /// The lines of the picklist.
    #[serde(default)]
    pub products: Vec<PicklistProduct>,
    /// The customer.
    pub idcustomer: Option<CustomerId>,
    /// The order.
    pub idorder: Option<OrderId>,
    /// The return, for a picklist of a return.
    pub idreturn: Option<i64>,
    /// The batch the picklist is part of.
    pub idpicklist_batch: Option<PicklistBatchId>,
    /// The shipping provider profile.
    pub idshippingprovider_profile: Option<ShippingProviderProfileId>,
    /// The contact person to deliver to.
    pub deliverycontact: Option<String>,
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
    /// The phone number.
    pub telephone: Option<String>,
    /// The email address.
    pub emailaddress: Option<String>,
    /// Your own reference.
    pub reference: Option<String>,
    /// The user the picklist is assigned to.
    pub assigned_to_iduser: Option<UserId>,
    /// The preferred delivery date.
    pub preferred_delivery_date: Option<Date>,
    /// Why the picklist is paused, when it is.
    pub paused_reason: Option<String>,
    /// Until when the picklist is snoozed, when it is.
    pub snoozed_until: Option<DateTime>,
    /// The user that closed the picklist.
    pub closed_by_iduser: Option<UserId>,
    /// When the picklist was closed.
    pub closed_at: Option<DateTime>,
    /// The fulfilment customer, with Picqer Fulfilment.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// The tags of the picklist, by title.
    #[serde(default, deserialize_with = "crate::serde_util::tag_map")]
    pub tags: BTreeMap<String, Tag>,
}

string_enum! {
    /// How the packaging advice of a picklist came about.
    pub enum AdviceSource {
        /// Picqer worked it out.
        Automatic = "automatic",
        /// Set through the API.
        Api = "api",
    }
}

/// The packaging Picqer advises for a picklist.
///
/// A picklist without advice has all fields empty.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PackagingAdvice {
    /// The packaging.
    pub idpackaging: Option<PackagingId>,
    /// How the advice came about.
    pub source: Option<Reported<AdviceSource>>,
    /// When the advice was made.
    pub created_at: Option<DateTime>,
    /// When the advice was last changed.
    pub updated_at: Option<DateTime>,
}

/// Where a line is picked, after a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PickedAt {
    /// The location.
    pub idlocation: LocationId,
    /// How many are to be picked there.
    pub amount: i64,
}

#[derive(Deserialize)]
pub(crate) struct PdfBody {
    #[serde(deserialize_with = "base64_bytes")]
    pub pdf: Vec<u8>,
}

fn base64_bytes<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    let text = String::deserialize(deserializer)?;
    STANDARD
        .decode(text.trim())
        .map_err(serde::de::Error::custom)
}
