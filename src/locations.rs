//! Locations and containers.
//!
//! Locations are places in a warehouse where products are kept. A container is a kind of location
//! without a fixed place in the warehouse: it has only a number for a name, and the API treats it as
//! a location otherwise. Linking products to locations and moving stock are on the products, see
//! [`crate::stock`].
//!
//! The documentation describes the fields and its examples show a few more, which are read here
//! too. What a developer account also reports but the documentation never shows for this resource,
//! such as `stock_for_location` with the products of a location, is not read.

use serde_json::{Map, Value, json};

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::location_types::LocationType;
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{LocationId, LocationTypeId, Money, ProductId, WarehouseId};

bounded_string! {
    /// The name of a location. Picqer requires it to be unique.
    LocationName, "location name", 1, usize::MAX
}

string_enum! {
    /// The kind of a location.
    pub enum LocationKind {
        /// A place in the warehouse, which can be nested in another.
        Location = "location",
        /// A container, which is not at a fixed place.
        Container = "container",
    }
}

string_enum! {
    /// What a container suggests to do with a product.
    pub enum IntentType {
        /// Move the product to a location.
        ToLocation = "to_location",
    }
}

/// A location or a container.
///
/// A container has no `parent_idlocation`, `location_type` or flags, and Picqer leaves out of the
/// answer to making a location the flags that were not given, which are `false`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Location {
    /// The location.
    pub idlocation: LocationId,
    /// The warehouse.
    pub idwarehouse: WarehouseId,
    /// The kind of location.
    #[serde(rename = "type")]
    pub kind: Option<Reported<LocationKind>>,
    /// The location this one is in.
    pub parent_idlocation: Option<LocationId>,
    /// The name of the location. A container is named `Container` and a number.
    pub name: String,
    /// Remarks.
    pub remarks: Option<String>,
    /// Whether products are unlinked from the location when their stock reaches zero.
    pub unlink_on_empty: Option<bool>,
    /// The type of the location.
    pub location_type: Option<LocationType>,
    /// Whether the location holds bulk stock.
    pub is_bulk_location: Option<bool>,
    /// Whether the location can only be linked to one product.
    pub is_exclusive_location: Option<bool>,
}

/// The stock of a product in the warehouse of a location.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LocationProductStock {
    /// The warehouse.
    pub idwarehouse: Option<WarehouseId>,
    /// The stock.
    pub stock: Option<i64>,
    /// How much of it is reserved.
    pub reserved: Option<i64>,
    /// How much is reserved for backorders.
    pub reservedbackorders: Option<i64>,
    /// How much is reserved for picklists.
    pub reservedpicklists: Option<i64>,
    /// How much is reserved by allocations.
    pub reservedallocations: Option<i64>,
    /// The stock that is free.
    pub freestock: Option<i64>,
}

/// The location Picqer suggests as the destination of a product in a container.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LocationIntent {
    /// What to do.
    #[serde(rename = "type")]
    pub intent_type: Option<Reported<IntentType>>,
    /// The location to move the product to.
    pub idlocation: Option<LocationId>,
}

/// A product that is linked to a location.
///
/// The stock is that of the whole warehouse of the location, not of the location.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LocationProduct {
    /// The product.
    pub idproduct: ProductId,
    /// The product code.
    pub productcode: Option<String>,
    /// The name of the product.
    pub name: Option<String>,
    /// The price.
    pub price: Option<Money>,
    /// Where the image of the product can be downloaded.
    pub image_url: Option<String>,
    /// The stock in the warehouse.
    pub stock: Option<LocationProductStock>,
    /// The suggested destination, for a product in a container.
    pub intent: Option<LocationIntent>,
}

/// Filters for listing locations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationQuery {
    kind: Option<LocationKind>,
    idwarehouse: Option<WarehouseId>,
    name: Option<String>,
    idlocation_type: Option<LocationTypeId>,
    only_available: bool,
}

