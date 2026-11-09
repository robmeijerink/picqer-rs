//! Webshop orders.
//!
//! A webshop order is an order of a webshop integration that Picqer imports, or has imported, as an
//! order of its own. A developer account has no webshop integration, so this follows the
//! documentation and has not been compared with real responses.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{CompanyWebshopId, DateTime, OrderId, WebshopOrderId};

string_enum! {
    /// How far the import of a webshop order is.
    ///
    /// The documentation gives these as examples, so another status is kept as `Reported::Unknown`.
    pub enum WebshopOrderStatus {
        /// The order is expected to be imported.
        Expected = "expected",
        /// The order is being imported.
        Importing = "importing",
        /// The order is ignored.
        Ignored = "ignored",
        /// The order is imported.
        Imported = "imported",
        /// The import failed, and `reason` says why.
        Error = "error",
        /// The order is cancelled.
        Canceled = "canceled",
    }
}

/// An order of a webshop integration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct WebshopOrder {
    /// The webshop order.
    pub idcompany_webshop_order: WebshopOrderId,
    /// The webshop integration.
    pub idcompany_webshop: CompanyWebshopId,
    /// The order in Picqer, once the webshop order is imported.
    pub idorder: Option<OrderId>,
    /// The identifier of the order in the webshop.
    #[serde(deserialize_with = "crate::serde_util::text")]
    pub foreign_id: String,
    /// The number of the order as the webshop shows it.
    #[serde(default, deserialize_with = "crate::serde_util::opt_text")]
    pub foreign_number: Option<String>,
    /// The last known status of the order in the webshop.
    pub foreign_status: Option<String>,
    /// How far the import is.
    pub status: Reported<WebshopOrderStatus>,
    /// More about the status, such as an error message.
    pub reason: Option<String>,
    /// When the order was placed in the webshop.
    pub ordered: DateTime,
    /// When the webshop order was made in Picqer.
    pub created: DateTime,
    /// When the webshop order was last changed in Picqer.
    pub updated: DateTime,
}

/// Filters for listing webshop orders.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WebshopOrderQuery {
    foreign_id: Option<String>,
    foreign_number: Option<String>,
    idorder: Option<OrderId>,
    idcompany_webshop: Option<CompanyWebshopId>,
}

impl WebshopOrderQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the order with this identifier in the webshop.
    pub fn foreign_id(mut self, foreign_id: impl Into<String>) -> Self {
        self.foreign_id = Some(foreign_id.into());
        self
    }

    /// Lists the order with this number in the webshop.
    pub fn foreign_number(mut self, foreign_number: impl Into<String>) -> Self {
        self.foreign_number = Some(foreign_number.into());
        self
    }

    /// Lists the webshop order that became this order.
    pub fn idorder(mut self, idorder: OrderId) -> Self {
        self.idorder = Some(idorder);
        self
    }

    /// Lists the orders of one webshop integration.
    pub fn idcompany_webshop(mut self, idcompany_webshop: CompanyWebshopId) -> Self {
        self.idcompany_webshop = Some(idcompany_webshop);
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(value) = &self.foreign_id {
            pairs.push(("foreign_id", value.clone()));
        }
        if let Some(value) = &self.foreign_number {
            pairs.push(("foreign_number", value.clone()));
        }
        if let Some(id) = self.idorder {
            pairs.push(("idorder", id.to_string()));
        }
        if let Some(id) = self.idcompany_webshop {
            pairs.push(("idcompany_webshop", id.to_string()));
        }
        pairs
    }
}

pub(crate) fn list(query: &WebshopOrderQuery, page: Page) -> Call<Vec<WebshopOrder>> {
    let mut request = Request::new(Method::GET, segments(&[&"webshoporders"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: WebshopOrderId) -> Call<WebshopOrder> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"webshoporders", &id]),
    ))
}

macro_rules! webshop_orders_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of webshop orders.
        pub $($async)? fn list(
            &self,
            query: &$crate::webshop_orders::WebshopOrderQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::webshop_orders::WebshopOrder>> {
            self.client.execute($crate::webshop_orders::list(query, page)) $(.$await)?
        }

        /// Lists all webshop orders that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::webshop_orders::WebshopOrderQuery,
        ) -> $crate::error::Result<Vec<$crate::webshop_orders::WebshopOrder>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::webshop_orders::list(query, page)) $(.$await)?)
        }

        /// Fetches a webshop order.
        pub $($async)? fn get(
            &self,
            id: $crate::types::WebshopOrderId,
        ) -> $crate::error::Result<$crate::webshop_orders::WebshopOrder> {
            self.client.execute($crate::webshop_orders::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use webshop_orders_methods;

/// Operations on webshop orders.
#[derive(Debug, Clone, Copy)]
pub struct WebshopOrders<'a> {
    pub(crate) client: &'a Client,
}

impl WebshopOrders<'_> {
    webshop_orders_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_become_query_pairs() {
        assert!(WebshopOrderQuery::new().pairs().is_empty());
        let query = WebshopOrderQuery::new()
            .foreign_id("ord_1234")
            .foreign_number("20250001")
            .idorder(OrderId(362231))
            .idcompany_webshop(CompanyWebshopId(1));
        assert_eq!(
            query.pairs(),
            vec![
                ("foreign_id", "ord_1234".to_owned()),
                ("foreign_number", "20250001".to_owned()),
                ("idorder", "362231".to_owned()),
                ("idcompany_webshop", "1".to_owned()),
            ]
        );
    }
}
