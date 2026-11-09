use serde::Deserialize;

use crate::customers::Language;
use crate::reported::Reported;
use crate::types::{
    Date, DateTime, FulfilmentCustomerId, Money, ProductId, PurchaseOrderId,
    PurchaseOrderProductId, SupplierId, TemplateId, UserId, VatGroupId, WarehouseId,
};

string_enum! {
    /// Where a purchase order is in its life.
    pub enum PurchaseOrderStatus {
        /// Being prepared.
        Concept = "concept",
        /// Sent to the supplier.
        Purchased = "purchased",
        /// Received, or closed because the rest is not expected.
        Received = "received",
        /// Cancelled.
        Cancelled = "cancelled",
    }
}

/// A line of a purchase order.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PurchaseOrderProduct {
    /// The line.
    pub idpurchaseorder_product: PurchaseOrderProductId,
    /// The product.
    pub idproduct: ProductId,
    /// How many are ordered.
    pub amount: i64,
    /// The VAT group.
    pub idvatgroup: Option<VatGroupId>,
    /// The code of the product.
    pub productcode: Option<String>,
    /// The code of the product at the supplier.
    pub productcode_supplier: Option<String>,
    /// The name of the product.
    pub name: Option<String>,
    /// The purchase price per piece, excluding VAT.
    pub price: Option<Money>,
    /// How many are received.
    pub amountreceived: Option<i64>,
    /// When this line is expected.
    pub delivery_date: Option<Date>,
    /// The weight in grams.
    pub weight: Option<i64>,
}

/// A purchase order: products bought from a supplier.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct PurchaseOrder {
    /// The purchase order.
    pub idpurchaseorder: PurchaseOrderId,
    /// The number of the purchase order, such as `PO2026-1012`.
    pub purchaseorderid: String,
    /// Where the purchase order is in its life.
    pub status: Reported<PurchaseOrderStatus>,
    /// The supplier. Missing for a purchase order made with only the name of a supplier.
    pub idsupplier: Option<SupplierId>,
    /// The warehouse the products are delivered to.
    pub idwarehouse: Option<WarehouseId>,
    /// The template.
    pub idtemplate: Option<TemplateId>,
    /// The number the supplier gave the order.
    pub supplier_orderid: Option<String>,
    /// The name of the supplier, when it was given as text.
    pub supplier_name: Option<String>,
    /// Remarks.
    pub remarks: Option<String>,
    /// When the delivery is expected.
    pub delivery_date: Option<Date>,
    /// The language used for what is sent to the supplier.
    pub language: Option<Reported<Language>>,
    /// The user that marked the order as purchased.
    pub purchased_by_iduser: Option<UserId>,
    /// When the order was marked as purchased.
    pub purchased_at: Option<DateTime>,
    /// The user that completed the order.
    pub completed_by_iduser: Option<UserId>,
    /// When the order was completed.
    pub completed_at: Option<DateTime>,
    /// The user that created the order.
    pub created_by_iduser: Option<UserId>,
    /// When the order was created.
    pub created: Option<DateTime>,
    /// When the order was last changed.
    pub updated: Option<DateTime>,
    /// The fulfilment customer, with Picqer Fulfilment.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// The number of comments on the order.
    pub comment_count: Option<i64>,
    /// The lines of the order.
    #[serde(default)]
    pub products: Vec<PurchaseOrderProduct>,
}
