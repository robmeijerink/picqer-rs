//! Location types.
//!
//! A location type tells locations apart, for example by their size. Every account has a default
//! type, which a new location gets when none is given.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::tags::HexColor;
use crate::types::LocationTypeId;

bounded_string! {
    /// The name of a location type. Picqer requires it to be unique.
    LocationTypeName, "location type name", 1, usize::MAX
}

/// A kind of location.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LocationType {
    /// The location type.
    pub idlocation_type: LocationTypeId,
    /// The name of the type.
    pub name: String,
    /// Whether this is the default type. Picqer leaves it out of the answer to making a type,
    /// where it is `false`.
    pub default: Option<bool>,
    /// The color of the type. Picqer does not check it, so it is not read as a [`HexColor`]: a type
    /// with another color would make a whole list fail.
    pub color: String,
}

/// A location type to create.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewLocationType {
    name: LocationTypeName,
    color: HexColor,
}

impl NewLocationType {
    /// A location type with a name and a color.
    pub fn new(name: LocationTypeName, color: HexColor) -> Self {
        Self { name, color }
    }
}

/// Changes to a location type. Only what is set is sent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LocationTypeChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<LocationTypeName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<HexColor>,
}

impl LocationTypeChanges {
    /// No changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the name.
    pub fn name(mut self, name: LocationTypeName) -> Self {
        self.name = Some(name);
        self
    }

    /// Changes the color.
    pub fn color(mut self, color: HexColor) -> Self {
        self.color = Some(color);
        self
    }
}

pub(crate) fn list(page: Page) -> Call<Vec<LocationType>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"location_types"]))))
}

pub(crate) fn get(id: LocationTypeId) -> Call<LocationType> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"location_types", &id]),
    ))
}

pub(crate) fn create(location_type: &NewLocationType) -> Result<Call<LocationType>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"location_types"])).json_body(location_type)?,
    ))
}

pub(crate) fn update(
    id: LocationTypeId,
    changes: &LocationTypeChanges,
) -> Result<Call<LocationType>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"location_types", &id])).json_body(changes)?,
    ))
}

pub(crate) fn delete(id: LocationTypeId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"location_types", &id]),
    ))
}

pub(crate) fn set_default(id: LocationTypeId) -> Call<LocationType> {
    Call::new(Request::new(
        Method::POST,
        segments(&[&"location_types", &id, &"default"]),
    ))
}

macro_rules! location_types_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of location types.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::location_types::LocationType>> {
            self.client.execute($crate::location_types::list(page)) $(.$await)?
        }

        /// Lists all location types, following the pages.
        pub $($async)? fn list_all(
            &self,
        ) -> $crate::error::Result<Vec<$crate::location_types::LocationType>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::location_types::list(page)) $(.$await)?)
        }

        /// Fetches a location type.
        pub $($async)? fn get(
            &self,
            id: $crate::types::LocationTypeId,
        ) -> $crate::error::Result<$crate::location_types::LocationType> {
            self.client.execute($crate::location_types::get(id)) $(.$await)?
        }

        /// Creates a location type.
        pub $($async)? fn create(
            &self,
            location_type: &$crate::location_types::NewLocationType,
        ) -> $crate::error::Result<$crate::location_types::LocationType> {
            self.client.execute($crate::location_types::create(location_type)?) $(.$await)?
        }

        /// Changes a location type.
        pub $($async)? fn update(
            &self,
            id: $crate::types::LocationTypeId,
            changes: &$crate::location_types::LocationTypeChanges,
        ) -> $crate::error::Result<$crate::location_types::LocationType> {
            self.client.execute($crate::location_types::update(id, changes)?) $(.$await)?
        }

        /// Deletes a location type. Picqer refuses the default type with code 5, and the
        /// documentation says it refuses one that locations still use.
        pub $($async)? fn delete(
            &self,
            id: $crate::types::LocationTypeId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::location_types::delete(id)) $(.$await)?
        }

        /// Makes a location type the default, which takes that away from the previous one: a
        /// list then has one default type.
        pub $($async)? fn set_default(
            &self,
            id: $crate::types::LocationTypeId,
        ) -> $crate::error::Result<$crate::location_types::LocationType> {
            self.client.execute($crate::location_types::set_default(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use location_types_methods;

/// Operations on location types.
#[derive(Debug, Clone, Copy)]
pub struct LocationTypes<'a> {
    pub(crate) client: &'a Client,
}

impl LocationTypes<'_> {
    location_types_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_type_is_sent_with_a_name_and_a_color() {
        let new = NewLocationType::new(
            LocationTypeName::new("Bulk").unwrap(),
            HexColor::new("#32CD32").unwrap(),
        );
        assert_eq!(
            serde_json::to_value(&new).unwrap(),
            json!({"name": "Bulk", "color": "#32CD32"})
        );
        assert!(LocationTypeName::new("").is_err());
    }

    #[test]
    fn only_what_is_set_is_changed() {
        assert_eq!(
            serde_json::to_value(LocationTypeChanges::new()).unwrap(),
            json!({})
        );
        let changes = LocationTypeChanges::new().color(HexColor::new("#FF5733").unwrap());
        assert_eq!(
            serde_json::to_value(changes).unwrap(),
            json!({"color": "#FF5733"})
        );
    }

    #[test]
    fn a_color_that_is_not_a_hex_color_is_still_read() {
        let read: LocationType = serde_json::from_value(
            json!({"idlocation_type": 1, "name": "X", "default": null, "color": "red"}),
        )
        .unwrap();
        assert_eq!(read.color, "red");
        assert_eq!(read.default, None);
    }
}
