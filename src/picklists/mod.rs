//! Picklists.

mod query;
mod request;
mod response;

pub use query::PicklistQuery;
pub use request::{
    CancelProduct, Pick, PickLocationAmount, PickTarget, PicklistChanges, Split, SplitLine,
};
pub use response::{
    AdviceSource, PackagingAdvice, PickLocation, PickedAt, Picklist, PicklistProduct,
    PicklistStatus,
};

use reqwest::Method;
use serde_json::json;

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::types::{
    DateTime, NonEmpty, PackagingId, PauseReason, PicklistId, PicklistProductId, UserId,
};

pub(crate) fn list(query: &PicklistQuery, page: Page) -> Call<Vec<Picklist>> {
    let mut request = Request::new(Method::GET, segments(&[&"picklists"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: PicklistId) -> Call<Picklist> {
    Call::new(Request::new(Method::GET, segments(&[&"picklists", &id])))
}

pub(crate) fn update(id: PicklistId, changes: &PicklistChanges) -> Result<Call<Picklist>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"picklists", &id])).json_body(changes)?,
    ))
}

pub(crate) fn action(id: PicklistId, action: &'static str) -> Call<Picklist> {
    Call::new(Request::new(
        Method::POST,
        segments(&[&"picklists", &id, &action]),
    ))
}

pub(crate) fn cancel(id: PicklistId) -> NoContent {
    NoContent::new(Request::new(
        Method::POST,
        segments(&[&"picklists", &id, &"cancel"]),
    ))
}

pub(crate) fn pick(id: PicklistId, pick: &Pick) -> Result<Call<Picklist>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"picklists", &id, &"pick"]))
            .json_body(&pick.to_value())?,
    ))
}

pub(crate) fn set_pick_locations(
    id: PicklistId,
    line: PicklistProductId,
    locations: &NonEmpty<PickLocationAmount>,
) -> Result<Call<Vec<PickedAt>>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"picklists", &id, &"products", &line, &"pick_locations"]),
        )
        .json_body(locations)?,
    ))
}

pub(crate) fn assign(id: PicklistId, user: UserId) -> Result<Call<Picklist>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"picklists", &id, &"assign"]))
            .json_body(&json!({"iduser": user}))?,
    ))
}

pub(crate) fn snooze(id: PicklistId, until: Option<DateTime>) -> Result<Call<Picklist>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"picklists", &id, &"snooze"]))
            .json_body(&json!({"snooze_until": until.map(|moment| moment.to_string())}))?,
    ))
}

pub(crate) fn pause(id: PicklistId, reason: &PauseReason) -> Result<Call<Picklist>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"picklists", &id, &"pause"]))
            .json_body(&json!({"reason": reason.as_str()}))?,
    ))
}

pub(crate) fn cancel_product(id: PicklistId, product: &CancelProduct) -> Result<Call<Picklist>> {
    Ok(Call::new(
        Request::new(
            Method::POST,
            segments(&[&"picklists", &id, &"cancelproduct"]),
        )
        .json_body(&product.to_value())?,
    ))
}

pub(crate) fn split(id: PicklistId, split: &Split) -> Result<Call<Picklist>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"picklists", &id, &"split"])).json_body(split)?,
    ))
}

pub(crate) fn packaging_advice(id: PicklistId) -> Call<PackagingAdvice> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"picklists", &id, &"packaging-advice"]),
    ))
}

pub(crate) fn set_packaging_advice(
    id: PicklistId,
    packaging: PackagingId,
) -> Result<Call<PackagingAdvice>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"picklists", &id, &"packaging-advice"]),
        )
        .json_body(&json!({"idpackaging": packaging}))?,
    ))
}

pub(crate) fn pdf(id: PicklistId, document: &'static str) -> Call<response::PdfBody> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"picklists", &id, &document]),
    ))
}

