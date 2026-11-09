//! Picklist batches.
//!
//! A batch collects several picklists in one run through the warehouse. The documentation describes
//! the fields and its examples show a few more, which are read here too. What a developer account
//! also reports but the documentation never shows, such as the weight and the dimensions of the
//! products in a batch, is not read.

use std::num::NonZeroU32;

use reqwest::Method;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, Download};
use crate::pagination::Page;
use crate::picklists::{Picklist, PicklistStatus};
use crate::products::ProductFieldEntry;
use crate::reported::Reported;
use crate::types::{
    DateTime, FulfilmentCustomerId, NonEmpty, PicklistBatchId, PicklistBatchPresetId, PicklistId,
    ProductId, UserId, WarehouseId,
};

string_enum! {
    /// The kind of a batch.
    pub enum BatchType {
        /// A batch of picklists with one product each.
        Singles = "singles",
        /// A batch of picklists with several products.
        Normal = "normal",
    }
}

string_enum! {
    /// Where a batch is in its life.
    pub enum BatchStatus {
        /// The batch is being picked.
        Open = "open",
        /// The batch is done.
        Completed = "completed",
    }
}

/// A user that a batch is assigned to or was completed by.
///
/// The documentation calls this an array, but Picqer sends an object, as in its example.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BatchUser {
    /// The user.
    pub iduser: UserId,
    /// The full name of the user.
    pub full_name: Option<String>,
    /// The username.
    pub username: Option<String>,
}

/// A product in a batch, added up over all picklists of the batch.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BatchProduct {
    /// The product.
    pub idproduct: ProductId,
    /// The name of the product.
    pub name: Option<String>,
    /// The product code.
    pub productcode: Option<String>,
    /// The product code of the supplier.
    pub productcode_supplier: Option<String>,
    /// Where the product is picked.
    pub stock_location: Option<String>,
    /// The URL of the image of the product.
    pub image: Option<String>,
    /// The barcodes of the product.
    #[serde(default)]
    pub barcodes: Vec<String>,
    /// The product fields that have a value.
    #[serde(default)]
    pub productfields: Vec<ProductFieldEntry>,
    /// How many of the product are in the batch.
    pub amount: Option<i64>,
    /// How many are picked.
    pub amount_picked: Option<i64>,
    /// How many are collected.
    pub amount_collected: Option<i64>,
}

/// A picklist in a batch, in short.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BatchPicklist {
    /// The picklist.
    pub idpicklist: PicklistId,
    /// The number of the picklist, such as `P2026-1370`.
    pub picklistid: Option<String>,
    /// Your own reference.
    pub reference: Option<String>,
    /// Where the picklist is in its life.
    pub status: Option<Reported<PicklistStatus>>,
    /// The letter that tells the picklist apart in the batch.
    pub alias: Option<String>,
    /// How many products the picklist has.
    pub total_products: Option<i64>,
    /// The name to deliver to.
    pub delivery_name: Option<String>,
    /// Whether the picklist has notes.
    pub has_notes: Option<bool>,
    /// Whether the customer left remarks.
    pub has_customer_remarks: Option<bool>,
    /// The remarks of the customer.
    pub customer_remarks: Option<String>,
    /// When the picklist was created.
    pub created_at: Option<DateTime>,
}

