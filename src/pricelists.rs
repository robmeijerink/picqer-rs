//! Pricelists.
//!
//! Picqer documents these as read-only: pricelists are made in the app, and prices per pricelist
//! are set on a product.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::types::{Money, PricelistId};

/// A named set of prices.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Pricelist {
    /// The pricelist.
    pub idpricelist: PricelistId,
    /// The name of the pricelist.
    pub name: String,
}

/// The price of a product in a pricelist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricelistPrice {
    /// The pricelist.
    pub idpricelist: PricelistId,
    /// The price, excluding VAT.
    pub price: Money,
}

pub(crate) fn list(page: Page) -> Call<Vec<Pricelist>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"pricelists"]))))
}

pub(crate) fn get(id: PricelistId) -> Call<Pricelist> {
    Call::new(Request::new(Method::GET, segments(&[&"pricelists", &id])))
}

macro_rules! pricelists_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of pricelists.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::pricelists::Pricelist>> {
            self.client.execute($crate::pricelists::list(page)) $(.$await)?
        }

        /// Lists all pricelists, following the pages.
        pub $($async)? fn list_all(&self) -> $crate::error::Result<Vec<$crate::pricelists::Pricelist>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::pricelists::list(page)) $(.$await)?)
        }

        /// Fetches a pricelist.
        pub $($async)? fn get(
            &self,
            id: $crate::types::PricelistId,
        ) -> $crate::error::Result<$crate::pricelists::Pricelist> {
            self.client.execute($crate::pricelists::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use pricelists_methods;

/// Operations on pricelists.
#[derive(Debug, Clone, Copy)]
pub struct Pricelists<'a> {
    pub(crate) client: &'a Client,
}

impl Pricelists<'_> {
    pricelists_methods!(async; await);
}
