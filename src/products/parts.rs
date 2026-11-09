use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::products::response::ProductType;
use crate::reported::Reported;
use crate::stock::StockReason;
use crate::types::{ProductId, ProductPartId, WarehouseId};

/// A product in a composition, or a composition a product is part of.
///
/// For the parts of a composition, `idproduct` is the part and `amount` how many of it the
/// composition holds. For the compositions of a product, `idproduct` is the composition and
/// `amount` how many of the product it holds.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ProductPart {
    /// The link between the composition and the part.
    pub idproduct_part: ProductPartId,
    /// The product on the other side of the link.
    pub idproduct: ProductId,
    /// How many of the part the composition holds.
    pub amount: i64,
    /// The code of the product.
    pub productcode: String,
    /// The name of the product.
    pub name: String,
    /// The type of the product.
    #[serde(rename = "type")]
    pub product_type: Reported<ProductType>,
}

/// A part with the parts below it, when it is a virtual composition itself.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PartTree {
    /// The part.
    #[serde(flatten)]
    pub part: ProductPart,
    /// The parts of the part. Only a virtual composition has any.
    #[serde(default)]
    pub parts: Vec<PartTree>,
}

/// A composition with the compositions above it, when it is a virtual composition itself.
///
/// Compositions that have stock of their own are not followed further.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CompositionTree {
    /// The composition.
    #[serde(flatten)]
    pub part: ProductPart,
    /// The compositions the composition is part of. Only a virtual composition has any.
    #[serde(default)]
    pub compositions: Vec<CompositionTree>,
}

/// A product to add to a composition.
///
/// A product can be added once, and not when that would make a composition part of itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NewProductPart {
    idproduct: ProductId,
    amount: NonZeroU32,
}

impl NewProductPart {
    /// Adds `amount` of a product to the composition.
    pub fn new(idproduct: ProductId, amount: NonZeroU32) -> Self {
        Self { idproduct, amount }
    }
}

/// Production of a composition that has stock of its own.
///
/// The parts are taken out of stock and the composition is added to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Production {
    idwarehouse: WarehouseId,
    amount: NonZeroU32,
    reason: StockReason,
}

impl Production {
    /// Produces `amount` of a composition in a warehouse.
    pub fn new(idwarehouse: WarehouseId, amount: NonZeroU32, reason: StockReason) -> Self {
        Self {
            idwarehouse,
            amount,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_part_is_sent_with_its_amount() {
        let part = NewProductPart::new(ProductId(5), NonZeroU32::new(2).unwrap());
        assert_eq!(
            serde_json::to_value(part).unwrap(),
            serde_json::json!({"idproduct": 5, "amount": 2})
        );
    }

    #[test]
    fn production_names_the_warehouse_amount_and_reason() {
        let production = Production::new(
            WarehouseId(67),
            NonZeroU32::new(3).unwrap(),
            StockReason::new("New production at table 2").unwrap(),
        );
        assert_eq!(
            serde_json::to_value(production).unwrap(),
            serde_json::json!({
                "idwarehouse": 67, "amount": 3, "reason": "New production at table 2"
            })
        );
    }
}