/// A picklist batch.
///
/// Only a single batch, one that was just made and one that was just changed come with their
/// `products` and `picklists`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PicklistBatch {
    /// The batch.
    pub idpicklist_batch: PicklistBatchId,
    /// The warehouse.
    pub idwarehouse: Option<WarehouseId>,
    /// The preset the batch was made from.
    pub idpicklist_batch_preset: Option<PicklistBatchPresetId>,
    /// The number of the batch within the account.
    #[serde(default, deserialize_with = "crate::serde_util::opt_text")]
    pub picklist_batchid: Option<String>,
    /// The title that was set, if any.
    pub title: Option<String>,
    /// The title as the Picqer app shows it: the title, or else the creation date and, with several
    /// warehouses, the name of the warehouse.
    pub display_title: Option<String>,
    /// The kind of batch.
    #[serde(rename = "type")]
    pub batch_type: Option<Reported<BatchType>>,
    /// Where the batch is in its life.
    pub status: Option<Reported<BatchStatus>>,
    /// The user the batch is assigned to.
    pub assigned_to: Option<BatchUser>,
    /// The user that completed the batch.
    pub completed_by: Option<BatchUser>,
    /// How many products are in the batch.
    pub total_products: Option<i64>,
    /// How many picklists are in the batch.
    pub total_picklists: Option<i64>,
    /// When the batch was completed.
    pub completed_at: Option<DateTime>,
    /// When the batch was created.
    pub created_at: Option<DateTime>,
    /// When the batch was last changed.
    pub updated_at: Option<DateTime>,
    /// The fulfilment customer, with Picqer Fulfilment.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// The products of the batch.
    #[serde(default)]
    pub products: Vec<BatchProduct>,
    /// The picklists of the batch.
    #[serde(default)]
    pub picklists: Vec<BatchPicklist>,
}

/// Filters for listing batches.
///
/// The documentation also lists a `products` filter. It changes nothing in the list: only a single
/// batch comes with its products.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BatchQuery {
    idwarehouse: Option<WarehouseId>,
    assigned_to: Option<UserId>,
    batch_type: Option<BatchType>,
    status: Option<BatchStatus>,
}

impl BatchQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the batches of a warehouse.
    pub fn idwarehouse(mut self, idwarehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(idwarehouse);
        self
    }

    /// Lists the batches assigned to a user.
    pub fn assigned_to(mut self, user: UserId) -> Self {
        self.assigned_to = Some(user);
        self
    }

    /// Lists the batches of one kind.
    pub fn batch_type(mut self, batch_type: BatchType) -> Self {
        self.batch_type = Some(batch_type);
        self
    }

    /// Lists the batches with this status.
    pub fn status(mut self, status: BatchStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(id) = self.idwarehouse {
            pairs.push(("idwarehouse", id.to_string()));
        }
        if let Some(user) = self.assigned_to {
            pairs.push(("assigned_to_iduser", user.to_string()));
        }
        if let Some(batch_type) = self.batch_type {
            pairs.push(("type", batch_type.as_str().to_owned()));
        }
        if let Some(status) = self.status {
            pairs.push(("status", status.as_str().to_owned()));
        }
        pairs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Source {
    Picklists(NonEmpty<PicklistId>),
    Preset {
        preset: PicklistBatchPresetId,
        preferred_number_of_picklists: Option<NonZeroU32>,
    },
}

/// A batch to create.
///
/// Either from specific picklists or from a preset. A batch cannot be deleted, and it is completed as
/// soon as it has no picklists left, so one that is made and emptied stays behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewBatch {
    source: Source,
}

impl NewBatch {
    /// A batch of specific picklists. They must be open and for the same warehouse.
    pub fn from_picklists(picklists: NonEmpty<PicklistId>) -> Self {
        Self {
            source: Source::Picklists(picklists),
        }
    }

    /// A batch made from a preset, which takes the picklists that match its filters.
    pub fn from_preset(preset: PicklistBatchPresetId) -> Self {
        Self {
            source: Source::Preset {
                preset,
                preferred_number_of_picklists: None,
            },
        }
    }

    /// Overrides the maximum number of picklists of the preset. Only a batch from a preset has it.
    pub fn preferred_number_of_picklists(mut self, number: NonZeroU32) -> Self {
        if let Source::Preset {
            preferred_number_of_picklists,
            ..
        } = &mut self.source
        {
            *preferred_number_of_picklists = Some(number);
        }
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        match &self.source {
            Source::Picklists(picklists) => json!({"idpicklists": picklists}),
            Source::Preset {
                preset,
                preferred_number_of_picklists: None,
            } => json!({"idpicklist_batch_preset": preset}),
            Source::Preset {
                preset,
                preferred_number_of_picklists: Some(number),
            } => json!({
                "idpicklist_batch_preset": preset,
                "preferred_number_of_picklists": number
            }),
        }
    }
}

