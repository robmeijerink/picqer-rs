//! Backorders.
//!
//! A backorder is an order line that cannot be picked yet because of the stock. Picqer makes one
//! when an order is processed and the product is not available, or when a part of the order may not
//! be delivered alone and a product of the order has too little stock.
//!
//! The documentation describes the fields and its examples show a few more, which are read here too.
//! What a developer account also reports but the documentation never shows, such as the duplicate
//! `amountavailable`, is not read.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::types::{
    BackorderId, CustomerId, DateTime, FulfilmentCustomerId, OrderId, OrderProductId, ProductId,
    ReturnId, WarehouseId,
};

/// An order line that cannot be picked yet.
///
/// Only a single backorder and the backorders of an order come with `is_purchased` and
/// `date_available`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Backorder {
    /// The backorder.
    pub idbackorder: BackorderId,
    /// The line of the order.
    pub idorder_product: Option<OrderProductId>,
    /// The order.
    pub idorder: OrderId,
    /// The return, for a backorder of a replacement product.
    pub idreturn: Option<ReturnId>,
    /// The product.
    pub idproduct: ProductId,
    /// The customer.
    pub idcustomer: Option<CustomerId>,
    /// The warehouse.
    pub idwarehouse: Option<WarehouseId>,
    /// How many are in backorder.
    pub amount: i64,
    /// How many of them are in stock and reserved for this backorder.
    pub amount_available: Option<i64>,
    /// The priority. The lower it is, the earlier stock goes to this backorder.
    pub priority: Option<i64>,
    /// Whether the product is a composition.
    pub has_parts: Option<bool>,
    /// The backorder of the composition this one is a part of.
    pub part_of_idbackorder: Option<BackorderId>,
    /// When the backorder was made.
    pub created_at: Option<DateTime>,
    /// Whether the product is purchased. A developer account always answered `false`.
    pub is_purchased: Option<bool>,
    /// When the product is expected, as Picqer sends it: its documentation calls this a date and
    /// time and shows a date. A developer account always answered `null`, so the form is not known.
    pub date_available: Option<String>,
}

/// Filters for listing backorders.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BackorderQuery {
    idorder: Option<OrderId>,
    idproduct: Option<ProductId>,
    idcustomer: Option<CustomerId>,
    since_id: Option<BackorderId>,
    since: Option<DateTime>,
    idfulfilment_customer: Option<FulfilmentCustomerId>,
}

impl BackorderQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the backorders of an order.
    pub fn idorder(mut self, idorder: OrderId) -> Self {
        self.idorder = Some(idorder);
        self
    }

    /// Lists the backorders of a product.
    pub fn idproduct(mut self, idproduct: ProductId) -> Self {
        self.idproduct = Some(idproduct);
        self
    }

    /// Lists the backorders of a customer.
    pub fn idcustomer(mut self, idcustomer: CustomerId) -> Self {
        self.idcustomer = Some(idcustomer);
        self
    }

    /// Lists the backorders from this id on: Picqer also lists the one with this id, although its
    /// documentation says a later one.
    pub fn since_id(mut self, id: BackorderId) -> Self {
        self.since_id = Some(id);
        self
    }

    /// Lists the backorders made after this moment.
    pub fn since(mut self, moment: DateTime) -> Self {
        self.since = Some(moment);
        self
    }

    /// Lists the backorders of a fulfilment customer, with Picqer Fulfilment.
    pub fn idfulfilment_customer(mut self, customer: FulfilmentCustomerId) -> Self {
        self.idfulfilment_customer = Some(customer);
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(id) = self.idorder {
            pairs.push(("idorder", id.to_string()));
        }
        if let Some(id) = self.idproduct {
            pairs.push(("idproduct", id.to_string()));
        }
        if let Some(id) = self.idcustomer {
            pairs.push(("idcustomer", id.to_string()));
        }
        if let Some(id) = self.since_id {
            pairs.push(("sinceid", id.to_string()));
        }
        if let Some(moment) = self.since {
            pairs.push(("sincedate", moment.to_string()));
        }
        if let Some(id) = self.idfulfilment_customer {
            pairs.push(("idfulfilment_customer", id.to_string()));
        }
        pairs
    }
}

pub(crate) fn list(query: &BackorderQuery, page: Page) -> Call<Vec<Backorder>> {
    let mut request = Request::new(Method::GET, segments(&[&"backorders"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: BackorderId) -> Call<Backorder> {
    Call::new(Request::new(Method::GET, segments(&[&"backorders", &id])))
}

pub(crate) fn for_order(id: OrderId) -> Call<Vec<Backorder>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"orders", &id, &"backorders"]),
    ))
}

pub(crate) fn delete(id: BackorderId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"backorders", &id]),
    ))
}

pub(crate) fn process() -> NoContent {
    NoContent::new(Request::new(
        Method::POST,
        segments(&[&"backorders", &"process"]),
    ))
}

macro_rules! backorders_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of backorders.
        pub $($async)? fn list(
            &self,
            query: &$crate::backorders::BackorderQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::backorders::Backorder>> {
            self.client.execute($crate::backorders::list(query, page)) $(.$await)?
        }

        /// Lists all backorders that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::backorders::BackorderQuery,
        ) -> $crate::error::Result<Vec<$crate::backorders::Backorder>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::backorders::list(query, page)) $(.$await)?)
        }

        /// Fetches a backorder.
        pub $($async)? fn get(
            &self,
            id: $crate::types::BackorderId,
        ) -> $crate::error::Result<$crate::backorders::Backorder> {
            self.client.execute($crate::backorders::get(id)) $(.$await)?
        }

        /// Lists the backorders of an order.
        pub $($async)? fn for_order(
            &self,
            order: $crate::types::OrderId,
        ) -> $crate::error::Result<Vec<$crate::backorders::Backorder>> {
            self.client.execute($crate::backorders::for_order(order)) $(.$await)?
        }

        /// Deletes a backorder, which also cancels the line of the order. An order whose lines are
        /// all cancelled is cancelled too.
        pub $($async)? fn delete(&self, id: $crate::types::BackorderId) -> $crate::error::Result<()> {
            self.client.execute($crate::backorders::delete(id)) $(.$await)?
        }

        /// Makes picklists of all backorders that have stock available. Picqer does not do this by
        /// itself. It answers an empty list, also when it made picklists, so look at the picklists
        /// or at the backorders afterwards. To do this for one order, use
        /// [`process_backorders`](crate::orders::Orders::process_backorders).
        pub $($async)? fn process(&self) -> $crate::error::Result<()> {
            self.client.execute($crate::backorders::process()) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use backorders_methods;

/// Operations on backorders.
#[derive(Debug, Clone, Copy)]
pub struct Backorders<'a> {
    pub(crate) client: &'a Client,
}

impl Backorders<'_> {
    backorders_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_become_query_pairs() {
        assert!(BackorderQuery::new().pairs().is_empty());
        let moment: DateTime = "2026-11-09 16:00:00".parse().unwrap();
        let query = BackorderQuery::new()
            .idorder(OrderId(2))
            .idproduct(ProductId(13))
            .idcustomer(CustomerId(5))
            .since_id(BackorderId(201992))
            .since(moment)
            .idfulfilment_customer(FulfilmentCustomerId(623));
        assert_eq!(
            query.pairs(),
            vec![
                ("idorder", "2".to_owned()),
                ("idproduct", "13".to_owned()),
                ("idcustomer", "5".to_owned()),
                ("sinceid", "201992".to_owned()),
                ("sincedate", "2026-11-09 16:00:00".to_owned()),
                ("idfulfilment_customer", "623".to_owned()),
            ]
        );
    }
}
