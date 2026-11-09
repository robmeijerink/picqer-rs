//! Customers and their addresses.

mod request;
mod response;

pub use request::{
    AddressChanges, AddressName, CustomerChanges, CustomerFieldValue, CustomerName, CustomerNumber,
    DiscountPercentage, NewCustomer, NewCustomerAddress,
};
pub use response::{Customer, CustomerAddress, CustomerFieldEntry, Language};

use reqwest::Method;
use serde_json::json;

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::tags::Tag;
use crate::types::{CustomerAddressId, CustomerId, TagId};

/// Filters for listing customers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomerQuery {
    search: Option<String>,
    customerid: Option<CustomerNumber>,
}

impl CustomerQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Searches through the name, contact name, email address and number of the customers.
    pub fn search(mut self, text: impl Into<String>) -> Self {
        self.search = Some(text.into());
        self
    }

    /// Lists the customer with this number of your own.
    pub fn customerid(mut self, customerid: CustomerNumber) -> Self {
        self.customerid = Some(customerid);
        self
    }
}

pub(crate) fn list(query: &CustomerQuery, page: Page) -> Call<Vec<Customer>> {
    let mut request = Request::new(Method::GET, segments(&[&"customers"]));
    if let Some(search) = &query.search {
        request = request.query("search", search.as_str());
    }
    if let Some(customerid) = &query.customerid {
        request = request.query("customerid", customerid.as_str());
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: CustomerId) -> Call<Customer> {
    Call::new(Request::new(Method::GET, segments(&[&"customers", &id])))
}

pub(crate) fn create(customer: &NewCustomer) -> Result<Call<Customer>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"customers"])).json_body(customer)?,
    ))
}

pub(crate) fn update(id: CustomerId, changes: &CustomerChanges) -> Result<Call<Customer>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"customers", &id])).json_body(changes)?,
    ))
}

pub(crate) fn delete(id: CustomerId) -> NoContent {
    NoContent::new(Request::new(Method::DELETE, segments(&[&"customers", &id])))
}

pub(crate) fn addresses(id: CustomerId, page: Page) -> Call<Vec<CustomerAddress>> {
    Call::new(page.apply(Request::new(
        Method::GET,
        segments(&[&"customers", &id, &"addresses"]),
    )))
}

pub(crate) fn add_address(
    id: CustomerId,
    address: &NewCustomerAddress,
) -> Result<Call<CustomerAddress>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"customers", &id, &"addresses"]))
            .json_body(address)?,
    ))
}

pub(crate) fn update_address(
    id: CustomerId,
    address: CustomerAddressId,
    changes: &AddressChanges,
) -> Result<Call<CustomerAddress>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"customers", &id, &"addresses", &address]),
        )
        .json_body(changes)?,
    ))
}

pub(crate) fn remove_address(id: CustomerId, address: CustomerAddressId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"customers", &id, &"addresses", &address]),
    ))
}

pub(crate) fn tags(id: CustomerId) -> Call<Vec<Tag>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"customers", &id, &"tags"]),
    ))
}

pub(crate) fn add_tag(id: CustomerId, tag: TagId) -> Result<Call<Tag>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"customers", &id, &"tags"]))
            .json_body(&json!({"idtag": tag}))?,
    ))
}

pub(crate) fn remove_tag(id: CustomerId, tag: TagId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"customers", &id, &"tags", &tag]),
    ))
}