/// What the PDF of a batch holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchPdf {
    /// The batch itself. A normal batch has another layout than a singles batch.
    Batch,
    /// The batch and the picklists in it.
    WithPicklists,
    /// The batch and the packinglists of the picklists in it.
    WithPackinglists,
}

pub(crate) fn list(query: &BatchQuery, page: Page) -> Call<Vec<PicklistBatch>> {
    let mut request = Request::new(Method::GET, segments(&[&"picklists", &"batches"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: PicklistBatchId) -> Call<PicklistBatch> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"picklists", &"batches", &id]),
    ))
}

pub(crate) fn create(batch: &NewBatch) -> Result<Call<PicklistBatch>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"picklists", &"batches"]))
            .json_body(&batch.to_value())?,
    ))
}

pub(crate) fn add_picklist(
    id: PicklistBatchId,
    picklist: PicklistId,
) -> Result<Call<PicklistBatch>> {
    Ok(Call::new(
        Request::new(
            Method::POST,
            segments(&[&"picklists", &"batches", &id, &"picklists"]),
        )
        .json_body(&json!({"idpicklist": picklist}))?,
    ))
}

pub(crate) fn remove_picklist(id: PicklistBatchId, picklist: PicklistId) -> Call<PicklistBatch> {
    Call::new(Request::new(
        Method::DELETE,
        segments(&[&"picklists", &"batches", &id, &"picklists", &picklist]),
    ))
}

pub(crate) fn assign(id: PicklistBatchId, user: Option<UserId>) -> Result<Call<PicklistBatch>> {
    Ok(Call::new(
        Request::new(
            Method::POST,
            segments(&[&"picklists", &"batches", &id, &"assign"]),
        )
        .json_body(&json!({"iduser": user}))?,
    ))
}

pub(crate) fn picklists_of_product(id: PicklistBatchId, product: ProductId) -> Call<Vec<Picklist>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[
            &"picklists",
            &"batches",
            &id,
            &"products",
            &product,
            &"picklists",
        ]),
    ))
}

pub(crate) fn pdf(id: PicklistBatchId, content: BatchPdf) -> Download {
    let request = Request::new(
        Method::GET,
        segments(&[&"picklists", &"batches", &id, &"pdf"]),
    );
    Download::new(match content {
        BatchPdf::Batch => request,
        BatchPdf::WithPicklists => request.query("includePicklists", "1"),
        BatchPdf::WithPackinglists => request.query("includePackinglists", "1"),
    })
}

