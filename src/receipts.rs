//! Receipts, to register received purchases.
//!
//! This follows version 2 of the receipts API, in which a product that is received goes into the
//! stock at once and can be reverted. Picqer deprecates version 1 and makes one when the version is
//! left out of a request, so [`NewReceipt`] always asks for version 2. A list can still hold
//! receipts of version 1, which are read with what they have in common.
//!
//! The documentation describes the fields and its examples show a few more, which are read here
//! too. What a developer account also reports but the documentation never shows, such as the
//! fulfilment customer and the price of a received product, is not read.

use std::num::NonZeroU32;

use reqwest::Method;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::locations::Location;
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::products::{AbcClassification, LotNumber, Product};
use crate::reported::Reported;
use crate::types::{
    Date, DateTime, LocationId, ProductId, PurchaseOrderId, PurchaseOrderProductId, ReceiptId,
    ReceiptProductId, SupplierId, UserId, WarehouseId,
};

string_enum! {
    /// Where a receipt is in its life.
    pub enum ReceiptStatus {
        /// Products are being received.
        Processing = "processing",
        /// The receipt is done. Products can no longer be added, but they can still be reverted.
        Completed = "completed",
    }
}

string_enum! {
    /// How the amount of a product that is received is divided over purchase orders.
    pub enum StrategyKind {
        /// Divides the amount over the open lines of purchase orders, for a receipt that is not
        /// for one purchase order.
        Automatic = "automatic",
        /// Registers the amount on one line of a purchase order.
        PurchaseorderProduct = "purchaseorder_product",
        /// Adds the product to the purchase order of the receipt, or to a new purchase order for
        /// what was not ordered.
        New = "new",
    }
}

/// The supplier of a receipt.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReceiptSupplier {
    /// The supplier.
    pub idsupplier: SupplierId,
    /// The name of the supplier.
    pub name: Option<String>,
}

/// The purchase order of a receipt.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReceiptPurchaseOrder {
    /// The purchase order.
    pub idpurchaseorder: PurchaseOrderId,
    /// The number of the purchase order.
    pub purchaseorderid: Option<String>,
}

/// The user that completed a receipt.
///
/// The documentation calls this an array, but Picqer sends an object, as in its example.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReceiptUser {
    /// The user.
    pub iduser: UserId,
    /// The name of the user.
    pub name: Option<String>,
}

/// A product that was received.
///
/// Receiving puts it into the stock at once. Reverting it takes the amount out of the stock again
/// and marks it as reverted.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReceiptProduct {
    /// The line of the receipt.
    pub idreceipt_product: ReceiptProductId,
    /// The line of the purchase order this was received on.
    pub idpurchaseorder_product: Option<PurchaseOrderProductId>,
    /// The product.
    pub idproduct: ProductId,
    /// The purchase order.
    pub idpurchaseorder: Option<PurchaseOrderId>,
    /// The number of the purchase order.
    pub purchaseorderid: Option<String>,
    /// The product code.
    pub productcode: Option<String>,
    /// The product code of the supplier.
    pub productcode_supplier: Option<String>,
    /// The name of the product.
    pub name: Option<String>,
    /// The barcode.
    pub barcode: Option<String>,
    /// The ABC classification of the product.
    pub abc_classification: Option<Reported<AbcClassification>>,
    /// Where the image of the product can be downloaded.
    pub image: Option<String>,
    /// How many were received.
    pub amount: Option<i64>,
    /// How many were ordered.
    pub amount_ordered: Option<i64>,
    /// Whether the product was not on the purchase order and was added by the receipt.
    pub added_by_receipt: Option<bool>,
    /// The location the stock went to. Not in a list of receipts, and `None` for stock that is not
    /// kept at a specific location. The documentation calls this an array, but it is one location.
    pub location: Option<Location>,
    /// When the product was received.
    pub created_at: Option<DateTime>,
    /// The user that received the product.
    pub received_by_iduser: Option<UserId>,
    /// The lot number that was recorded.
    pub lot_number: Option<String>,
    /// The expiration date that was recorded.
    pub expiration_date: Option<Date>,
    /// When the product was reverted.
    pub reverted_at: Option<DateTime>,
    /// The user that reverted the product.
    pub reverted_by_iduser: Option<UserId>,
}