macro_rules! picklists_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of picklists.
        pub $($async)? fn list(
            &self,
            query: &$crate::picklists::PicklistQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::picklists::Picklist>> {
            self.client.execute($crate::picklists::list(query, page)) $(.$await)?
        }

        /// Lists all picklists that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::picklists::PicklistQuery,
        ) -> $crate::error::Result<Vec<$crate::picklists::Picklist>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::picklists::list(query, page)) $(.$await)?)
        }

        /// Fetches a picklist.
        pub $($async)? fn get(
            &self,
            id: $crate::types::PicklistId,
        ) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::get(id)) $(.$await)?
        }

        /// Finds a picklist by its number, such as `P2026-1370`.
        pub $($async)? fn find_by_number(
            &self,
            number: &$crate::types::PicklistNumber,
        ) -> $crate::error::Result<Option<$crate::picklists::Picklist>> {
            let query = $crate::picklists::PicklistQuery::new().picklistid(number.clone());
            let found = self.client.execute($crate::picklists::list(&query, $crate::pagination::Page::FIRST)) $(.$await)? ?;
            Ok(found.into_iter().next())
        }

        /// Changes a picklist and returns it.
        pub $($async)? fn update(
            &self,
            id: $crate::types::PicklistId,
            changes: &$crate::picklists::PicklistChanges,
        ) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::update(id, changes)?) $(.$await)?
        }

        /// Closes a picklist whose products are all picked, without creating a shipment.
        pub $($async)? fn close(&self, id: $crate::types::PicklistId) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::action(id, "close")) $(.$await)?
        }

        /// Marks every product of a picklist as picked.
        pub $($async)? fn pick_all(&self, id: $crate::types::PicklistId) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::action(id, "pickall")) $(.$await)?
        }

        /// Picks an amount of a product, or undoes it.
        pub $($async)? fn pick(
            &self,
            id: $crate::types::PicklistId,
            pick: &$crate::picklists::Pick,
        ) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::pick(id, pick)?) $(.$await)?
        }

        /// Changes where a line of a picklist is picked.
        pub $($async)? fn set_pick_locations(
            &self,
            id: $crate::types::PicklistId,
            line: $crate::types::PicklistProductId,
            locations: &$crate::types::NonEmpty<$crate::picklists::PickLocationAmount>,
        ) -> $crate::error::Result<Vec<$crate::picklists::PickedAt>> {
            self.client.execute($crate::picklists::set_pick_locations(id, line, locations)?) $(.$await)?
        }

        /// Assigns a user to a picklist.
        pub $($async)? fn assign(
            &self,
            id: $crate::types::PicklistId,
            user: $crate::types::UserId,
        ) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::assign(id, user)?) $(.$await)?
        }

        /// Takes the user off a picklist.
        pub $($async)? fn unassign(&self, id: $crate::types::PicklistId) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::action(id, "unassign")) $(.$await)?
        }

        /// Hides a picklist until a moment in the future. Picqer refuses a moment in the past.
        pub $($async)? fn snooze(
            &self,
            id: $crate::types::PicklistId,
            until: $crate::types::DateTime,
        ) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::snooze(id, Some(until))?) $(.$await)?
        }

        /// Brings a snoozed picklist back.
        pub $($async)? fn unsnooze(&self, id: $crate::types::PicklistId) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::snooze(id, None)?) $(.$await)?
        }

        /// Pauses a picklist that is new, with the reason why.
        pub $($async)? fn pause(
            &self,
            id: $crate::types::PicklistId,
            reason: &$crate::types::PauseReason,
        ) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::pause(id, reason)?) $(.$await)?
        }

        /// Resumes a paused picklist, when its order is not paused.
        pub $($async)? fn resume(&self, id: $crate::types::PicklistId) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::action(id, "resume")) $(.$await)?
        }

        /// Cancels an amount of a product from a picklist that is not closed. The amount becomes a
        /// backorder, and cancelling the last products cancels the picklist.
        pub $($async)? fn cancel_product(
            &self,
            id: $crate::types::PicklistId,
            product: &$crate::picklists::CancelProduct,
        ) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::cancel_product(id, product)?) $(.$await)?
        }

        /// Moves lines to a new picklist of the same order and returns the new picklist.
        pub $($async)? fn split(
            &self,
            id: $crate::types::PicklistId,
            split: &$crate::picklists::Split,
        ) -> $crate::error::Result<$crate::picklists::Picklist> {
            self.client.execute($crate::picklists::split(id, split)?) $(.$await)?
        }

        /// Cancels a picklist, which turns its products into backorders. Cancelling a closed
        /// picklist puts its products back in stock and removes its shipments.
        pub $($async)? fn cancel(&self, id: $crate::types::PicklistId) -> $crate::error::Result<()> {
            self.client.execute($crate::picklists::cancel(id)) $(.$await)?
        }

        /// Fetches the packaging Picqer advises for a picklist.
        pub $($async)? fn packaging_advice(
            &self,
            id: $crate::types::PicklistId,
        ) -> $crate::error::Result<$crate::picklists::PackagingAdvice> {
            self.client.execute($crate::picklists::packaging_advice(id)) $(.$await)?
        }

        /// Sets the packaging advice of a picklist.
        pub $($async)? fn set_packaging_advice(
            &self,
            id: $crate::types::PicklistId,
            packaging: $crate::types::PackagingId,
        ) -> $crate::error::Result<$crate::picklists::PackagingAdvice> {
            self.client.execute($crate::picklists::set_packaging_advice(id, packaging)?) $(.$await)?
        }

        /// Fetches the picklist as a PDF document.
        pub $($async)? fn picklist_pdf(&self, id: $crate::types::PicklistId) -> $crate::error::Result<Vec<u8>> {
            Ok(self.client.execute($crate::picklists::pdf(id, "picklistpdf")) $(.$await)? ?.pdf)
        }

        /// Fetches the packing list of a picklist as a PDF document.
        pub $($async)? fn packinglist_pdf(&self, id: $crate::types::PicklistId) -> $crate::error::Result<Vec<u8>> {
            Ok(self.client.execute($crate::picklists::pdf(id, "packinglistpdf")) $(.$await)? ?.pdf)
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use picklists_methods;

/// Operations on picklists.
#[derive(Debug, Clone, Copy)]
pub struct Picklists<'a> {
    pub(crate) client: &'a Client,
}

impl Picklists<'_> {
    picklists_methods!(async; await);
}