macro_rules! picklist_batches_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of batches, newest first.
        pub $($async)? fn list(
            &self,
            query: &$crate::picklist_batches::BatchQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::picklist_batches::PicklistBatch>> {
            self.client.execute($crate::picklist_batches::list(query, page)) $(.$await)?
        }

        /// Lists all batches that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::picklist_batches::BatchQuery,
        ) -> $crate::error::Result<Vec<$crate::picklist_batches::PicklistBatch>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::picklist_batches::list(query, page)) $(.$await)?)
        }

        /// Fetches a batch with its products and picklists.
        pub $($async)? fn get(
            &self,
            id: $crate::types::PicklistBatchId,
        ) -> $crate::error::Result<$crate::picklist_batches::PicklistBatch> {
            self.client.execute($crate::picklist_batches::get(id)) $(.$await)?
        }

        /// Creates a batch from picklists or from a preset.
        ///
        /// Picqer refuses picklists that are not open or that are for another warehouse. A batch
        /// cannot be deleted, and it is completed as soon as it has no picklists left.
        pub $($async)? fn create(
            &self,
            batch: &$crate::picklist_batches::NewBatch,
        ) -> $crate::error::Result<$crate::picklist_batches::PicklistBatch> {
            self.client.execute($crate::picklist_batches::create(batch)?) $(.$await)?
        }

        /// Adds a picklist to a batch that is open and for the same warehouse. A picklist with
        /// more than one product only goes into a normal batch. A picklist that is in another
        /// batch is moved.
        pub $($async)? fn add_picklist(
            &self,
            id: $crate::types::PicklistBatchId,
            picklist: $crate::types::PicklistId,
        ) -> $crate::error::Result<$crate::picklist_batches::PicklistBatch> {
            self.client.execute($crate::picklist_batches::add_picklist(id, picklist)?) $(.$await)?
        }

        /// Takes a picklist out of a batch. The picklist itself stays. A batch that has no
        /// picklists left is completed.
        pub $($async)? fn remove_picklist(
            &self,
            id: $crate::types::PicklistBatchId,
            picklist: $crate::types::PicklistId,
        ) -> $crate::error::Result<$crate::picklist_batches::PicklistBatch> {
            self.client.execute($crate::picklist_batches::remove_picklist(id, picklist)) $(.$await)?
        }

        /// Assigns a batch, and all its picklists, to a user, or to nobody with `None`.
        pub $($async)? fn assign(
            &self,
            id: $crate::types::PicklistBatchId,
            user: Option<$crate::types::UserId>,
        ) -> $crate::error::Result<$crate::picklist_batches::PicklistBatch> {
            self.client.execute($crate::picklist_batches::assign(id, user)?) $(.$await)?
        }

        /// Lists the picklists in a batch that have a product.
        pub $($async)? fn picklists_of_product(
            &self,
            id: $crate::types::PicklistBatchId,
            product: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::picklists::Picklist>> {
            self.client.execute($crate::picklist_batches::picklists_of_product(id, product)) $(.$await)?
        }

        /// Downloads the PDF of a batch.
        pub $($async)? fn pdf(
            &self,
            id: $crate::types::PicklistBatchId,
            content: $crate::picklist_batches::BatchPdf,
        ) -> $crate::error::Result<Vec<u8>> {
            self.client.execute($crate::picklist_batches::pdf(id, content)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use picklist_batches_methods;

/// Operations on picklist batches.
#[derive(Debug, Clone, Copy)]
pub struct PicklistBatches<'a> {
    pub(crate) client: &'a Client,
}

impl PicklistBatches<'_> {
    picklist_batches_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_batch_from_picklists_sends_their_ids() {
        let batch = NewBatch::from_picklists(
            NonEmpty::new(vec![PicklistId(422), PicklistId(763)]).unwrap(),
        );
        assert_eq!(batch.to_value(), json!({"idpicklists": [422, 763]}));
    }

    #[test]
    fn a_batch_from_a_preset_sends_the_preset_and_the_number() {
        let batch = NewBatch::from_preset(PicklistBatchPresetId(5));
        assert_eq!(batch.to_value(), json!({"idpicklist_batch_preset": 5}));
        let batch = batch.preferred_number_of_picklists(NonZeroU32::new(3).unwrap());
        assert_eq!(
            batch.to_value(),
            json!({"idpicklist_batch_preset": 5, "preferred_number_of_picklists": 3})
        );
    }

    #[test]
    fn a_number_does_not_belong_to_a_batch_of_picklists() {
        let batch = NewBatch::from_picklists(NonEmpty::new(vec![PicklistId(1)]).unwrap())
            .preferred_number_of_picklists(NonZeroU32::new(3).unwrap());
        assert_eq!(batch.to_value(), json!({"idpicklists": [1]}));
    }

    #[test]
    fn filters_become_query_pairs() {
        assert!(BatchQuery::new().pairs().is_empty());
        let query = BatchQuery::new()
            .idwarehouse(WarehouseId(2))
            .assigned_to(UserId(3))
            .batch_type(BatchType::Normal)
            .status(BatchStatus::Open);
        assert_eq!(
            query.pairs(),
            vec![
                ("idwarehouse", "2".to_owned()),
                ("assigned_to_iduser", "3".to_owned()),
                ("type", "normal".to_owned()),
                ("status", "open".to_owned()),
            ]
        );
    }
}
