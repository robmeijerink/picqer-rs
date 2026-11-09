//! Purchase orders.

mod request;
mod response;

pub use request::{
    NewPurchaseOrder, NewPurchaseOrderProduct, PurchaseOrderChanges, PurchaseOrderLineChanges,
};
pub use response::{PurchaseOrder, PurchaseOrderProduct, PurchaseOrderStatus};

use reqwest::Method;

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::types::{
    DateTime, FulfilmentCustomerId, PurchaseOrderId, PurchaseOrderProductId, SupplierId,
    WarehouseId,
};

/// Filters for listing purchase orders.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurchaseOrderQuery {
    status: Option<PurchaseOrderStatus>,
    search: Option<String>,
    updated_after: Option<DateTime>,
    idfulfilment_customer: Option<FulfilmentCustomerId>,
    idsupplier: Option<SupplierId>,
    idwarehouse: Option<WarehouseId>,
}

impl PurchaseOrderQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the purchase orders with this status.
    pub fn status(mut self, status: PurchaseOrderStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Searches through the purchase orders, for instance by their number.
    pub fn search(mut self, text: impl Into<String>) -> Self {
        self.search = Some(text.into());
        self
    }

    /// Lists the purchase orders changed after this moment.
    pub fn updated_after(mut self, moment: DateTime) -> Self {
        self.updated_after = Some(moment);
        self
    }

    /// Lists the purchase orders of a fulfilment customer, with Picqer Fulfilment.
    pub fn idfulfilment_customer(mut self, customer: FulfilmentCustomerId) -> Self {
        self.idfulfilment_customer = Some(customer);
        self
    }

    /// Lists the purchase orders of a supplier.
    pub fn idsupplier(mut self, supplier: SupplierId) -> Self {
        self.idsupplier = Some(supplier);
        self
    }

    /// Lists the purchase orders for a warehouse.
    pub fn idwarehouse(mut self, warehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(warehouse);
        self
    }

    fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        let mut push = |name: &'static str, value: Option<String>| {
            if let Some(value) = value {
                pairs.push((name, value));
            }
        };
        push(
            "status",
            self.status.map(|status| status.as_str().to_owned()),
        );
        push("search", self.search.clone());
        push(
            "updated_after",
            self.updated_after.map(|moment| moment.to_string()),
        );
        push(
            "idfulfilment_customer",
            self.idfulfilment_customer.map(|id| id.to_string()),
        );
        push("idsupplier", self.idsupplier.map(|id| id.to_string()));
        push("idwarehouse", self.idwarehouse.map(|id| id.to_string()));
        pairs
    }
}

pub(crate) fn list(query: &PurchaseOrderQuery, page: Page) -> Call<Vec<PurchaseOrder>> {
    let mut request = Request::new(Method::GET, segments(&[&"purchaseorders"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: PurchaseOrderId) -> Call<PurchaseOrder> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"purchaseorders", &id]),
    ))
}

pub(crate) fn create(order: &NewPurchaseOrder) -> Result<Call<PurchaseOrder>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"purchaseorders"])).json_body(order)?,
    ))
}

pub(crate) fn update(
    id: PurchaseOrderId,
    changes: &PurchaseOrderChanges,
) -> Result<Call<PurchaseOrder>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"purchaseorders", &id])).json_body(changes)?,
    ))
}

pub(crate) fn action(id: PurchaseOrderId, action: &'static str) -> Call<PurchaseOrder> {
    Call::new(Request::new(
        Method::POST,
        segments(&[&"purchaseorders", &id, &action]),
    ))
}

pub(crate) fn lines(id: PurchaseOrderId) -> Call<Vec<PurchaseOrderProduct>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"purchaseorders", &id, &"products"]),
    ))
}

pub(crate) fn add_line(
    id: PurchaseOrderId,
    line: &NewPurchaseOrderProduct,
) -> Result<Call<PurchaseOrderProduct>> {
    Ok(Call::new(
        Request::new(
            Method::POST,
            segments(&[&"purchaseorders", &id, &"products"]),
        )
        .json_body(line)?,
    ))
}

pub(crate) fn update_line(
    id: PurchaseOrderId,
    line: PurchaseOrderProductId,
    changes: &PurchaseOrderLineChanges,
) -> Result<Call<PurchaseOrderProduct>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"purchaseorders", &id, &"products", &line]),
        )
        .json_body(changes)?,
    ))
}

pub(crate) fn remove_line(id: PurchaseOrderId, line: PurchaseOrderProductId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"purchaseorders", &id, &"products", &line]),
    ))
}

