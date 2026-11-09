//! VAT groups.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::types::{Percentage, VatGroupId};

/// A VAT rate that products are linked to.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct VatGroup {
    /// The VAT group.
    pub idvatgroup: VatGroupId,
    /// The name of the VAT group.
    pub name: String,
    /// The rate of VAT.
    pub percentage: Percentage,
    /// Whether this is the standard VAT group. The attribute table of the documentation does not
    /// list this field, but the API reports it.
    pub standard: Option<bool>,
}

pub(crate) fn list(page: Page) -> Call<Vec<VatGroup>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"vatgroups"]))))
}

pub(crate) fn get(id: VatGroupId) -> Call<VatGroup> {
    Call::new(Request::new(Method::GET, segments(&[&"vatgroups", &id])))
}

macro_rules! vat_groups_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of VAT groups.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::vat_groups::VatGroup>> {
            self.client.execute($crate::vat_groups::list(page)) $(.$await)?
        }

        /// Lists all VAT groups, following the pages.
        pub $($async)? fn list_all(&self) -> $crate::error::Result<Vec<$crate::vat_groups::VatGroup>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::vat_groups::list(page)) $(.$await)?)
        }

        /// Fetches a VAT group.
        pub $($async)? fn get(
            &self,
            id: $crate::types::VatGroupId,
        ) -> $crate::error::Result<$crate::vat_groups::VatGroup> {
            self.client.execute($crate::vat_groups::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use vat_groups_methods;

/// Operations on VAT groups.
#[derive(Debug, Clone, Copy)]
pub struct VatGroups<'a> {
    pub(crate) client: &'a Client,
}

impl VatGroups<'_> {
    vat_groups_methods!(async; await);
}
