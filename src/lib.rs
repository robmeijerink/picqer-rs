//! Unofficial client for the [Picqer](https://picqer.com) API.
//!
//! Everything you send is validated when you construct it, so a value the API would refuse never
//! reaches the network. Values the API reports are kept as they are: an enum value this crate does
//! not know is returned as [`Reported::Unknown`] instead of failing the response.
//!
//! ```no_run
//! use picqer_rs::orders::{OrderQuery, OrderStatus};
//! use picqer_rs::pagination::Page;
//! use picqer_rs::{Client, UserAgent};
//!
//! # async fn run() -> picqer_rs::Result<()> {
//! let client = Client::from_env(UserAgent::new("My integration", "dev@example.com")?)?;
//!
//! let query = OrderQuery::new().status(OrderStatus::Processing);
//! for order in client.orders().list(&query, Page::FIRST).await? {
//!     println!("{} {}", order.orderid, order.deliveryname.as_deref().unwrap_or("-"));
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Making an order takes the same few steps:
//!
//! ```no_run
//! use std::num::NonZeroU32;
//!
//! use picqer_rs::orders::{NewOrder, NewOrderProduct};
//! use picqer_rs::types::{CustomerId, NonEmpty, ProductId};
//! use picqer_rs::Client;
//!
//! # async fn run(client: &Client) -> picqer_rs::Result<()> {
//! let line = NewOrderProduct::by_id(ProductId(1234), NonZeroU32::new(2).unwrap());
//! let order = NewOrder::for_customer(CustomerId(5678), NonEmpty::new(vec![line])?)
//!     .reference("webshop-1001");
//!
//! let created = client.orders().create(&order).await?;
//! println!("{} is {:?}", created.orderid, created.status);
//!
//! client.orders().process(created.idorder).await?;
//! let order = client.orders().get(created.idorder).await?;
//! println!("{} is now {:?}", order.orderid, order.status);
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - `rustls-tls` (default): TLS through rustls.
//! - `native-tls`: TLS through the platform library.
//! - `blocking`: the [`blocking`] client with the same operations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[macro_use]
mod macros;

mod auth;
mod client;
mod error;
mod http;
mod ops;
mod reported;
mod serde_util;

#[cfg(feature = "blocking")]
pub mod blocking;

pub mod backorders;
pub mod comments;
pub mod country;
pub mod customer_fields;
pub mod customers;
pub mod fields;
pub mod fulfilment;
pub mod location_stock_history;
pub mod location_types;
pub mod locations;
pub mod order_fields;
pub mod orders;
pub mod packagings;
pub mod packing_stations;
pub mod pagination;
pub mod picking_containers;
pub mod picklist_batch_presets;
pub mod picklist_batches;
pub mod picklists;
pub mod pricelists;
pub mod product_fields;
pub mod products;
pub mod purchase_orders;
pub mod rate_limit;
pub mod receipts;
pub mod returns;
pub mod shipments;
pub mod shipping_integration;
pub mod shipping_providers;
pub mod stats;
pub mod stock;
pub mod stock_history;
pub mod suppliers;
pub mod tags;
pub mod tasks;
pub mod templates;
pub mod types;
pub mod users;
pub mod vat_groups;
pub mod warehouses;
pub mod webhooks;
pub mod webshop_orders;

pub use auth::ApiKey;
pub use client::{Client, ClientBuilder};
pub use error::{ApiError, Error, ErrorCode, Result, ValidationError};
pub use reported::Reported;
pub use types::{Subdomain, UserAgent};