macro_rules! purchase_orders_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of purchase orders.
        pub $($async)? fn list(
            &self,
            query: &$crate::purchase_orders::PurchaseOrderQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::purchase_orders::PurchaseOrder>> {
            self.client.execute($crate::purchase_orders::list(query, page)) $(.$await)?
        }

        /// Lists all purchase orders that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::purchase_orders::PurchaseOrderQuery,
        ) -> $crate::error::Result<Vec<$crate::purchase_orders::PurchaseOrder>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::purchase_orders::list(query, page)) $(.$await)?)
        }

        /// Fetches a purchase order.
        pub $($async)? fn get(
            &self,
            id: $crate::types::PurchaseOrderId,
        ) -> $crate::error::Result<$crate::purchase_orders::PurchaseOrder> {
            self.client.execute($crate::purchase_orders::get(id)) $(.$await)?
        }

        /// Creates a purchase order. Picqer creates it as a concept.
        pub $($async)? fn create(
            &self,
            order: &$crate::purchase_orders::NewPurchaseOrder,
        ) -> $crate::error::Result<$crate::purchase_orders::PurchaseOrder> {
            self.client.execute($crate::purchase_orders::create(order)?) $(.$await)?
        }

        /// Changes a purchase order.
        pub $($async)? fn update(
            &self,
            id: $crate::types::PurchaseOrderId,
            changes: &$crate::purchase_orders::PurchaseOrderChanges,
        ) -> $crate::error::Result<$crate::purchase_orders::PurchaseOrder> {
            self.client.execute($crate::purchase_orders::update(id, changes)?) $(.$await)?
        }

        /// Marks a concept purchase order as purchased.
        pub $($async)? fn mark_as_purchased(
            &self,
            id: $crate::types::PurchaseOrderId,
        ) -> $crate::error::Result<$crate::purchase_orders::PurchaseOrder> {
            self.client.execute($crate::purchase_orders::action(id, "mark-as-purchased")) $(.$await)?
        }

        /// Closes a purchase order, to tell Picqer that the products that have not arrived will
        /// not arrive. It becomes `received`.
        pub $($async)? fn close(
            &self,
            id: $crate::types::PurchaseOrderId,
        ) -> $crate::error::Result<$crate::purchase_orders::PurchaseOrder> {
            self.client.execute($crate::purchase_orders::action(id, "close")) $(.$await)?
        }

        /// Cancels a purchase order that is a concept or purchased.
        pub $($async)? fn cancel(
            &self,
            id: $crate::types::PurchaseOrderId,
        ) -> $crate::error::Result<$crate::purchase_orders::PurchaseOrder> {
            self.client.execute($crate::purchase_orders::action(id, "cancel")) $(.$await)?
        }

        /// Lists the lines of a purchase order.
        pub $($async)? fn lines(
            &self,
            id: $crate::types::PurchaseOrderId,
        ) -> $crate::error::Result<Vec<$crate::purchase_orders::PurchaseOrderProduct>> {
            self.client.execute($crate::purchase_orders::lines(id)) $(.$await)?
        }

        /// Adds a line to a purchase order. A product can only be on it once.
        pub $($async)? fn add_line(
            &self,
            id: $crate::types::PurchaseOrderId,
            line: &$crate::purchase_orders::NewPurchaseOrderProduct,
        ) -> $crate::error::Result<$crate::purchase_orders::PurchaseOrderProduct> {
            self.client.execute($crate::purchase_orders::add_line(id, line)?) $(.$await)?
        }

        /// Changes a line of a purchase order.
        pub $($async)? fn update_line(
            &self,
            id: $crate::types::PurchaseOrderId,
            line: $crate::types::PurchaseOrderProductId,
            changes: &$crate::purchase_orders::PurchaseOrderLineChanges,
        ) -> $crate::error::Result<$crate::purchase_orders::PurchaseOrderProduct> {
            self.client.execute($crate::purchase_orders::update_line(id, line, changes)?) $(.$await)?
        }

        /// Takes a line out of a purchase order.
        pub $($async)? fn remove_line(
            &self,
            id: $crate::types::PurchaseOrderId,
            line: $crate::types::PurchaseOrderProductId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::purchase_orders::remove_line(id, line)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use purchase_orders_methods;

/// Operations on purchase orders.
#[derive(Debug, Clone, Copy)]
pub struct PurchaseOrders<'a> {
    pub(crate) client: &'a Client,
}

impl PurchaseOrders<'_> {
    purchase_orders_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_use_their_wire_names_and_formats() {
        let moment: DateTime = "2026-11-09 00:00:00".parse().unwrap();
        let query = PurchaseOrderQuery::new()
            .status(PurchaseOrderStatus::Purchased)
            .updated_after(moment)
            .idsupplier(SupplierId(4));
        assert_eq!(
            query.pairs(),
            vec![
                ("status", "purchased".to_owned()),
                ("updated_after", "2026-11-09 00:00:00".to_owned()),
                ("idsupplier", "4".to_owned()),
            ]
        );
        assert!(PurchaseOrderQuery::new().pairs().is_empty());
    }
}
