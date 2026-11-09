use std::num::NonZeroU32;

use serde::Serialize;
use serde_json::{Value, json};

use crate::types::{
    LocationId, NonEmpty, PicklistProductId, PicklistProductLocationId, ProductCode,
    ShippingProviderProfileId,
};

/// Changes to a picklist. Only what is set is sent.
///
/// Picqer only changes these three fields and ignores the others, such as the status.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PicklistChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    urgent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invoiced: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idshippingprovider_profile: Option<ShippingProviderProfileId>,
}

impl PicklistChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether the picklist is urgent.
    pub fn urgent(mut self, urgent: bool) -> Self {
        self.urgent = Some(urgent);
        self
    }

    /// Sets whether the picklist was invoiced.
    pub fn invoiced(mut self, invoiced: bool) -> Self {
        self.invoiced = Some(invoiced);
        self
    }

    /// Sets the shipping provider profile.
    pub fn idshippingprovider_profile(mut self, profile: ShippingProviderProfileId) -> Self {
        self.idshippingprovider_profile = Some(profile);
        self
    }
}

/// Which product of a picklist to pick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickTarget {
    /// The line of the product with this code.
    Product(ProductCode),
    /// A line of the picklist.
    Line(PicklistProductId),
    /// The part of a line that is picked at one location.
    Location(PicklistProductLocationId),
}

impl PickTarget {
    fn insert(&self, body: &mut serde_json::Map<String, Value>) {
        match self {
            Self::Product(code) => body.insert("product".into(), json!(code.as_str())),
            Self::Line(line) => body.insert("idpicklist_product".into(), json!(line)),
            Self::Location(part) => body.insert("idpicklist_product_location".into(), json!(part)),
        };
    }
}

/// Picking an amount of a product, or undoing it.
///
/// Picqer refuses more than what is left to pick, and an amount to undo that is more than was
/// picked. It treats a missing amount as one and truncates a fraction, which cannot be sent here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    target: PickTarget,
    amount: i64,
}

impl Pick {
    /// Picks `amount` of a product.
    pub fn new(target: PickTarget, amount: NonZeroU32) -> Self {
        Self {
            target,
            amount: i64::from(amount.get()),
        }
    }

    /// Undoes the picking of `amount` of a product.
    pub fn undo(target: PickTarget, amount: NonZeroU32) -> Self {
        Self {
            target,
            amount: -i64::from(amount.get()),
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut body = serde_json::Map::new();
        self.target.insert(&mut body);
        body.insert("amount".into(), json!(self.amount));
        Value::Object(body)
    }
}

/// An amount of a picklist line to pick at a location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PickLocationAmount {
    idlocation: LocationId,
    amount: NonZeroU32,
}

impl PickLocationAmount {
    /// Picks `amount` at a location. The location must be linked to the product and hold enough
    /// stock, and all amounts of a line together must equal the amount of the line.
    pub fn new(idlocation: LocationId, amount: NonZeroU32) -> Self {
        Self { idlocation, amount }
    }
}

/// An amount of a picklist line to move to a new picklist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SplitLine {
    idpicklist_product: PicklistProductId,
    amount: NonZeroU32,
    #[serde(skip_serializing_if = "Option::is_none")]
    idlocation: Option<LocationId>,
}

impl SplitLine {
    /// Splits off `amount` of a line. Only what is not picked yet can be split off.
    pub fn new(line: PicklistProductId, amount: NonZeroU32) -> Self {
        Self {
            idpicklist_product: line,
            amount,
            idlocation: None,
        }
    }

    /// Splits off what is to be picked at one location.
    pub fn at(mut self, location: LocationId) -> Self {
        self.idlocation = Some(location);
        self
    }
}

/// The lines to split off a picklist into a new picklist of the same order.
///
/// At least one line must remain on the picklist, and picklists of returns and in batches cannot
/// be split.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Split {
    products: NonEmpty<SplitLine>,
}

impl Split {
    /// Splits off these lines.
    pub fn new(products: NonEmpty<SplitLine>) -> Self {
        Self { products }
    }
}

/// An amount of a product to cancel from a picklist. The cancelled amount becomes a backorder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelProduct {
    target: CancelTarget,
    amount: NonZeroU32,
    location: Option<LocationId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CancelTarget {
    Product(ProductCode),
    Line(PicklistProductId),
}

impl CancelProduct {
    /// Cancels `amount` of the line of the product with this code.
    pub fn product(code: ProductCode, amount: NonZeroU32) -> Self {
        Self {
            target: CancelTarget::Product(code),
            amount,
            location: None,
        }
    }

    /// Cancels `amount` of a line.
    pub fn line(line: PicklistProductId, amount: NonZeroU32) -> Self {
        Self {
            target: CancelTarget::Line(line),
            amount,
            location: None,
        }
    }

    /// Cancels what is to be picked at one location.
    pub fn at(mut self, location: LocationId) -> Self {
        self.location = Some(location);
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut body = serde_json::Map::new();
        match &self.target {
            CancelTarget::Product(code) => body.insert("product".into(), json!(code.as_str())),
            CancelTarget::Line(line) => body.insert("idpicklist_product".into(), json!(line)),
        };
        body.insert("amount".into(), json!(self.amount));
        if let Some(location) = self.location {
            body.insert("idlocation".into(), json!(location));
        }
        Value::Object(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).unwrap()
    }

    #[test]
    fn a_pick_names_its_target_and_undoing_is_a_negative_amount() {
        let code = ProductCode::new("A-1").unwrap();
        assert_eq!(
            Pick::new(PickTarget::Product(code), amount(2)).to_value(),
            json!({"product": "A-1", "amount": 2})
        );
        assert_eq!(
            Pick::undo(PickTarget::Line(PicklistProductId(7)), amount(1)).to_value(),
            json!({"idpicklist_product": 7, "amount": -1})
        );
        assert_eq!(
            Pick::new(
                PickTarget::Location(PicklistProductLocationId(3)),
                amount(1)
            )
            .to_value(),
            json!({"idpicklist_product_location": 3, "amount": 1})
        );
    }

    #[test]
    fn a_split_lists_its_lines() {
        let lines = NonEmpty::new(vec![
            SplitLine::new(PicklistProductId(4821), amount(1)),
            SplitLine::new(PicklistProductId(4822), amount(2)).at(LocationId(9)),
        ])
        .unwrap();
        assert_eq!(
            serde_json::to_value(Split::new(lines)).unwrap(),
            json!({"products": [
                {"idpicklist_product": 4821, "amount": 1},
                {"idpicklist_product": 4822, "amount": 2, "idlocation": 9}
            ]})
        );
    }

    #[test]
    fn cancelling_a_product_names_the_product_or_the_line() {
        let code = ProductCode::new("A-1").unwrap();
        assert_eq!(
            CancelProduct::product(code, amount(1)).to_value(),
            json!({"product": "A-1", "amount": 1})
        );
        assert_eq!(
            CancelProduct::line(PicklistProductId(5), amount(2))
                .at(LocationId(1))
                .to_value(),
            json!({"idpicklist_product": 5, "amount": 2, "idlocation": 1})
        );
    }

    #[test]
    fn changes_send_only_what_is_set() {
        assert_eq!(
            serde_json::to_string(&PicklistChanges::new()).unwrap(),
            "{}"
        );
        assert_eq!(
            serde_json::to_string(&PicklistChanges::new().urgent(true)).unwrap(),
            r#"{"urgent":true}"#
        );
    }
}
