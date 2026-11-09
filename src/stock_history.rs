//! The stock history.
//!
//! Every change in the stock of a product is kept. The history cannot be changed.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{DateTime, LocationId, ProductId, StockHistoryId, UserId, WarehouseId};

string_enum! {
    /// What caused a change in stock.
    pub enum ChangeType {
        /// Through the API.
        Api = "api",
        /// An import.
        Import = "import",
        /// The first import of stock.
        InitialImport = "initialimport",
        /// By hand.
        Manual = "manual",
        /// Moved between locations or warehouses.
        Moved = "moved",
        /// Picking.
        Picking = "picking",
        /// A picking that was rolled back.
        PickingRollback = "pickingrollback",
        /// Production.
        Production = "production",
        /// A purchase order.
        PurchaseOrder = "purchaseorder",
        /// A quick scan.
        QuickScan = "quickscan",
        /// A receipt.
        Receipt = "receipt",
        /// A return.
        Returned = "returned",
        /// A stock count.
        StockCount = "stockcount",
        /// An import from a webshop.
        WebshopImport = "webshopimport",
    }
}

/// One change in the stock of a product.
///
/// The documentation only shows examples, so what is not an identifier or a number of the change
/// is optional here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct StockHistoryEntry {
    /// The entry.
    pub idproduct_stock_history: StockHistoryId,
    /// The product.
    pub idproduct: ProductId,
    /// The warehouse.
    pub idwarehouse: WarehouseId,
    /// The user that made the change.
    pub iduser: Option<UserId>,
    /// The location, when the change was at a specific location.
    pub idlocation: Option<LocationId>,
    /// The stock before the change.
    pub old_stock: i64,
    /// The change.
    pub stock_change: i64,
    /// The stock after the change.
    pub new_stock: i64,
    /// The reason that was given.
    pub reason: Option<String>,
    /// What caused the change.
    pub change_type: Option<Reported<ChangeType>>,
    /// When the stock changed.
    pub changed_at: Option<DateTime>,
}

/// Filters for listing the stock history.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StockHistoryQuery {
    since: Option<DateTime>,
    until: Option<DateTime>,
    idproduct: Option<ProductId>,
    idwarehouse: Option<WarehouseId>,
    iduser: Option<UserId>,
}

impl StockHistoryQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the history added after this moment.
    pub fn since(mut self, moment: DateTime) -> Self {
        self.since = Some(moment);
        self
    }

    /// Lists the history added before this moment.
    pub fn until(mut self, moment: DateTime) -> Self {
        self.until = Some(moment);
        self
    }

    /// Lists the history of a product.
    pub fn idproduct(mut self, idproduct: ProductId) -> Self {
        self.idproduct = Some(idproduct);
        self
    }

    /// Lists the history in a warehouse.
    pub fn idwarehouse(mut self, idwarehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(idwarehouse);
        self
    }

    /// Lists the history of changes made by a user.
    pub fn iduser(mut self, iduser: UserId) -> Self {
        self.iduser = Some(iduser);
        self
    }

    fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(since) = self.since {
            pairs.push(("sincedate", since.to_string()));
        }
        if let Some(until) = self.until {
            pairs.push(("untildate", until.to_string()));
        }
        if let Some(idproduct) = self.idproduct {
            pairs.push(("idproduct", idproduct.to_string()));
        }
        if let Some(idwarehouse) = self.idwarehouse {
            pairs.push(("idwarehouse", idwarehouse.to_string()));
        }
        if let Some(iduser) = self.iduser {
            pairs.push(("iduser", iduser.to_string()));
        }
        pairs
    }
}

pub(crate) fn list(query: &StockHistoryQuery, page: Page) -> Call<Vec<StockHistoryEntry>> {
    let mut request = Request::new(Method::GET, segments(&[&"stockhistory"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: StockHistoryId) -> Call<StockHistoryEntry> {
    Call::new(Request::new(Method::GET, segments(&[&"stockhistory", &id])))
}

macro_rules! stock_history_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of the stock history.
        pub $($async)? fn list(
            &self,
            query: &$crate::stock_history::StockHistoryQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::stock_history::StockHistoryEntry>> {
            self.client.execute($crate::stock_history::list(query, page)) $(.$await)?
        }

        /// Lists all the stock history that matches a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::stock_history::StockHistoryQuery,
        ) -> $crate::error::Result<Vec<$crate::stock_history::StockHistoryEntry>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::stock_history::list(query, page)) $(.$await)?)
        }

        /// Fetches one entry of the stock history.
        pub $($async)? fn get(
            &self,
            id: $crate::types::StockHistoryId,
        ) -> $crate::error::Result<$crate::stock_history::StockHistoryEntry> {
            self.client.execute($crate::stock_history::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use stock_history_methods;

/// Operations on the stock history.
#[derive(Debug, Clone, Copy)]
pub struct StockHistory<'a> {
    pub(crate) client: &'a Client,
}

impl StockHistory<'_> {
    stock_history_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_use_their_wire_names_and_formats() {
        let moment: DateTime = "2026-11-09 14:00:00".parse().unwrap();
        let query = StockHistoryQuery::new()
            .since(moment)
            .idproduct(ProductId(7))
            .iduser(UserId(3));
        assert_eq!(
            query.pairs(),
            vec![
                ("sincedate", "2026-11-09 14:00:00".to_owned()),
                ("idproduct", "7".to_owned()),
                ("iduser", "3".to_owned()),
            ]
        );
    }
}