/// A receipt.
///
/// Whatever the documentation does not mark as always present is optional.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Receipt {
    /// The receipt.
    pub idreceipt: ReceiptId,
    /// The warehouse.
    pub idwarehouse: Option<WarehouseId>,
    /// The version of the receipt: 1 or 2.
    pub version: Option<i64>,
    /// The supplier.
    pub supplier: Option<ReceiptSupplier>,
    /// The purchase order, for a receipt of one purchase order.
    pub purchaseorder: Option<ReceiptPurchaseOrder>,
    /// The number of the receipt within the account, such as `RC2026-1000`.
    pub receiptid: Option<String>,
    /// Where the receipt is in its life.
    pub status: Option<Reported<ReceiptStatus>>,
    /// The user that completed the receipt.
    pub completed_by: Option<ReceiptUser>,
    /// How many products were received.
    pub amount_received: Option<i64>,
    /// When the receipt was completed.
    pub completed_at: Option<DateTime>,
    /// When the receipt was made.
    pub created: Option<DateTime>,
    /// When the receipt was last changed.
    pub updated: Option<DateTime>,
    /// The products that were received.
    #[serde(default)]
    pub products: Vec<ReceiptProduct>,
}

/// How many of a product are expected, received and left.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ExpectedTotals {
    /// How many are expected.
    pub expected: Option<i64>,
    /// How many are received.
    pub received: Option<i64>,
    /// How many are left.
    pub remaining: Option<i64>,
}

/// The parameters of a strategy.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct StrategyParameters {
    /// The line of the purchase order, for the strategy that receives on one line.
    pub idpurchaseorder_product: Option<PurchaseOrderProductId>,
}

/// A way to receive a product, with how much it would cover.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ReceivingStrategy {
    /// The strategy.
    pub strategy: Reported<StrategyKind>,
    /// What to give to receive with it.
    pub parameters: Option<StrategyParameters>,
    /// The number of the purchase order.
    pub purchaseorderid: Option<String>,
    /// The purchase order, for the strategy that adds to one.
    pub idpurchaseorder: Option<PurchaseOrderId>,
    /// How many were ordered. Not for the strategy that adds a product.
    pub ordered: Option<i64>,
    /// How many were received.
    pub received: Option<i64>,
    /// How many are left.
    pub remaining: Option<i64>,
}

/// A product that is expected on a receipt, with the ways to receive it.
///
/// With traceability on the account, the product says whether a lot number and an expiration date
/// must be given when it is received.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ExpectedProduct {
    /// The product.
    pub product: Product,
    /// How many are expected, received and left.
    pub totals: Option<ExpectedTotals>,
    /// The ways to receive it.
    #[serde(default)]
    pub strategies: Vec<ReceivingStrategy>,
}

/// Filters for listing receipts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReceiptQuery {
    receiptid: Option<String>,
    status: Option<ReceiptStatus>,
    idpurchaseorder: Option<PurchaseOrderId>,
    idsupplier: Option<SupplierId>,
    updated_after: Option<DateTime>,
    completed_after: Option<DateTime>,
    include_indirect_receipts: bool,
}

