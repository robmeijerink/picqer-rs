//! Orders.

mod query;
mod request;
mod response;

pub use query::OrderQuery;
pub use request::{
    NewOrder, NewOrderLine, NewOrderProduct, OrderAddress, OrderAddressChanges, OrderChanges,
    OrderLineChanges,
};
pub use response::{Order, OrderFieldEntry, OrderLineStatus, OrderProduct, OrderStatus};

use reqwest::Method;
use serde_json::json;

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::order_fields::OrderFieldValue;
use crate::pagination::Page;
use crate::tags::Tag;
use crate::types::{OrderFieldId, OrderId, OrderProductId, PauseReason, TagId};

pub(crate) fn list(query: &OrderQuery, page: Page) -> Call<Vec<Order>> {
    let mut request = Request::new(Method::GET, segments(&[&"orders"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: OrderId) -> Call<Order> {
    Call::new(Request::new(Method::GET, segments(&[&"orders", &id])))
}

pub(crate) fn create(order: &NewOrder) -> Result<Call<Order>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"orders"])).json_body(&order.to_value())?,
    ))
}

pub(crate) fn update(id: OrderId, changes: &OrderChanges) -> Result<Call<Order>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"orders", &id])).json_body(&changes.to_value())?,
    ))
}

pub(crate) fn cancel(id: OrderId, force: bool) -> NoContent {
    let request = Request::new(Method::DELETE, segments(&[&"orders", &id]));
    NoContent::new(if force {
        request.query("force", "true")
    } else {
        request
    })
}

pub(crate) fn transition(id: OrderId, action: &'static str) -> Call<Order> {
    Call::new(Request::new(
        Method::POST,
        segments(&[&"orders", &id, &action]),
    ))
}

pub(crate) fn process_backorders(id: OrderId, prefer_bulk_locations: bool) -> Call<Order> {
    let request = Request::new(
        Method::POST,
        segments(&[&"orders", &id, &"process-backorders"]),
    );
    Call::new(if prefer_bulk_locations {
        request.query("prefer_bulk_locations", "true")
    } else {
        request
    })
}

pub(crate) fn pause(id: OrderId, reason: &PauseReason) -> Result<Call<Order>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"orders", &id, &"pause"]))
            .json_body(&json!({"reason": reason.as_str()}))?,
    ))
}

pub(crate) fn product_status(id: OrderId) -> Call<ProductStatus> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"orders", &id, &"productstatus"]),
    ))
}

#[derive(serde::Deserialize)]
pub(crate) struct ProductStatus {
    pub products: Vec<OrderLineStatus>,
}

pub(crate) fn lines(id: OrderId) -> Call<Vec<OrderProduct>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"orders", &id, &"products"]),
    ))
}

pub(crate) fn add_line(id: OrderId, line: &NewOrderLine) -> Result<Call<OrderProduct>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"orders", &id, &"products"])).json_body(line)?,
    ))
}

pub(crate) fn update_line(
    id: OrderId,
    line: OrderProductId,
    changes: &OrderLineChanges,
) -> Result<Call<OrderProduct>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"orders", &id, &"products", &line]))
            .json_body(changes)?,
    ))
}

pub(crate) fn remove_line(id: OrderId, line: OrderProductId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"orders", &id, &"products", &line]),
    ))
}

pub(crate) fn tags(id: OrderId, page: Page) -> Call<Vec<Tag>> {
    Call::new(page.apply(Request::new(
        Method::GET,
        segments(&[&"orders", &id, &"tags"]),
    )))
}

pub(crate) fn add_tag(id: OrderId, tag: TagId) -> Result<Call<Tag>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"orders", &id, &"tags"]))
            .json_body(&json!({"idtag": tag}))?,
    ))
}

pub(crate) fn remove_tag(id: OrderId, tag: TagId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"orders", &id, &"tags", &tag]),
    ))
}

pub(crate) fn fields(id: OrderId) -> Call<Vec<OrderFieldEntry>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"orders", &id, &"orderfields"]),
    ))
}

pub(crate) fn set_field(id: OrderId, value: &OrderFieldValue) -> Result<Call<OrderFieldEntry>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"orders", &id, &"orderfields", &value.field()]),
        )
        .json_body(&value.body())?,
    ))
}

