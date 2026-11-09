use serde::Deserialize;

use crate::types::{
    Date, DateTime, Money, PurchaseOrderId, PurchaseOrderProductId, UserId, WarehouseId,
};

/// A change of the price of a product.
///
/// The documentation only shows examples, so what is not an amount is optional here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PriceChange {
    /// The price before the change.
    pub oldprice: Money,
    /// The change.
    pub pricechange: Money,
    /// The price after the change.
    pub newprice: Money,
    /// The user that made the change.
    pub iduser: Option<UserId>,
    /// When the price changed.
    pub date: Option<DateTime>,
}

/// A change of the fixed stock price of a product.
///
/// The documentation only shows examples, so what is not an amount is optional here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct FixedStockPriceChange {
    /// The fixed stock price before the change.
    pub old_fixedstockprice: Money,
    /// The change.
    pub fixedstockprice_change: Money,
    /// The fixed stock price after the change.
    pub new_fixedstockprice: Money,
    /// The user that made the change.
    pub iduser: Option<UserId>,
    /// When the fixed stock price changed.
    pub date: Option<DateTime>,
}

/// A line of an outstanding purchase order that is still to be received.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ExpectedDelivery {
    /// The purchase order.
    pub idpurchaseorder: PurchaseOrderId,
    /// The line of the purchase order.
    pub idpurchaseorder_product: PurchaseOrderProductId,
    /// The warehouse the product is expected in.
    pub idwarehouse: WarehouseId,
    /// The amount that is still to be received.
    pub amount_to_receive: i64,
    /// When the delivery is expected, if the purchase order says.
    pub delivery_date: Option<Date>,
}
