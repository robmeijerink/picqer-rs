//! The stock history per location.
//!
//! [`crate::stock_history`] holds the changes of the stock of a product per warehouse. This holds
//! them per location, so it also shows stock that is moved inside a warehouse. The history cannot
//! be changed.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{DateTime, LocationId, LocationStockHistoryId, ProductId, UserId};

string_enum! {
    /// What caused a change in the stock of a location.
    pub enum LocationChangeType {
        /// A change in stock, the same as in the stock history of the product.
        Mutation = "mutation",
        /// Stock moved from one location to another, which gives an entry for each location.
        Movement = "movement",
    }
}

/// One change in the stock of a product at a location.
///
/// The documentation only shows examples, so what is not an identifier is optional here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LocationStockHistoryEntry {
    /// The entry.
    pub idproduct_location_stock_history: LocationStockHistoryId,
    /// The product.
    pub idproduct: ProductId,
    /// The location.
    pub idlocation: LocationId,
    /// The other location of a movement.
    pub contra_idlocation: Option<LocationId>,
    /// The user that made the change.
    pub iduser: Option<UserId>,
    /// The stock at the location before the change.
    pub old_stock: Option<i64>,
    /// How much the stock changed.
    pub stock_change: Option<i64>,
    /// The stock at the location after the change.
    pub new_stock: Option<i64>,
    /// What caused the change.
    pub change_type: Option<Reported<LocationChangeType>>,
    /// When the stock changed.
    pub changed_at: Option<DateTime>,
}

/// Filters for the stock history of locations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationStockHistoryQuery {
    since: Option<DateTime>,
    until: Option<DateTime>,
    idproduct: Option<ProductId>,
    idlocation: Option<LocationId>,
    iduser: Option<UserId>,
}

impl LocationStockHistoryQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the entries added after this moment.
    pub fn since(mut self, moment: DateTime) -> Self {
        self.since = Some(moment);
        self
    }

    /// Lists the entries added before this moment.
    pub fn until(mut self, moment: DateTime) -> Self {
        self.until = Some(moment);
        self
    }

    /// Lists the entries of a product.
    pub fn idproduct(mut self, idproduct: ProductId) -> Self {
        self.idproduct = Some(idproduct);
        self
    }

    /// Lists the entries at a location.
    pub fn idlocation(mut self, idlocation: LocationId) -> Self {
        self.idlocation = Some(idlocation);
        self
    }

    /// Lists the entries of a user.
    pub fn iduser(mut self, iduser: UserId) -> Self {
        self.iduser = Some(iduser);
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(moment) = self.since {
            pairs.push(("sincedate", moment.to_string()));
        }
        if let Some(moment) = self.until {
            pairs.push(("untildate", moment.to_string()));
        }
        if let Some(id) = self.idproduct {
            pairs.push(("idproduct", id.to_string()));
        }
        if let Some(id) = self.idlocation {
            pairs.push(("idlocation", id.to_string()));
        }
        if let Some(id) = self.iduser {
            pairs.push(("iduser", id.to_string()));
        }
        pairs
    }
}

pub(crate) fn list(
    query: &LocationStockHistoryQuery,
    page: Page,
) -> Call<Vec<LocationStockHistoryEntry>> {
    let mut request = Request::new(Method::GET, segments(&[&"location-stock-history"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: LocationStockHistoryId) -> Call<LocationStockHistoryEntry> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"location-stock-history", &id]),
    ))
}

macro_rules! location_stock_history_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of the stock history of locations, newest first.
        pub $($async)? fn list(
            &self,
            query: &$crate::location_stock_history::LocationStockHistoryQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::location_stock_history::LocationStockHistoryEntry>> {
            self.client.execute($crate::location_stock_history::list(query, page)) $(.$await)?
        }

        /// Lists all entries that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::location_stock_history::LocationStockHistoryQuery,
        ) -> $crate::error::Result<Vec<$crate::location_stock_history::LocationStockHistoryEntry>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::location_stock_history::list(query, page)) $(.$await)?)
        }

        /// Fetches an entry.
        pub $($async)? fn get(
            &self,
            id: $crate::types::LocationStockHistoryId,
        ) -> $crate::error::Result<$crate::location_stock_history::LocationStockHistoryEntry> {
            self.client.execute($crate::location_stock_history::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use location_stock_history_methods;

/// Operations on the stock history of locations.
#[derive(Debug, Clone, Copy)]
pub struct LocationStockHistory<'a> {
    pub(crate) client: &'a Client,
}

impl LocationStockHistory<'_> {
    location_stock_history_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_become_query_pairs() {
        assert!(LocationStockHistoryQuery::new().pairs().is_empty());
        let moment: DateTime = "2026-11-09 16:51:05".parse().unwrap();
        let query = LocationStockHistoryQuery::new()
            .since(moment)
            .until(moment)
            .idproduct(ProductId(1))
            .idlocation(LocationId(2))
            .iduser(UserId(3));
        assert_eq!(
            query.pairs(),
            vec![
                ("sincedate", "2026-11-09 16:51:05".to_owned()),
                ("untildate", "2026-11-09 16:51:05".to_owned()),
                ("idproduct", "1".to_owned()),
                ("idlocation", "2".to_owned()),
                ("iduser", "3".to_owned()),
            ]
        );
    }
}