impl LocationQuery {
    /// No filters: the locations, without the containers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists locations or containers.
    pub fn kind(mut self, kind: LocationKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Lists the locations of a warehouse.
    pub fn idwarehouse(mut self, idwarehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(idwarehouse);
        self
    }

    /// Lists the location with exactly this name. A part of a name finds nothing.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Lists the locations of a type. Not for containers.
    pub fn idlocation_type(mut self, idlocation_type: LocationTypeId) -> Self {
        self.idlocation_type = Some(idlocation_type);
        self
    }

    /// Lists only the locations that have no products linked yet. Not for containers. Picqer
    /// takes any value as yes, so this is only sent when set.
    pub fn only_available(mut self) -> Self {
        self.only_available = true;
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(kind) = self.kind {
            pairs.push(("type", kind.as_str().to_owned()));
        }
        if let Some(id) = self.idwarehouse {
            pairs.push(("idwarehouse", id.to_string()));
        }
        if let Some(name) = &self.name {
            pairs.push(("name", name.clone()));
        }
        if let Some(id) = self.idlocation_type {
            pairs.push(("idlocation_type", id.to_string()));
        }
        if self.only_available {
            pairs.push(("only_available", "true".to_owned()));
        }
        pairs
    }
}

/// A location to create.
///
/// Picqer requires the name to be unique. It does not require the type, which then is the default
/// location type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLocation {
    fields: Map<String, Value>,
    auto_link_to_parent: bool,
}

impl NewLocation {
    /// A location with a name in a warehouse.
    pub fn new(name: LocationName, idwarehouse: WarehouseId) -> Self {
        let mut fields = Map::new();
        fields.insert("name".into(), json!(name));
        fields.insert("idwarehouse".into(), json!(idwarehouse));
        Self {
            fields,
            auto_link_to_parent: false,
        }
    }

    /// Puts the location in another one.
    pub fn parent_idlocation(mut self, parent: LocationId) -> Self {
        self.fields
            .insert("parent_idlocation".into(), json!(parent));
        self
    }

    /// Looks in the name for an obvious parent, such as `A.1` for `A.1.2`, and puts the location in
    /// it, making the parent first when it does not exist.
    pub fn auto_link_to_parent(mut self) -> Self {
        self.auto_link_to_parent = true;
        self
    }

    /// Sets the type of the location.
    pub fn idlocation_type(mut self, idlocation_type: LocationTypeId) -> Self {
        self.fields
            .insert("idlocation_type".into(), json!(idlocation_type));
        self
    }

    /// Sets the remarks.
    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.fields.insert("remarks".into(), json!(remarks.into()));
        self
    }

    /// Sets whether products are unlinked when their stock reaches zero.
    pub fn unlink_on_empty(mut self, unlink: bool) -> Self {
        self.fields.insert("unlink_on_empty".into(), json!(unlink));
        self
    }

    /// Sets whether the location holds bulk stock.
    pub fn is_bulk_location(mut self, bulk: bool) -> Self {
        self.fields.insert("is_bulk_location".into(), json!(bulk));
        self
    }

    /// Sets whether the location can only be linked to one product.
    pub fn is_exclusive_location(mut self, exclusive: bool) -> Self {
        self.fields
            .insert("is_exclusive_location".into(), json!(exclusive));
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        Value::Object(self.fields.clone())
    }
}

/// A container to create. Picqer names it `Container` and the next number, and ignores a name that
/// is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewContainer {
    idwarehouse: WarehouseId,
}

impl NewContainer {
    /// A container in a warehouse.
    pub fn new(idwarehouse: WarehouseId) -> Self {
        Self { idwarehouse }
    }

    pub(crate) fn to_value(self) -> Value {
        json!({"idwarehouse": self.idwarehouse, "type": "container"})
    }
}

/// Changes to a location or a container. Only what is set is sent.
///
/// Picqer ignores a new name for a container, and a container shows no other fields than its
/// `remarks`. Picqer ignores a new warehouse for a location.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationChanges {
    fields: Map<String, Value>,
}

