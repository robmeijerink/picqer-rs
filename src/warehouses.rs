//! Warehouses.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::stock::StockLevel;
use crate::types::{ProductId, WarehouseId};

/// A physical location where stock is held and orders are picked.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Warehouse {
    /// The warehouse.
    pub idwarehouse: WarehouseId,
    /// The name of the warehouse.
    pub name: String,
    /// Whether the products of the warehouse can be ordered, picked and shipped.
    pub accept_orders: bool,
    /// Whether the stock counts for the consolidated stock of the products.
    pub counts_for_general_stock: bool,
    /// The priority of the warehouse. A lower number gets more orders.
    pub priority: i64,
    /// Whether the warehouse is active.
    pub active: bool,
}

/// A product that has stock or reservations in a warehouse.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct WarehouseStock {
    /// The product.
    pub idproduct: ProductId,
    /// The code of the product.
    pub productcode: String,
    /// The stock of the product in the warehouse.
    pub stock: StockLevel,
}

pub(crate) fn list(page: Page) -> Call<Vec<Warehouse>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"warehouses"]))))
}

pub(crate) fn get(id: WarehouseId) -> Call<Warehouse> {
    Call::new(Request::new(Method::GET, segments(&[&"warehouses", &id])))
}

pub(crate) fn stock(id: WarehouseId, page: Page) -> Call<Vec<WarehouseStock>> {
    Call::new(page.apply(Request::new(
        Method::GET,
        segments(&[&"warehouses", &id, &"stock"]),
    )))
}

macro_rules! warehouses_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of warehouses.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::warehouses::Warehouse>> {
            self.client.execute($crate::warehouses::list(page)) $(.$await)?
        }

        /// Lists all warehouses, following the pages.
        pub $($async)? fn list_all(&self) -> $crate::error::Result<Vec<$crate::warehouses::Warehouse>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::warehouses::list(page)) $(.$await)?)
        }

        /// Fetches a warehouse.
        pub $($async)? fn get(
            &self,
            id: $crate::types::WarehouseId,
        ) -> $crate::error::Result<$crate::warehouses::Warehouse> {
            self.client.execute($crate::warehouses::get(id)) $(.$await)?
        }

        /// Lists one page of the products that have stock or reservations in a warehouse.
        pub $($async)? fn stock(
            &self,
            id: $crate::types::WarehouseId,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::warehouses::WarehouseStock>> {
            self.client.execute($crate::warehouses::stock(id, page)) $(.$await)?
        }

        /// Lists all the products that have stock or reservations in a warehouse, following the
        /// pages.
        pub $($async)? fn stock_all(
            &self,
            id: $crate::types::WarehouseId,
        ) -> $crate::error::Result<Vec<$crate::warehouses::WarehouseStock>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::warehouses::stock(id, page)) $(.$await)?)
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use warehouses_methods;

/// Operations on warehouses.
#[derive(Debug, Clone, Copy)]
pub struct Warehouses<'a> {
    pub(crate) client: &'a Client,
}

impl Warehouses<'_> {
    warehouses_methods!(async; await);
}