impl ReceiptQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the receipt with this number, such as `RC2026-1000`.
    pub fn receiptid(mut self, receiptid: impl Into<String>) -> Self {
        self.receiptid = Some(receiptid.into());
        self
    }

    /// Lists the receipts with this status.
    pub fn status(mut self, status: ReceiptStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Lists the receipts for a purchase order.
    pub fn idpurchaseorder(mut self, idpurchaseorder: PurchaseOrderId) -> Self {
        self.idpurchaseorder = Some(idpurchaseorder);
        self
    }

    /// Also lists the receipts that are for several purchase orders and received products of this
    /// one. Only with [`idpurchaseorder`](Self::idpurchaseorder).
    pub fn include_indirect_receipts(mut self) -> Self {
        self.include_indirect_receipts = true;
        self
    }

    /// Lists the receipts for a supplier.
    pub fn idsupplier(mut self, idsupplier: SupplierId) -> Self {
        self.idsupplier = Some(idsupplier);
        self
    }

    /// Lists the receipts that changed after this moment.
    pub fn updated_after(mut self, moment: DateTime) -> Self {
        self.updated_after = Some(moment);
        self
    }

    /// Lists the receipts that were completed after this moment.
    pub fn completed_after(mut self, moment: DateTime) -> Self {
        self.completed_after = Some(moment);
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(receiptid) = &self.receiptid {
            pairs.push(("receiptid", receiptid.clone()));
        }
        if let Some(status) = self.status {
            pairs.push(("status", status.as_str().to_owned()));
        }
        if let Some(id) = self.idpurchaseorder {
            pairs.push(("idpurchaseorder", id.to_string()));
        }
        if let Some(id) = self.idsupplier {
            pairs.push(("idsupplier", id.to_string()));
        }
        if let Some(moment) = self.updated_after {
            pairs.push(("updated_after", moment.to_string()));
        }
        if let Some(moment) = self.completed_after {
            pairs.push(("completed_after", moment.to_string()));
        }
        if self.include_indirect_receipts {
            pairs.push(("include_indirect_receipts", "true".to_owned()));
        }
        pairs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReceiptFor {
    PurchaseOrder(PurchaseOrderId),
    Supplier(SupplierId),
}

/// A receipt to create, for a purchase order or for a supplier.
///
/// A receipt for a supplier registers products of several purchase orders. Picqer also lets a
/// receipt be made for a purchase order that is still a concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewReceipt {
    receipt_for: ReceiptFor,
    idwarehouse: Option<WarehouseId>,
}

impl NewReceipt {
    /// A receipt for a purchase order.
    pub fn for_purchase_order(purchase_order: PurchaseOrderId) -> Self {
        Self {
            receipt_for: ReceiptFor::PurchaseOrder(purchase_order),
            idwarehouse: None,
        }
    }

    /// A receipt for a supplier.
    pub fn for_supplier(supplier: SupplierId) -> Self {
        Self {
            receipt_for: ReceiptFor::Supplier(supplier),
            idwarehouse: None,
        }
    }

    /// Sets the warehouse. Picqer takes the first warehouse when none is given.
    pub fn idwarehouse(mut self, idwarehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(idwarehouse);
        self
    }

    pub(crate) fn to_value(self) -> Value {
        let mut body = Map::new();
        match self.receipt_for {
            ReceiptFor::PurchaseOrder(id) => body.insert("idpurchaseorder".into(), json!(id)),
            ReceiptFor::Supplier(id) => body.insert("idsupplier".into(), json!(id)),
        };
        if let Some(warehouse) = self.idwarehouse {
            body.insert("idwarehouse".into(), json!(warehouse));
        }
        body.insert("version".into(), json!(2));
        Value::Object(body)
    }
}

/// How to divide the amount that is received over purchase orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveStrategy {
    /// Registers the amount on one line of a purchase order.
    PurchaseOrderProduct(PurchaseOrderProductId),
    /// Divides the amount over the open lines of purchase orders.
    Automatic,
    /// Adds the product to the purchase order of the receipt, or to a new one for what was not
    /// ordered.
    New,
}

/// A product to receive, which goes into the stock at once.
///
/// Picqer accepts an amount that is more than what is expected. With traceability on the account, a
/// product that tracks lot numbers and expiration dates needs both and a location, and a location
/// can only hold one lot: another one is refused with code 44. Picqer refuses a lot number or an
/// expiration date that is not valid, or a missing one, with code 46.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveProduct {
    idproduct: ProductId,
    amount: NonZeroU32,
    strategy: ReceiveStrategy,
    idlocation: Option<LocationId>,
    lot_number: Option<LotNumber>,
    expiration_date: Option<Date>,
}

impl ReceiveProduct {
    /// Receives an amount of a product with a strategy.
    pub fn new(idproduct: ProductId, amount: NonZeroU32, strategy: ReceiveStrategy) -> Self {
        Self {
            idproduct,
            amount,
            strategy,
            idlocation: None,
            lot_number: None,
            expiration_date: None,
        }
    }

    /// Puts the stock on a location or container in the warehouse of the receipt. Without one it
    /// is not kept at a specific location.
    pub fn idlocation(mut self, idlocation: LocationId) -> Self {
        self.idlocation = Some(idlocation);
        self
    }

    /// Records the lot number.
    pub fn lot_number(mut self, lot_number: LotNumber) -> Self {
        self.lot_number = Some(lot_number);
        self
    }