pub(crate) fn remove_field(id: OrderId, field: OrderFieldId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"orders", &id, &"orderfields", &field]),
    ))
}

macro_rules! orders_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of orders, newest first.
        pub $($async)? fn list(
            &self,
            query: &$crate::orders::OrderQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::orders::Order>> {
            self.client.execute($crate::orders::list(query, page)) $(.$await)?
        }

        /// Lists all orders that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::orders::OrderQuery,
        ) -> $crate::error::Result<Vec<$crate::orders::Order>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::orders::list(query, page)) $(.$await)?)
        }

        /// Fetches an order.
        pub $($async)? fn get(
            &self,
            id: $crate::types::OrderId,
        ) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::get(id)) $(.$await)?
        }

        /// Finds an order by its number, such as `O2026-1599`.
        pub $($async)? fn find_by_number(
            &self,
            number: &$crate::types::OrderNumber,
        ) -> $crate::error::Result<Option<$crate::orders::Order>> {
            let query = $crate::orders::OrderQuery::new().orderid(number.clone());
            let found = self.client.execute($crate::orders::list(&query, $crate::pagination::Page::FIRST)) $(.$await)? ?;
            Ok(found.into_iter().next())
        }

        /// Creates an order. Picqer creates it as a concept.
        pub $($async)? fn create(
            &self,
            order: &$crate::orders::NewOrder,
        ) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::create(order)?) $(.$await)?
        }

        /// Changes an order and returns it.
        pub $($async)? fn update(
            &self,
            id: $crate::types::OrderId,
            changes: &$crate::orders::OrderChanges,
        ) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::update(id, changes)?) $(.$await)?
        }

        /// Cancels an order that is a concept or expected.
        ///
        /// Picqer does the work in the background, so the order can still show its old status for
        /// a moment. While it is still busy with an earlier request for the order, such as
        /// allocating stock, it refuses with `ErrorCode::OrderBeingProcessed`; try again a few
        /// seconds later.
        pub $($async)? fn cancel(&self, id: $crate::types::OrderId) -> $crate::error::Result<()> {
            self.client.execute($crate::orders::cancel(id, false)) $(.$await)?
        }

        /// Cancels an order whatever its status, taking its picklists away, even when they were
        /// already picked.
        pub $($async)? fn force_cancel(&self, id: $crate::types::OrderId) -> $crate::error::Result<()> {
            self.client.execute($crate::orders::cancel(id, true)) $(.$await)?
        }

        /// Processes a concept or expected order, which makes Picqer create picklists or
        /// backorders for it.
        ///
        /// Picqer does the work in the background after it answers, so the order that comes back
        /// still has its old status. Fetch it again a little later.
        pub $($async)? fn process(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "process")) $(.$await)?
        }

        /// Pauses an order that is being processed, with the reason why.
        pub $($async)? fn pause(
            &self,
            id: $crate::types::OrderId,
            reason: &$crate::types::PauseReason,
        ) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::pause(id, reason)?) $(.$await)?
        }

        /// Resumes a paused order.
        pub $($async)? fn resume(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "resume")) $(.$await)?
        }

        /// Turns an expected order back into a concept.
        pub $($async)? fn change_to_concept(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "change-to-concept")) $(.$await)?
        }

        /// Reopens an order that is being processed and has no picklists, which makes it a
        /// concept again.
        pub $($async)? fn reopen(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "reopen")) $(.$await)?
        }

        /// Undoes the cancellation of an order. It becomes a concept again, in the background.
        pub $($async)? fn undo_cancellation(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "undo-cancellation")) $(.$await)?
        }

        /// Lowers the free stock for a concept or expected order, so a shop only shows what is
        /// really available.
        pub $($async)? fn allocate(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "allocate")) $(.$await)?
        }

        /// Releases the stock that was allocated to an order.
        pub $($async)? fn deallocate(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "deallocate")) $(.$await)?
        }

        /// Lets the backorders of an order use the available stock first.
        pub $($async)? fn prioritise(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "prioritise")) $(.$await)?
        }

        /// Creates a picklist from the backorders of an order that can be fulfilled. With
        /// `prefer_bulk_locations` Picqer picks from bulk locations first.
        pub $($async)? fn process_backorders(
            &self,
            id: $crate::types::OrderId,
            prefer_bulk_locations: bool,
        ) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::process_backorders(id, prefer_bulk_locations)) $(.$await)?
        }

        /// Anonymizes an order that is older than seven days: the names and addresses are removed,
        /// and so are the customer and its addresses when the order has one. This cannot be undone.
        pub $($async)? fn anonymize(&self, id: $crate::types::OrderId) -> $crate::error::Result<$crate::orders::Order> {
            self.client.execute($crate::orders::transition(id, "anonymize")) $(.$await)?
        }

        /// Fetches how far each line of an order has come.
        pub $($async)? fn product_status(
            &self,
            id: $crate::types::OrderId,
        ) -> $crate::error::Result<Vec<$crate::orders::OrderLineStatus>> {
            Ok(self.client.execute($crate::orders::product_status(id)) $(.$await)? ?.products)
        }

        /// Lists the lines of an order.
        pub $($async)? fn lines(
            &self,
            id: $crate::types::OrderId,
        ) -> $crate::error::Result<Vec<$crate::orders::OrderProduct>> {
            self.client.execute($crate::orders::lines(id)) $(.$await)?
        }

        /// Adds a line to an order that is a concept.
        pub $($async)? fn add_line(
            &self,
            id: $crate::types::OrderId,
            line: &$crate::orders::NewOrderLine,
        ) -> $crate::error::Result<$crate::orders::OrderProduct> {
            self.client.execute($crate::orders::add_line(id, line)?) $(.$await)?
        }

        /// Changes a line of an order.
        pub $($async)? fn update_line(
            &self,
            id: $crate::types::OrderId,
            line: $crate::types::OrderProductId,
            changes: &$crate::orders::OrderLineChanges,
        ) -> $crate::error::Result<$crate::orders::OrderProduct> {
            self.client.execute($crate::orders::update_line(id, line, changes)?) $(.$await)?
        }

        /// Takes a line out of an order that is a concept.
        pub $($async)? fn remove_line(
            &self,
            id: $crate::types::OrderId,
            line: $crate::types::OrderProductId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::orders::remove_line(id, line)) $(.$await)?
        }

        /// Lists the order fields that have a value on an order.
        pub $($async)? fn fields(
            &self,
            id: $crate::types::OrderId,
        ) -> $crate::error::Result<Vec<$crate::orders::OrderFieldEntry>> {
            self.client.execute($crate::orders::fields(id)) $(.$await)?
        }

        /// Sets the value of an order field on an order. One field per call.
        pub $($async)? fn set_field(
            &self,
            id: $crate::types::OrderId,
            value: &$crate::order_fields::OrderFieldValue,
        ) -> $crate::error::Result<$crate::orders::OrderFieldEntry> {
            self.client.execute($crate::orders::set_field(id, value)?) $(.$await)?
        }

        /// Removes the value of an order field from an order.
        pub $($async)? fn remove_field(
            &self,
            id: $crate::types::OrderId,
            field: $crate::types::OrderFieldId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::orders::remove_field(id, field)) $(.$await)?
        }

        /// Lists one page of the tags of an order.
        pub $($async)? fn tags(
            &self,
            id: $crate::types::OrderId,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::tags::Tag>> {
            self.client.execute($crate::orders::tags(id, page)) $(.$await)?
        }

        /// Puts a tag on an order.
        pub $($async)? fn add_tag(
            &self,
            id: $crate::types::OrderId,
            tag: $crate::types::TagId,
        ) -> $crate::error::Result<$crate::tags::Tag> {
            self.client.execute($crate::orders::add_tag(id, tag)?) $(.$await)?
        }

        /// Takes a tag off an order.
        pub $($async)? fn remove_tag(
            &self,
            id: $crate::types::OrderId,
            tag: $crate::types::TagId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::orders::remove_tag(id, tag)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use orders_methods;

/// Operations on orders.
#[derive(Debug, Clone, Copy)]
pub struct Orders<'a> {
    pub(crate) client: &'a Client,
}

impl Orders<'_> {
    orders_methods!(async; await);
}
