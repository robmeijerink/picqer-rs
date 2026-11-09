use serde::{Deserialize, Serialize};

use crate::types::{ProductId, WarehouseId};

/// The settings of a product in one warehouse.
///
/// A product has none for a warehouse until they are set. The documentation only shows examples,
/// so what is not an identifier is optional here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct WarehouseSettings {
    /// The product.
    pub idproduct: ProductId,
    /// The warehouse.
    pub idwarehouse: WarehouseId,
    /// The stock level at which the product is ordered.
    pub stock_level_order: Option<i64>,
    /// The stock level to order up to.
    pub stock_level_desired: Option<i64>,
    /// The highest stock level.
    pub stock_level_maximum: Option<i64>,
    /// The stock level of the picking location at which it is replenished.
    pub picking_stock_replenish_trigger: Option<i64>,
    /// The stock level the picking location is replenished to.
    pub picking_stock_replenish_to: Option<i64>,
    /// The location of the product. Deprecated in favor of locations.
    pub stock_location: Option<String>,
}

/// Changes to the settings of a product in a warehouse. Only what is set is sent.
///
/// Picqer applies rules of its own, which the documentation does not describe. On the account this
/// was checked against, `stock_level_desired` was only kept when `stock_level_order` was above zero,
/// `stock_level_maximum` was not stored at all, and a replenish trigger without a target set the
/// target to the same value. The `stock_location` field is deprecated and creates locations as a side
/// effect, so it is not offered here.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct WarehouseSettingsChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    stock_level_order: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stock_level_desired: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stock_level_maximum: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    picking_stock_replenish_trigger: Option<Option<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    picking_stock_replenish_to: Option<Option<u32>>,
}

impl WarehouseSettingsChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the stock level at which the product is ordered.
    pub fn stock_level_order(mut self, level: u32) -> Self {
        self.stock_level_order = Some(level);
        self
    }

    /// Sets the stock level to order up to.
    pub fn stock_level_desired(mut self, level: u32) -> Self {
        self.stock_level_desired = Some(level);
        self
    }

    /// Sets the highest stock level.
    pub fn stock_level_maximum(mut self, level: u32) -> Self {
        self.stock_level_maximum = Some(level);
        self
    }

    /// Sets the stock level of the picking location at which it is replenished, or clears it with
    /// `None`.
    pub fn picking_stock_replenish_trigger(mut self, level: Option<u32>) -> Self {
        self.picking_stock_replenish_trigger = Some(level);
        self
    }

    /// Sets the stock level the picking location is replenished to, or clears it with `None`.
    pub fn picking_stock_replenish_to(mut self, level: Option<u32>) -> Self {
        self.picking_stock_replenish_to = Some(level);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_what_is_set_is_sent_and_none_clears() {
        assert_eq!(
            serde_json::to_string(&WarehouseSettingsChanges::new()).unwrap(),
            "{}"
        );
        let changes = WarehouseSettingsChanges::new()
            .stock_level_order(5)
            .picking_stock_replenish_trigger(None)
            .picking_stock_replenish_to(Some(25));
        assert_eq!(
            serde_json::to_value(changes).unwrap(),
            serde_json::json!({
                "stock_level_order": 5,
                "picking_stock_replenish_trigger": null,
                "picking_stock_replenish_to": 25
            })
        );
    }
}
