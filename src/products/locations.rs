use std::collections::BTreeMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::http::{Request, segments};
use crate::locations::Location;
use crate::ops::{Call, NoContent};
use crate::stock::{StockMove, StockMoveResult};
use crate::types::{Date, LocationId, ProductId};

bounded_string! {
    /// A lot number, which Picqer takes up to 255 characters of.
    LotNumber, "lot number", 1, 255
}

/// The stock of a product at one location.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ProductLocationStock {
    /// The stock at the location. Picqer leaves it out in the answer to linking a location.
    pub stock: Option<i64>,
    /// How much of it is reserved for picklists.
    pub stock_reserved_picklists: Option<i64>,
}

/// A location a product is linked to.
///
/// A product can be linked to none, one or more locations. The stock is that of this location: the
/// product can have stock at other locations and at no specific location too, see
/// [`crate::stock`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ProductLocation {
    /// The location, as locations are described in [`crate::locations`].
    #[serde(flatten)]
    pub location: Location,
    /// The stock of the product at the location.
    pub stock_for_product: Option<ProductLocationStock>,
    /// A note on the link between the product and the location.
    pub note: Option<String>,
    /// Whether this is the preferred location of the product in its warehouse.
    pub is_preferred: Option<bool>,
    /// The lot number, for accounts with traceability.
    pub lot_number: Option<String>,
    /// The expiration date, for accounts with traceability.
    pub expiration_date: Option<Date>,
}

/// A location to link to a product.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewProductLocation {
    idlocation: LocationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_preferred: Option<bool>,
}

impl NewProductLocation {
    /// Links a location. Linking one that is linked already changes nothing.
    pub fn new(idlocation: LocationId) -> Self {
        Self {
            idlocation,
            note: None,
            is_preferred: None,
        }
    }

    /// Adds a note.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Makes the location the preferred one of the product, which takes that away from the one that
    /// was. There is one per warehouse.
    pub fn preferred(mut self) -> Self {
        self.is_preferred = Some(true);
        self
    }
}

/// Changes to the link between a product and a location. Only what is set is sent.
///
/// Correcting a lot number or an expiration date replaces what was recorded, without changing
/// stock or reservations. Picqer leaves a value that is not sent as it is, so neither can be cleared.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProductLocationChanges {
    fields: BTreeMap<&'static str, Value>,
}

impl ProductLocationChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the note.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.fields.insert("note", Value::String(note.into()));
        self
    }

    /// Removes the note.
    pub fn without_note(mut self) -> Self {
        self.fields.insert("note", Value::Null);
        self
    }

    /// Makes the location the preferred one of the product, which takes that away from the one that
    /// was. Picqer has no way to take it away without making another one preferred.
    pub fn preferred(mut self) -> Self {
        self.fields.insert("is_preferred", Value::Bool(true));
        self
    }

    /// Makes the location no longer the preferred one.
    pub fn not_preferred(mut self) -> Self {
        self.fields.insert("is_preferred", Value::Bool(false));
        self
    }

    /// Corrects the lot number. This needs traceability on the account and a product that tracks lot
    /// numbers, or Picqer refuses it with `422`.
    pub fn lot_number(mut self, lot_number: LotNumber) -> Self {
        self.fields.insert(
            "lot_number",
            serde_json::to_value(lot_number).unwrap_or(Value::Null),
        );
        self
    }

    /// Corrects the expiration date, which may be in the past. This needs traceability on the account
    /// and a product that tracks expiration dates, or Picqer refuses it with `422`.
    pub fn expiration_date(mut self, date: Date) -> Self {
        self.fields.insert(
            "expiration_date",
            serde_json::to_value(date).unwrap_or(Value::Null),
        );
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        Value::Object(
            self.fields
                .iter()
                .map(|(name, value)| ((*name).to_owned(), value.clone()))
                .collect(),
        )
    }
}

pub(crate) fn locations(id: ProductId) -> Call<Vec<ProductLocation>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"locations"]),
    ))
}

pub(crate) fn link_location(
    id: ProductId,
    location: &NewProductLocation,
) -> Result<Call<ProductLocation>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"products", &id, &"locations"]))
            .json_body(location)?,
    ))
}

pub(crate) fn update_location(
    id: ProductId,
    location: LocationId,
    changes: &ProductLocationChanges,
) -> Result<Call<ProductLocation>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"products", &id, &"locations", &location]),
        )
        .json_body(&changes.to_value())?,
    ))
}

pub(crate) fn unlink_location(id: ProductId, location: LocationId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"products", &id, &"locations", &location]),
    ))
}

pub(crate) fn move_stock(id: ProductId, movement: &StockMove) -> Result<Call<StockMoveResult>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"products", &id, &"move-stock"]))
            .json_body(movement)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_link_sends_what_is_set() {
        assert_eq!(
            serde_json::to_value(NewProductLocation::new(LocationId(5))).unwrap(),
            json!({"idlocation": 5})
        );
        let link = NewProductLocation::new(LocationId(5))
            .note("top shelf")
            .preferred();
        assert_eq!(
            serde_json::to_value(link).unwrap(),
            json!({"idlocation": 5, "note": "top shelf", "is_preferred": true})
        );
    }

    #[test]
    fn changes_send_only_what_is_set_and_can_remove_a_note() {
        assert_eq!(ProductLocationChanges::new().to_value(), json!({}));
        let changes = ProductLocationChanges::new()
            .without_note()
            .not_preferred()
            .lot_number(LotNumber::new("LOT-2026-001").unwrap())
            .expiration_date("2027-01-15".parse().unwrap());
        assert_eq!(
            changes.to_value(),
            json!({
                "note": null, "is_preferred": false,
                "lot_number": "LOT-2026-001", "expiration_date": "2027-01-15"
            })
        );
        assert!(LotNumber::new("").is_err());
        assert!(LotNumber::new("x".repeat(256)).is_err());
    }
}
