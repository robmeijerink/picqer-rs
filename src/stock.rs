//! Stock of a product in a warehouse.

use std::num::{NonZeroI64, NonZeroU32};

use serde::{Deserialize, Serialize, Serializer};

use crate::locations::Location;
use crate::types::{Date, LocationId, PicklistId, WarehouseId};

bounded_string! {
    /// Why stock is changed, shown in the stock history.
    StockReason, "stock reason", 1, usize::MAX
}

/// The stock of a product in one warehouse.
///
/// The numbers are what Picqer reports. `freestock` is what is left after the reservations, so it
/// is negative when more is reserved than is in stock. Picqer documents that stock cannot be
/// negative, but it accepts changes that make it so, so `stock` can be negative too.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct StockLevel {
    /// The warehouse.
    pub idwarehouse: WarehouseId,
    /// The amount in stock.
    pub stock: i64,
    /// The amount reserved in total.
    pub reserved: i64,
    /// The amount reserved for backorders.
    pub reservedbackorders: i64,
    /// The amount reserved for picklists.
    pub reservedpicklists: i64,
    /// The amount reserved for allocations.
    pub reservedallocations: i64,
    /// The amount that is not reserved.
    pub freestock: i64,
}

/// The stock of a product at one location in a warehouse.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LocationStock {
    /// The location.
    pub idlocation: LocationId,
    /// The warehouse of the location.
    pub idwarehouse: WarehouseId,
    /// The name of the location.
    pub name: String,
    /// The amount in stock at the location.
    pub stock: i64,
    /// The amount reserved for picklists at the location.
    pub reserved_picklists: i64,
    /// The amount at the location that is not reserved.
    pub free_stock: i64,
    /// Whether the location is a bulk location.
    pub is_bulk_location: bool,
    /// The lot number registered at the location. Only reported when traceability is enabled.
    pub lot_number: Option<String>,
    /// The expiration date of the lot. Only reported when traceability is enabled.
    pub expiration_date: Option<Date>,
}

/// The stock of a product in one warehouse, with the stock per location.
///
/// The stock of a warehouse is the total over its locations and the stock on no specific location.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct WarehouseStockDetail {
    /// The stock of the warehouse in total.
    #[serde(flatten)]
    pub level: StockLevel,
    /// The stock per location the product is linked to.
    pub locations: Vec<LocationStock>,
}

/// Where stock is added or removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StockTarget {
    /// A location in the warehouse.
    Location(LocationId),
    /// Stock that is not kept at a specific location.
    NoSpecificLocation,
}

impl Serialize for StockTarget {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Location(id) => id.serialize(serializer),
            Self::NoSpecificLocation => serializer.serialize_none(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
enum Update {
    Adjust {
        idlocation: StockTarget,
        change: i64,
        reason: StockReason,
    },
    Set {
        amount: u32,
        reason: StockReason,
    },
}

/// A change of the stock of a product in a warehouse.
///
/// Picqer only changes the stock of active products of the type `normal` or
/// `composition_with_stock`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct StockUpdate(Update);

impl StockUpdate {
    /// Adds stock when `change` is positive and removes it when it is negative.
    ///
    /// The target is always given, as Picqer advises: without one Picqer tries to find a matching
    /// location.
    pub fn adjust(target: StockTarget, change: NonZeroI64, reason: StockReason) -> Self {
        Self(Update::Adjust {
            idlocation: target,
            change: change.get(),
            reason,
        })
    }

    /// Sets the stock of the warehouse to an amount, whatever it was.
    ///
    /// Picqer does not accept a location with this, so the amount is for the warehouse as a whole.
    pub fn set(amount: u32, reason: StockReason) -> Self {
        Self(Update::Set { amount, reason })
    }
}

/// One end of a movement of stock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StockPlace {
    /// A location or a container. The warehouse is the one of the location.
    Location(LocationId),
    /// Stock that is not kept at a specific location, in a warehouse.
    NoSpecificLocation(WarehouseId),
}

impl Serialize for StockPlace {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            Self::Location(id) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("idlocation", id)?;
                map.end()
            }
            Self::NoSpecificLocation(warehouse) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("idlocation", &None::<LocationId>)?;
                map.serialize_entry("idwarehouse", warehouse)?;
                map.end()
            }
        }
    }
}

/// A movement of stock of a product between locations, between a location and no specific
/// location, or between warehouses.
///
/// Picqer only moves the stock of active products of the type `normal` or `composition_with_stock`.
/// It takes an amount of zero as a movement of nothing, an amount of text as a number and a negative
/// amount as a positive one, so the amount is a positive number here. Stock that is reserved for
/// picklists moves along, and only free stock can leave its warehouse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StockMove {
    amount: NonZeroU32,
    from: StockPlace,
    to: StockPlace,
    #[serde(skip_serializing_if = "Option::is_none")]
    move_claims_of_idpicklist: Option<PicklistId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<StockReason>,
}

impl StockMove {
    /// Moves an amount from one place to another.
    pub fn new(amount: NonZeroU32, from: StockPlace, to: StockPlace) -> Self {
        Self {
            amount,
            from,
            to,
            move_claims_of_idpicklist: None,
            reason: None,
        }
    }

    /// Moves the claims of one picklist along with the stock, for the amount that is moved.
    ///
    /// This is only for a movement within a warehouse, and the picklist must have enough claims for
    /// the product at the source to cover the whole amount, or Picqer moves nothing.
    pub fn move_claims_of(mut self, picklist: PicklistId) -> Self {
        self.move_claims_of_idpicklist = Some(picklist);
        self
    }

    /// Gives the reason, which shows in the stock history. Picqer requires one for a movement
    /// between warehouses and answers code 5 without it.
    pub fn reason(mut self, reason: StockReason) -> Self {
        self.reason = Some(reason);
        self
    }
}

/// One end of a movement that was made, with the stock of its warehouse afterwards.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct StockMoveSide {
    /// The location, or `None` for no specific location.
    pub location: Option<Location>,
    /// The stock of the product in the warehouse, with the stock per location.
    pub warehouse: WarehouseStockDetail,
}

/// A movement of stock that was made.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct StockMoveResult {
    /// The amount that was moved.
    pub amount: i64,
    /// Where the stock came from.
    pub from: StockMoveSide,
    /// Where the stock went to.
    pub to: StockMoveSide,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reason() -> StockReason {
        StockReason::new("Purchased inventory").unwrap()
    }

    #[test]
    fn an_adjustment_names_its_location() {
        let update = StockUpdate::adjust(
            StockTarget::Location(LocationId(5)),
            NonZeroI64::new(6).unwrap(),
            reason(),
        );
        assert_eq!(
            serde_json::to_value(update).unwrap(),
            serde_json::json!({
                "idlocation": 5, "change": 6, "reason": "Purchased inventory"
            })
        );
    }

    #[test]
    fn no_specific_location_is_sent_as_null() {
        let update = StockUpdate::adjust(
            StockTarget::NoSpecificLocation,
            NonZeroI64::new(-2).unwrap(),
            reason(),
        );
        assert_eq!(
            serde_json::to_value(update).unwrap(),
            serde_json::json!({
                "idlocation": null, "change": -2, "reason": "Purchased inventory"
            })
        );
    }

    #[test]
    fn setting_the_stock_sends_no_location() {
        assert_eq!(
            serde_json::to_value(StockUpdate::set(0, reason())).unwrap(),
            serde_json::json!({"amount": 0, "reason": "Purchased inventory"})
        );
    }

    #[test]
    fn a_reason_is_required() {
        assert!(StockReason::new("").is_err());
    }
}