impl LocationChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the name.
    pub fn name(mut self, name: LocationName) -> Self {
        self.fields.insert("name".into(), json!(name));
        self
    }

    /// Changes the remarks.
    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.fields.insert("remarks".into(), json!(remarks.into()));
        self
    }

    /// Removes the remarks.
    pub fn without_remarks(mut self) -> Self {
        self.fields.insert("remarks".into(), Value::Null);
        self
    }

    /// Puts the location in another one. Picqer refuses a location that would end up in itself.
    pub fn parent_idlocation(mut self, parent: LocationId) -> Self {
        self.fields
            .insert("parent_idlocation".into(), json!(parent));
        self
    }

    /// Takes the location out of its parent.
    pub fn without_parent(mut self) -> Self {
        self.fields.insert("parent_idlocation".into(), Value::Null);
        self
    }

    /// Changes the type of the location.
    pub fn idlocation_type(mut self, idlocation_type: LocationTypeId) -> Self {
        self.fields
            .insert("idlocation_type".into(), json!(idlocation_type));
        self
    }

    /// Changes whether products are unlinked when their stock reaches zero.
    pub fn unlink_on_empty(mut self, unlink: bool) -> Self {
        self.fields.insert("unlink_on_empty".into(), json!(unlink));
        self
    }

    /// Changes whether the location holds bulk stock.
    pub fn is_bulk_location(mut self, bulk: bool) -> Self {
        self.fields.insert("is_bulk_location".into(), json!(bulk));
        self
    }

    /// Changes whether the location can only be linked to one product.
    pub fn is_exclusive_location(mut self, exclusive: bool) -> Self {
        self.fields
            .insert("is_exclusive_location".into(), json!(exclusive));
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        Value::Object(self.fields.clone())
    }
}

pub(crate) fn list(query: &LocationQuery, page: Page) -> Call<Vec<Location>> {
    let mut request = Request::new(Method::GET, segments(&[&"locations"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: LocationId) -> Call<Location> {
    Call::new(Request::new(Method::GET, segments(&[&"locations", &id])))
}

pub(crate) fn create(location: &NewLocation) -> Result<Call<Location>> {
    let mut request = Request::new(Method::POST, segments(&[&"locations"]));
    if location.auto_link_to_parent {
        request = request.query("auto_link_to_parent", "1");
    }
    Ok(Call::new(request.json_body(&location.to_value())?))
}

pub(crate) fn create_container(container: NewContainer) -> Result<Call<Location>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"locations"])).json_body(&container.to_value())?,
    ))
}

pub(crate) fn update(id: LocationId, changes: &LocationChanges) -> Result<Call<Location>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"locations", &id])).json_body(&changes.to_value())?,
    ))
}

pub(crate) fn delete(id: LocationId) -> NoContent {
    NoContent::new(Request::new(Method::DELETE, segments(&[&"locations", &id])))
}

pub(crate) fn products(id: LocationId, page: Page) -> Call<Vec<LocationProduct>> {
    Call::new(page.apply(Request::new(
        Method::GET,
        segments(&[&"locations", &id, &"products"]),
    )))
}

pub(crate) fn set_intent(
    id: LocationId,
    product: ProductId,
    destination: Option<LocationId>,
) -> Result<Call<LocationIntent>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"locations", &id, &"products", &product, &"intent"]),
        )
        .json_body(&json!({"idlocation": destination}))?,
    ))
}

