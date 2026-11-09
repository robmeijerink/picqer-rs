//! Users.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::types::{DateTime, PackingStationId, UserId};

/// Whether a user may do something.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct UserRight {
    /// The name of the right, such as `orders.edit`.
    pub right: String,
    /// Whether the user has it.
    pub granted: bool,
}

/// A user of the account.
///
/// Picqer also reports `full_name`, `created_at` and `updated_at`, which the documentation does
/// not describe. Those are not read here. The documentation only shows examples, so what is not an
/// identifier is optional.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct User {
    /// The user.
    pub iduser: UserId,
    /// The packing station the user works at.
    pub idpacking_station: Option<PackingStationId>,
    /// The username.
    pub username: Option<String>,
    /// The first name.
    pub firstname: Option<String>,
    /// The last name.
    pub lastname: Option<String>,
    /// The first name, under its newer name.
    pub first_name: Option<String>,
    /// The last name, under its newer name.
    pub last_name: Option<String>,
    /// The email address.
    pub emailaddress: Option<String>,
    /// The language of the user.
    pub language: Option<String>,
    /// Whether the user is an administrator.
    pub admin: Option<bool>,
    /// Whether the user can log in.
    pub active: Option<bool>,
    /// When the user last logged in.
    pub last_login_at: Option<DateTime>,
    /// The rights of the user. Picqer only reports them when a single user is fetched.
    #[serde(default)]
    pub rights: Vec<UserRight>,
}

/// Filters for listing users.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UserQuery {
    active_only: bool,
}

impl UserQuery {
    /// No filters: all users.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists only the users that can log in.
    pub fn active_only(mut self) -> Self {
        self.active_only = true;
        self
    }
}

pub(crate) fn list(query: UserQuery, page: Page) -> Call<Vec<User>> {
    let mut request = Request::new(Method::GET, segments(&[&"users"]));
    if query.active_only {
        request = request.query("active", "true");
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: UserId) -> Call<User> {
    Call::new(Request::new(Method::GET, segments(&[&"users", &id])))
}

pub(crate) fn me() -> Call<User> {
    Call::new(Request::new(Method::GET, segments(&[&"users", &"me"])))
}

macro_rules! users_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of users.
        pub $($async)? fn list(
            &self,
            query: $crate::users::UserQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::users::User>> {
            self.client.execute($crate::users::list(query, page)) $(.$await)?
        }

        /// Lists all users that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: $crate::users::UserQuery,
        ) -> $crate::error::Result<Vec<$crate::users::User>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::users::list(query, page)) $(.$await)?)
        }

        /// Fetches a user, with their rights.
        pub $($async)? fn get(
            &self,
            id: $crate::types::UserId,
        ) -> $crate::error::Result<$crate::users::User> {
            self.client.execute($crate::users::get(id)) $(.$await)?
        }

        /// Fetches the user the API key belongs to, with their rights.
        pub $($async)? fn me(&self) -> $crate::error::Result<$crate::users::User> {
            self.client.execute($crate::users::me()) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use users_methods;

/// Operations on users.
#[derive(Debug, Clone, Copy)]
pub struct Users<'a> {
    pub(crate) client: &'a Client,
}

impl Users<'_> {
    users_methods!(async; await);
}
