//! Stats.
//!
//! A few numbers about the account, such as the number of open orders, to show on a dashboard.

use std::collections::BTreeMap;

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::reported::Reported;

string_enum! {
    /// A number Picqer keeps about the account.
    pub enum StatKey {
        /// The number of open picklists.
        OpenPicklists = "open-picklists",
        /// The number of open orders.
        OpenOrders = "open-orders",
        /// The number of current backorders.
        Backorders = "backorders",
        /// The number of orders made today.
        NewOrdersToday = "new-orders-today",
        /// The number of orders made this week.
        NewOrdersThisWeek = "new-orders-this-week",
        /// The number of picklists closed today.
        ClosedPicklistsToday = "closed-picklists-today",
        /// The number of picklists closed this week.
        ClosedPicklistsThisWeek = "closed-picklists-this-week",
        /// The number of customers made this week.
        NewCustomersThisWeek = "new-customers-this-week",
        /// The total number of orders.
        TotalOrders = "total-orders",
        /// The number of products.
        TotalProducts = "total-products",
        /// The number of active products.
        ActiveProducts = "active-products",
        /// The number of inactive products.
        InactiveProducts = "inactive-products",
    }
}

/// What a stat tells.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct StatDescription {
    /// The description.
    pub description: String,
}

#[derive(Deserialize)]
pub(crate) struct StatValue {
    pub value: i64,
}

pub(crate) fn list() -> Call<BTreeMap<String, StatDescription>> {
    Call::new(Request::new(Method::GET, segments(&[&"stats"])))
}

pub(crate) fn get(key: &str) -> Call<StatValue> {
    Call::new(Request::new(Method::GET, segments(&[&"stats", &key])))
}

macro_rules! stats_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists the stats that exist, with what they tell. A key that this crate does not know
        /// comes back as `Reported::Unknown`.
        pub $($async)? fn list(
            &self,
        ) -> $crate::error::Result<Vec<($crate::reported::Reported<$crate::stats::StatKey>, $crate::stats::StatDescription)>> {
            let stats = self.client.execute($crate::stats::list()) $(.$await)? ?;
            Ok(stats
                .into_iter()
                .map(|(key, description)| ($crate::stats::reported_key(&key), description))
                .collect())
        }

        /// Fetches the value of a stat.
        pub $($async)? fn get(&self, key: $crate::stats::StatKey) -> $crate::error::Result<i64> {
            Ok(self.client.execute($crate::stats::get(key.as_str())) $(.$await)? ?.value)
        }

        /// Fetches the value of a stat by its key, for one that [`StatKey`](crate::stats::StatKey)
        /// does not have. Picqer answers a key it does not know with not found.
        pub $($async)? fn get_by_key(&self, key: &str) -> $crate::error::Result<i64> {
            Ok(self.client.execute($crate::stats::get(key)) $(.$await)? ?.value)
        }
    };
}

pub(crate) fn reported_key(key: &str) -> Reported<StatKey> {
    match StatKey::from_name(key) {
        Some(known) => Reported::Known(known),
        None => Reported::Unknown(serde_json::Value::String(key.to_owned())),
    }
}

#[cfg(feature = "blocking")]
pub(crate) use stats_methods;

/// Operations on stats.
#[derive(Debug, Clone, Copy)]
pub struct Stats<'a> {
    pub(crate) client: &'a Client,
}

impl Stats<'_> {
    stats_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_round_trips() {
        for key in StatKey::ALL {
            assert_eq!(StatKey::from_name(key.as_str()), Some(*key));
        }
        assert_eq!(StatKey::ALL.len(), 12);
    }

    #[test]
    fn a_key_that_is_not_known_is_kept() {
        assert!(matches!(
            reported_key("open-tasks"),
            Reported::Unknown(serde_json::Value::String(text)) if text == "open-tasks"
        ));
        assert_eq!(
            reported_key("open-orders"),
            Reported::Known(StatKey::OpenOrders)
        );
    }
}
