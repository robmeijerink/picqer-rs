//! Picking containers.
//!
//! A picking container is a temporary bin or bag to collect the products of a picklist in, used with
//! [picklist batches](crate::picklist_batches). A picklist can be linked to a container, and when the
//! barcode of the container is scanned, Picqer opens its picklist.

use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::types::{DateTime, PickingContainerId, PicklistId};

bounded_string! {
    /// The name of a picking container. Picqer requires it to be unique.
    PickingContainerName, "picking container name", 1, usize::MAX
}

/// A picking container.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PickingContainer {
    /// The picking container.
    pub idpicking_container: PickingContainerId,
    /// The picklist that is linked to the container.
    pub idpicklist: Option<PicklistId>,
    /// The name. Picqer answers a name that is a number as a number once, when the container is
    /// made, and as text after that, so a number is read as text.
    #[serde(deserialize_with = "crate::serde_util::text")]
    pub name: String,
    /// When the container was made.
    pub created_at: Option<DateTime>,
    /// When the container was last changed.
    pub updated_at: Option<DateTime>,
}

/// Filters for listing picking containers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PickingContainerQuery {
    idpicklist: Option<PicklistId>,
}

impl PickingContainerQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the container a picklist is linked to.
    pub fn idpicklist(mut self, idpicklist: PicklistId) -> Self {
        self.idpicklist = Some(idpicklist);
        self
    }

    pub(crate) fn pairs(self) -> Vec<(&'static str, String)> {
        self.idpicklist
            .map(|id| vec![("idpicklist", id.to_string())])
            .unwrap_or_default()
    }
}

#[derive(Serialize)]
struct Name<'a> {
    name: &'a PickingContainerName,
}

pub(crate) fn list(query: PickingContainerQuery, page: Page) -> Call<Vec<PickingContainer>> {
    let mut request = Request::new(Method::GET, segments(&[&"picking-containers"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: PickingContainerId) -> Call<PickingContainer> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"picking-containers", &id]),
    ))
}

pub(crate) fn create(name: &PickingContainerName) -> Result<Call<PickingContainer>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"picking-containers"])).json_body(&Name { name })?,
    ))
}

pub(crate) fn rename(
    id: PickingContainerId,
    name: &PickingContainerName,
) -> Result<Call<PickingContainer>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"picking-containers", &id]))
            .json_body(&Name { name })?,
    ))
}

pub(crate) fn delete(id: PickingContainerId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"picking-containers", &id]),
    ))
}

pub(crate) fn link(id: PickingContainerId, picklist: PicklistId) -> Result<Call<PickingContainer>> {
    Ok(Call::new(
        Request::new(
            Method::POST,
            segments(&[&"picking-containers", &id, &"link"]),
        )
        .json_body(&json!({"idpicklist": picklist}))?,
    ))
}

macro_rules! picking_containers_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of picking containers.
        pub $($async)? fn list(
            &self,
            query: $crate::picking_containers::PickingContainerQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::picking_containers::PickingContainer>> {
            self.client.execute($crate::picking_containers::list(query, page)) $(.$await)?
        }

        /// Lists all picking containers that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: $crate::picking_containers::PickingContainerQuery,
        ) -> $crate::error::Result<Vec<$crate::picking_containers::PickingContainer>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::picking_containers::list(query, page)) $(.$await)?)
        }

        /// Fetches a picking container.
        pub $($async)? fn get(
            &self,
            id: $crate::types::PickingContainerId,
        ) -> $crate::error::Result<$crate::picking_containers::PickingContainer> {
            self.client.execute($crate::picking_containers::get(id)) $(.$await)?
        }

        /// Creates a picking container with a name.
        pub $($async)? fn create(
            &self,
            name: &$crate::picking_containers::PickingContainerName,
        ) -> $crate::error::Result<$crate::picking_containers::PickingContainer> {
            self.client.execute($crate::picking_containers::create(name)?) $(.$await)?
        }

        /// Changes the name of a picking container.
        pub $($async)? fn rename(
            &self,
            id: $crate::types::PickingContainerId,
            name: &$crate::picking_containers::PickingContainerName,
        ) -> $crate::error::Result<$crate::picking_containers::PickingContainer> {
            self.client.execute($crate::picking_containers::rename(id, name)?) $(.$await)?
        }

        /// Deletes a picking container, also when a picklist is linked to it.
        pub $($async)? fn delete(
            &self,
            id: $crate::types::PickingContainerId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::picking_containers::delete(id)) $(.$await)?
        }

        /// Links a picklist to a picking container. A container holds one picklist, so a picklist
        /// that was linked takes the place of the one before it, and a picklist that was in another
        /// container leaves it. Picqer has no call to unlink one without deleting the container.
        pub $($async)? fn link(
            &self,
            id: $crate::types::PickingContainerId,
            picklist: $crate::types::PicklistId,
        ) -> $crate::error::Result<$crate::picking_containers::PickingContainer> {
            self.client.execute($crate::picking_containers::link(id, picklist)?) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use picking_containers_methods;

/// Operations on picking containers.
#[derive(Debug, Clone, Copy)]
pub struct PickingContainers<'a> {
    pub(crate) client: &'a Client,
}

impl PickingContainers<'_> {
    picking_containers_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_not_empty() {
        assert!(PickingContainerName::new("").is_err());
        assert_eq!(
            serde_json::to_value(Name {
                name: &PickingContainerName::new("Blue bin").unwrap()
            })
            .unwrap(),
            json!({"name": "Blue bin"})
        );
    }

    #[test]
    fn the_filter_becomes_a_query_pair() {
        assert!(PickingContainerQuery::new().pairs().is_empty());
        assert_eq!(
            PickingContainerQuery::new()
                .idpicklist(PicklistId(8))
                .pairs(),
            vec![("idpicklist", "8".to_owned())]
        );
    }
}