    /// Records the expiration date.
    pub fn expiration_date(mut self, date: Date) -> Self {
        self.expiration_date = Some(date);
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut body = Map::new();
        body.insert("idproduct".into(), json!(self.idproduct));
        body.insert("amount".into(), json!(self.amount));
        match self.strategy {
            ReceiveStrategy::PurchaseOrderProduct(line) => {
                body.insert("idpurchaseorder_product".into(), json!(line));
            }
            ReceiveStrategy::Automatic => {
                body.insert("strategy".into(), json!("automatic"));
            }
            ReceiveStrategy::New => {
                body.insert("strategy".into(), json!("new"));
            }
        }
        if let Some(location) = self.idlocation {
            body.insert("idlocation".into(), json!(location));
        }
        if let Some(lot) = &self.lot_number {
            body.insert("lot_number".into(), json!(lot));
        }
        if let Some(date) = self.expiration_date {
            body.insert("expiration_date".into(), json!(date));
        }
        Value::Object(body)
    }
}

pub(crate) fn list(query: &ReceiptQuery, page: Page) -> Call<Vec<Receipt>> {
    let mut request = Request::new(Method::GET, segments(&[&"receipts"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: ReceiptId) -> Call<Receipt> {
    Call::new(Request::new(Method::GET, segments(&[&"receipts", &id])))
}

pub(crate) fn create(receipt: NewReceipt) -> Result<Call<Receipt>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"receipts"])).json_body(&receipt.to_value())?,
    ))
}

pub(crate) fn complete(id: ReceiptId) -> Result<Call<Receipt>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"receipts", &id]))
            .json_body(&json!({"status": "completed"}))?,
    ))
}

pub(crate) fn delete(id: ReceiptId) -> NoContent {
    NoContent::new(Request::new(Method::DELETE, segments(&[&"receipts", &id])))
}

pub(crate) fn expected_products(id: ReceiptId) -> Call<Vec<ExpectedProduct>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"receipts", &id, &"expected-products"]),
    ))
}

pub(crate) fn expected_product(id: ReceiptId, product: ProductId) -> Call<ExpectedProduct> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"receipts", &id, &"expected-products", &product]),
    ))
}

pub(crate) fn receive(
    id: ReceiptId,
    product: &ReceiveProduct,
) -> Result<Call<Vec<ReceiptProduct>>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"receipts", &id, &"products"]))
            .json_body(&product.to_value())?,
    ))
}

pub(crate) fn revert(id: ReceiptId, line: ReceiptProductId) -> Call<ReceiptProduct> {
    Call::new(Request::new(
        Method::POST,
        segments(&[&"receipts", &id, &"products", &line, &"revert"]),
    ))
}

