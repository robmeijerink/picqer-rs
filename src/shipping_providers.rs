//! Shipping providers.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::types::{ShippingProviderId, ShippingProviderProfileId};

/// A way of shipping with a shipping provider, such as a service level.
///
/// Picqer also reports whether a profile is the standard one and whether it is active, which the
/// documentation does not describe. Those are not read here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ShippingProfile {
    /// The profile.
    pub idshippingprovider_profile: ShippingProviderProfileId,
    /// The name of the profile.
    pub name: Option<String>,
}

/// A shipping provider an account is connected to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ShippingProvider {
    /// The shipping provider.
    pub idshippingprovider: ShippingProviderId,
    /// The name given to the provider.
    pub name: Option<String>,
    /// The kind of provider, such as `PostNL` or `Manual`.
    pub provider: Option<String>,
    /// Whether the provider is active.
    pub active: Option<bool>,
    /// The ways of shipping the provider offers.
    #[serde(default)]
    pub profiles: Vec<ShippingProfile>,
}

pub(crate) fn list(page: Page) -> Call<Vec<ShippingProvider>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"shippingproviders"]))))
}

macro_rules! shipping_providers_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of shipping providers.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::shipping_providers::ShippingProvider>> {
            self.client.execute($crate::shipping_providers::list(page)) $(.$await)?
        }

        /// Lists all shipping providers, following the pages.
        pub $($async)? fn list_all(
            &self,
        ) -> $crate::error::Result<Vec<$crate::shipping_providers::ShippingProvider>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::shipping_providers::list(page)) $(.$await)?)
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use shipping_providers_methods;

/// Operations on shipping providers.
#[derive(Debug, Clone, Copy)]
pub struct ShippingProviders<'a> {
    pub(crate) client: &'a Client,
}

impl ShippingProviders<'_> {
    shipping_providers_methods!(async; await);
}