macro_rules! customers_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of customers.
        pub $($async)? fn list(
            &self,
            query: &$crate::customers::CustomerQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::customers::Customer>> {
            self.client.execute($crate::customers::list(query, page)) $(.$await)?
        }

        /// Lists all customers that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::customers::CustomerQuery,
        ) -> $crate::error::Result<Vec<$crate::customers::Customer>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::customers::list(query, page)) $(.$await)?)
        }

        /// Fetches a customer.
        pub $($async)? fn get(
            &self,
            id: $crate::types::CustomerId,
        ) -> $crate::error::Result<$crate::customers::Customer> {
            self.client.execute($crate::customers::get(id)) $(.$await)?
        }

        /// Finds a customer by the number of your own.
        pub $($async)? fn find_by_number(
            &self,
            number: &$crate::customers::CustomerNumber,
        ) -> $crate::error::Result<Option<$crate::customers::Customer>> {
            let query = $crate::customers::CustomerQuery::new().customerid(number.clone());
            let found = self.client.execute($crate::customers::list(&query, $crate::pagination::Page::FIRST)) $(.$await)? ?;
            Ok(found.into_iter().next())
        }

        /// Creates a customer.
        pub $($async)? fn create(
            &self,
            customer: &$crate::customers::NewCustomer,
        ) -> $crate::error::Result<$crate::customers::Customer> {
            self.client.execute($crate::customers::create(customer)?) $(.$await)?
        }

        /// Changes a customer.
        pub $($async)? fn update(
            &self,
            id: $crate::types::CustomerId,
            changes: &$crate::customers::CustomerChanges,
        ) -> $crate::error::Result<$crate::customers::Customer> {
            self.client.execute($crate::customers::update(id, changes)?) $(.$await)?
        }

        /// Deletes a customer. Picqer refuses when orders or returns are linked to the customer.
        pub $($async)? fn delete(&self, id: $crate::types::CustomerId) -> $crate::error::Result<()> {
            self.client.execute($crate::customers::delete(id)) $(.$await)?
        }

        /// Lists one page of the addresses of a customer.
        pub $($async)? fn addresses(
            &self,
            id: $crate::types::CustomerId,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::customers::CustomerAddress>> {
            self.client.execute($crate::customers::addresses(id, page)) $(.$await)?
        }

        /// Lists all the addresses of a customer, following the pages.
        pub $($async)? fn addresses_all(
            &self,
            id: $crate::types::CustomerId,
        ) -> $crate::error::Result<Vec<$crate::customers::CustomerAddress>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::customers::addresses(id, page)) $(.$await)?)
        }

        /// Adds an address to a customer.
        pub $($async)? fn add_address(
            &self,
            id: $crate::types::CustomerId,
            address: &$crate::customers::NewCustomerAddress,
        ) -> $crate::error::Result<$crate::customers::CustomerAddress> {
            self.client.execute($crate::customers::add_address(id, address)?) $(.$await)?
        }

        /// Changes an address of a customer.
        pub $($async)? fn update_address(
            &self,
            id: $crate::types::CustomerId,
            address: $crate::types::CustomerAddressId,
            changes: &$crate::customers::AddressChanges,
        ) -> $crate::error::Result<$crate::customers::CustomerAddress> {
            self.client.execute($crate::customers::update_address(id, address, changes)?) $(.$await)?
        }

        /// Deletes an address of a customer.
        pub $($async)? fn remove_address(
            &self,
            id: $crate::types::CustomerId,
            address: $crate::types::CustomerAddressId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::customers::remove_address(id, address)) $(.$await)?
        }

        /// Lists the tags of a customer.
        pub $($async)? fn tags(
            &self,
            id: $crate::types::CustomerId,
        ) -> $crate::error::Result<Vec<$crate::tags::Tag>> {
            self.client.execute($crate::customers::tags(id)) $(.$await)?
        }

        /// Puts a tag on a customer.
        pub $($async)? fn add_tag(
            &self,
            id: $crate::types::CustomerId,
            tag: $crate::types::TagId,
        ) -> $crate::error::Result<$crate::tags::Tag> {
            self.client.execute($crate::customers::add_tag(id, tag)?) $(.$await)?
        }

        /// Takes a tag off a customer.
        pub $($async)? fn remove_tag(
            &self,
            id: $crate::types::CustomerId,
            tag: $crate::types::TagId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::customers::remove_tag(id, tag)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use customers_methods;

/// Operations on customers.
#[derive(Debug, Clone, Copy)]
pub struct Customers<'a> {
    pub(crate) client: &'a Client,
}

impl Customers<'_> {
    customers_methods!(async; await);
}