macro_rules! receipts_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of receipts, newest first.
        pub $($async)? fn list(
            &self,
            query: &$crate::receipts::ReceiptQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::receipts::Receipt>> {
            self.client.execute($crate::receipts::list(query, page)) $(.$await)?
        }

        /// Lists all receipts that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::receipts::ReceiptQuery,
        ) -> $crate::error::Result<Vec<$crate::receipts::Receipt>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::receipts::list(query, page)) $(.$await)?)
        }

        /// Fetches a receipt, with where each product went.
        pub $($async)? fn get(
            &self,
            id: $crate::types::ReceiptId,
        ) -> $crate::error::Result<$crate::receipts::Receipt> {
            self.client.execute($crate::receipts::get(id)) $(.$await)?
        }

        /// Creates a receipt, of version 2.
        pub $($async)? fn create(
            &self,
            receipt: $crate::receipts::NewReceipt,
        ) -> $crate::error::Result<$crate::receipts::Receipt> {
            self.client.execute($crate::receipts::create(receipt)?) $(.$await)?
        }

        /// Completes a receipt, after which products can no longer be added (code 33) but can
        /// still be reverted. A receipt of version 2 can be completed without products.
        pub $($async)? fn complete(
            &self,
            id: $crate::types::ReceiptId,
        ) -> $crate::error::Result<$crate::receipts::Receipt> {
            self.client.execute($crate::receipts::complete(id)?) $(.$await)?
        }

        /// Deletes a receipt that has no products, or whose products were all reverted. Picqer
        /// refuses a completed receipt that had products received.
        pub $($async)? fn delete(&self, id: $crate::types::ReceiptId) -> $crate::error::Result<()> {
            self.client.execute($crate::receipts::delete(id)) $(.$await)?
        }

        /// Lists the products that are expected on a receipt, with the ways to receive them.
        pub $($async)? fn expected_products(
            &self,
            id: $crate::types::ReceiptId,
        ) -> $crate::error::Result<Vec<$crate::receipts::ExpectedProduct>> {
            self.client.execute($crate::receipts::expected_products(id)) $(.$await)?
        }

        /// Fetches the ways to receive one product on a receipt.
        pub $($async)? fn expected_product(
            &self,
            id: $crate::types::ReceiptId,
            product: $crate::types::ProductId,
        ) -> $crate::error::Result<$crate::receipts::ExpectedProduct> {
            self.client.execute($crate::receipts::expected_product(id, product)) $(.$await)?
        }

        /// Receives a product, which puts it into the stock. The answer holds a line for each
        /// line of a purchase order the amount was divided over.
        pub $($async)? fn receive(
            &self,
            id: $crate::types::ReceiptId,
            product: &$crate::receipts::ReceiveProduct,
        ) -> $crate::error::Result<Vec<$crate::receipts::ReceiptProduct>> {
            self.client.execute($crate::receipts::receive(id, product)?) $(.$await)?
        }

        /// Reverts a received product, which takes its amount out of the stock again. This works
        /// when the stock is still there, also after the receipt was completed.
        pub $($async)? fn revert(
            &self,
            id: $crate::types::ReceiptId,
            line: $crate::types::ReceiptProductId,
        ) -> $crate::error::Result<$crate::receipts::ReceiptProduct> {
            self.client.execute($crate::receipts::revert(id, line)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use receipts_methods;

/// Operations on receipts.
#[derive(Debug, Clone, Copy)]
pub struct Receipts<'a> {
    pub(crate) client: &'a Client,
}

impl Receipts<'_> {
    receipts_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_receipt_always_asks_for_version_2() {
        let receipt = NewReceipt::for_purchase_order(PurchaseOrderId(7));
        assert_eq!(
            receipt.to_value(),
            json!({"idpurchaseorder": 7, "version": 2})
        );
        let receipt = NewReceipt::for_supplier(SupplierId(3)).idwarehouse(WarehouseId(2));
        assert_eq!(
            receipt.to_value(),
            json!({"idsupplier": 3, "idwarehouse": 2, "version": 2})
        );
    }

    #[test]
    fn a_product_is_received_with_the_strategy_the_documentation_shows() {
        let amount = NonZeroU32::new(2).unwrap();
        let by_line = ReceiveProduct::new(
            ProductId(78),
            amount,
            ReceiveStrategy::PurchaseOrderProduct(PurchaseOrderProductId(493)),
        )
        .idlocation(LocationId(664))
        .lot_number(LotNumber::new("LOT-2026-001").unwrap())
        .expiration_date("2027-01-15".parse().unwrap());
        assert_eq!(
            by_line.to_value(),
            json!({
                "idproduct": 78, "amount": 2, "idpurchaseorder_product": 493,
                "idlocation": 664, "lot_number": "LOT-2026-001", "expiration_date": "2027-01-15"
            })
        );
        assert_eq!(
            ReceiveProduct::new(ProductId(78), amount, ReceiveStrategy::Automatic).to_value(),
            json!({"idproduct": 78, "amount": 2, "strategy": "automatic"})
        );
        assert_eq!(
            ReceiveProduct::new(ProductId(78), amount, ReceiveStrategy::New).to_value(),
            json!({"idproduct": 78, "amount": 2, "strategy": "new"})
        );
    }

    #[test]
    fn filters_become_query_pairs() {
        assert!(ReceiptQuery::new().pairs().is_empty());
        let moment: DateTime = "2026-11-09 16:00:00".parse().unwrap();
        let query = ReceiptQuery::new()
            .receiptid("RC2026-1000")
            .status(ReceiptStatus::Completed)
            .idpurchaseorder(PurchaseOrderId(5))
            .include_indirect_receipts()
            .idsupplier(SupplierId(6))
            .updated_after(moment)
            .completed_after(moment);
        assert_eq!(
            query.pairs(),
            vec![
                ("receiptid", "RC2026-1000".to_owned()),
                ("status", "completed".to_owned()),
                ("idpurchaseorder", "5".to_owned()),
                ("idsupplier", "6".to_owned()),
                ("updated_after", "2026-11-09 16:00:00".to_owned()),
                ("completed_after", "2026-11-09 16:00:00".to_owned()),
                ("include_indirect_receipts", "true".to_owned()),
            ]
        );
    }
}