macro_rules! locations_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of locations. Without a query these are the locations and not the
        /// containers.
        pub $($async)? fn list(
            &self,
            query: &$crate::locations::LocationQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::locations::Location>> {
            self.client.execute($crate::locations::list(query, page)) $(.$await)?
        }

        /// Lists all locations that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::locations::LocationQuery,
        ) -> $crate::error::Result<Vec<$crate::locations::Location>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::locations::list(query, page)) $(.$await)?)
        }

        /// Fetches a location or a container.
        pub $($async)? fn get(
            &self,
            id: $crate::types::LocationId,
        ) -> $crate::error::Result<$crate::locations::Location> {
            self.client.execute($crate::locations::get(id)) $(.$await)?
        }

        /// Creates a location.
        pub $($async)? fn create(
            &self,
            location: &$crate::locations::NewLocation,
        ) -> $crate::error::Result<$crate::locations::Location> {
            self.client.execute($crate::locations::create(location)?) $(.$await)?
        }

        /// Creates a container.
        pub $($async)? fn create_container(
            &self,
            container: $crate::locations::NewContainer,
        ) -> $crate::error::Result<$crate::locations::Location> {
            self.client.execute($crate::locations::create_container(container)?) $(.$await)?
        }

        /// Changes a location or a container.
        pub $($async)? fn update(
            &self,
            id: $crate::types::LocationId,
            changes: &$crate::locations::LocationChanges,
        ) -> $crate::error::Result<$crate::locations::Location> {
            self.client.execute($crate::locations::update(id, changes)?) $(.$await)?
        }

        /// Deletes a location or a container. Picqer refuses one that has products linked.
        pub $($async)? fn delete(
            &self,
            id: $crate::types::LocationId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::locations::delete(id)) $(.$await)?
        }

        /// Lists one page of the products linked to a location, with or without stock.
        pub $($async)? fn products(
            &self,
            id: $crate::types::LocationId,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::locations::LocationProduct>> {
            self.client.execute($crate::locations::products(id, page)) $(.$await)?
        }

        /// Lists all products linked to a location, following the pages.
        pub $($async)? fn products_all(
            &self,
            id: $crate::types::LocationId,
        ) -> $crate::error::Result<Vec<$crate::locations::LocationProduct>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::locations::products(id, page)) $(.$await)?)
        }

        /// Changes the location suggested as the destination of a product in a container, or
        /// takes the suggestion away with `None`. Picqer refuses a location that is not a
        /// container and a product that is not in it.
        pub $($async)? fn set_intent(
            &self,
            container: $crate::types::LocationId,
            product: $crate::types::ProductId,
            destination: Option<$crate::types::LocationId>,
        ) -> $crate::error::Result<$crate::locations::LocationIntent> {
            self.client.execute($crate::locations::set_intent(container, product, destination)?) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use locations_methods;

/// Operations on locations.
#[derive(Debug, Clone, Copy)]
pub struct Locations<'a> {
    pub(crate) client: &'a Client,
}

impl Locations<'_> {
    locations_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_location_sends_what_is_set() {
        let location = NewLocation::new(LocationName::new("A.1.2").unwrap(), WarehouseId(13))
            .parent_idlocation(LocationId(4))
            .remarks("forward pick")
            .is_bulk_location(true);
        assert_eq!(
            location.to_value(),
            json!({
                "name": "A.1.2", "idwarehouse": 13, "parent_idlocation": 4,
                "remarks": "forward pick", "is_bulk_location": true
            })
        );
        assert!(LocationName::new("").is_err());
    }

    #[test]
    fn a_container_is_a_warehouse_and_a_kind() {
        assert_eq!(
            NewContainer::new(WarehouseId(13)).to_value(),
            json!({"idwarehouse": 13, "type": "container"})
        );
    }

    #[test]
    fn changes_can_clear_the_remarks_and_the_parent() {
        assert_eq!(LocationChanges::new().to_value(), json!({}));
        let changes = LocationChanges::new().without_remarks().without_parent();
        assert_eq!(
            changes.to_value(),
            json!({"remarks": null, "parent_idlocation": null})
        );
    }

    #[test]
    fn filters_become_query_pairs() {
        assert!(LocationQuery::new().pairs().is_empty());
        let query = LocationQuery::new()
            .kind(LocationKind::Container)
            .idwarehouse(WarehouseId(13))
            .name("A.B.12")
            .idlocation_type(LocationTypeId(2))
            .only_available();
        assert_eq!(
            query.pairs(),
            vec![
                ("type", "container".to_owned()),
                ("idwarehouse", "13".to_owned()),
                ("name", "A.B.12".to_owned()),
                ("idlocation_type", "2".to_owned()),
                ("only_available", "true".to_owned()),
            ]
        );
    }
}
