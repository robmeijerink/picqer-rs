//! Webhooks.
//!
//! With webhooks, Picqer sends a request to an address of yours when something happens in the account.
//! This module registers the subscriptions, which are called hooks, and helps with receiving what
//! Picqer sends: [`verify_signature`] checks that a delivery comes from Picqer and
//! [`WebhookPayload`] reads it.
//!
//! Picqer cannot delete a hook: [`Hooks::deactivate`] stops it and [`Hooks::reactivate`] starts it
//! again. A hook can only be made once for an event and an address, whatever its name, and it cannot
//! be changed.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::Method;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use sha2::Sha256;
use url::Url;
use zeroize::Zeroizing;

use crate::client::Client;
use crate::error::{Error, Result as ApiResult, ValidationError};
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{DateTime, HookId};

bounded_string! {
    /// Your own name for a hook.
    HookName, "hook name", 1, usize::MAX
}

string_enum! {
    /// An event that a webhook can subscribe to.
    pub enum WebhookEvent {
        /// Triggered when an order is created.
        OrdersCreated = "orders.created",
        /// Triggered when an order is allocated.
        OrdersAllocated = "orders.allocated",
        /// Triggered when you process an order.
        OrdersClosed = "orders.closed",
        /// Triggered when you pause an order.
        OrdersPaused = "orders.paused",
        /// Triggered when you resume an order.
        OrdersResumed = "orders.resumed",
        /// Triggered when the status of an order changes, e.g. from concept to processing or from completed to cancelled.
        OrdersStatusChanged = "orders.status_changed",
        /// Triggered when an order is completed, this is when all products are shipped to the customer.
        OrdersCompleted = "orders.completed",
        /// Triggered when a note is added to an order.
        OrdersNotesCreated = "orders.notes.created",
        /// Triggered when a picklist is created.
        PicklistsCreated = "picklists.created",
        /// Triggered when the metadata of a picklist changes. For example, changes in status, urgency, assignee, or when products are added to, cancelled/removed from or split off the picklist.
        PicklistsChanged = "picklists.changed",
        /// Triggered when you pause a picklist.
        PicklistsPaused = "picklists.paused",
        /// Triggered when you resume a picklist.
        PicklistsResumed = "picklists.resumed",
        /// Triggered when a picklist is closed.
        PicklistsClosed = "picklists.closed",
        /// Triggered when a picklist is cancelled.
        PicklistsCancelled = "picklists.cancelled",
        /// Triggered when a picklist is snoozed.
        PicklistsSnoozed = "picklists.snoozed",
        /// Triggered when a picklist is unsnoozed, either automatically or manually.
        PicklistsUnsnoozed = "picklists.unsnoozed",
        /// Triggered when a shipment is created on a picklist (can happen 0, 1 or multiple times per picklist).
        PicklistsShipmentsCreated = "picklists.shipments.created",
        /// Triggered when a shipment is cancelled on a picklist (can happen 0, 1 or multiple times per picklist).
        PicklistsShipmentsCancelled = "picklists.shipments.cancelled",
        /// Triggered when a picklist batch is created.
        PicklistBatchesCreated = "picklist_batches.created",
        /// Triggered when a picklist batch is changed (excluding pick actions).
        PicklistBatchesChanged = "picklist_batches.changed",
        /// Triggered when a picklist batch is completed.
        PicklistBatchesCompleted = "picklist_batches.completed",
        /// Triggered when a product is added.
        ProductsCreated = "products.created",
        /// Triggered when a product is added or edited.
        ProductsChanged = "products.changed",
        /// Triggered when the free stock of a normal product is changed.
        ProductsFreeStockChanged = "products.free_stock_changed",
        /// Triggered when the real stock of a normal product is changed. (Do not use this for tracking of free stock changes. Use free_stock_changed for that information.)
        ProductsStockChanged = "products.stock_changed",
        /// Triggered when the virtual stock of a virtual assembled product is changed.
        ProductsAssembledStockChanged = "products.assembled_stock_changed",
        /// Triggered when stock on a location is changed.
        ProductsStockOnLocationChanged = "products.stock_on_location_changed",
        /// Triggered when parts of a product change, like added or removed parts or changes to the amount of a part.
        ProductsPartsChanged = "products.parts.changed",
        /// Triggered when a purchase order is created.
        PurchaseOrdersCreated = "purchase_orders.created",
        /// Triggered when a purchase order is changed, like added or removed products.
        PurchaseOrdersChanged = "purchase_orders.changed",
        /// Triggered when a purchase order is marked as purchased.
        PurchaseOrdersPurchased = "purchase_orders.purchased",
        /// Deprecated, use [`ReceiptsCompleted`](Self::ReceiptsCompleted). Triggered when a purchase order receipt is created.
        PurchaseOrdersReceiptsCreated = "purchase_orders.receipts.created",
        /// Triggered when a receipt is created.
        ReceiptsCreated = "receipts.created",
        /// Triggered when a receipt is completed and all products added to the stock.
        ReceiptsCompleted = "receipts.completed",
        /// Triggered when a product for a v2 receipt is received.
        ReceiptsProductReceived = "receipts.product_received",
        /// Triggered when a product for a v2 receipt is reverted.
        ReceiptsProductReverted = "receipts.product_reverted",
        /// Triggered when a return is created.
        ReturnsCreated = "returns.created",
        /// Triggered when a return is edited, products or replacements are added.
        ReturnsChanged = "returns.changed",
        /// Triggered when the status of a return is changed, or when a comment/note is added.
        ReturnsStatusChanged = "returns.status_changed",
        /// Triggered when returned products for a return are received.
        ReturnsProductsReceived = "returns.products_received",
        /// Triggered when a new comment is added.
        CommentsCreated = "comments.created",
        /// Triggered when a movement is moved from one warehouse to another and the stock is changed.
        MovementsMoved = "movements.moved",
        /// Triggered when a location stock count is completed.
        LocationStockCountsCompleted = "location_stock_counts.completed",
        /// Triggered when a task is created.
        TasksCreated = "tasks.created",
        /// Triggered when a task is changed.
        TasksChanged = "tasks.changed",
        /// Triggered when a task is deleted.
        TasksDeleted = "tasks.deleted",
        /// Triggered when a task is completed.
        TasksCompleted = "tasks.completed",
        /// Triggered when a task is reopened.
        TasksUncompleted = "tasks.uncompleted",
        /// Triggered when an order is imported from a webshop integration.
        WebshopOrdersImported = "webshop_orders.imported",
    }
}

/// A subscription to an event.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Hook {
    /// The hook.
    pub idhook: HookId,
    /// Your own name for the hook.
    pub name: String,
    /// The event.
    pub event: Reported<WebhookEvent>,
    /// The address Picqer sends the event to.
    pub address: String,
    /// Whether Picqer sends the event. Picqer deactivates a hook that keeps failing, after five
    /// complete failures within 24 hours.
    pub active: Option<bool>,
    /// Whether the hook has a secret, which Picqer never shows.
    pub secret: Option<bool>,
}

/// A hook to create.
///
/// Picqer accepts an `http` address and an `https` address, and refuses anything else.
#[derive(Debug, Clone)]
pub struct NewHook {
    name: HookName,
    event: WebhookEvent,
    address: Url,
    secret: Option<WebhookSecret>,
}

impl NewHook {
    /// A hook that sends an event to an address.
    pub fn new(name: HookName, event: WebhookEvent, address: Url) -> Result<Self, ValidationError> {
        if !matches!(address.scheme(), "http" | "https") {
            return Err(ValidationError::new(
                "hook address",
                "must be an `http` or `https` address",
            ));
        }
        Ok(Self {
            name,
            event,
            address,
            secret: None,
        })
    }

    /// Signs every delivery with a secret, so [`verify_signature`] can check them.
    pub fn secret(mut self, secret: WebhookSecret) -> Self {
        self.secret = Some(secret);
        self
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut body = Map::new();
        body.insert("name".into(), json!(self.name));
        body.insert("event".into(), json!(self.event));
        body.insert("address".into(), json!(self.address.as_str()));
        if let Some(secret) = &self.secret {
            body.insert("secret".into(), json!(secret.0.as_str()));
        }
        Value::Object(body)
    }
}

/// What Picqer sends to the address of a hook.
///
/// The documentation describes the fields below. What `data` holds depends on the event, so it is
/// kept as it was sent: read it with [`data_as`](Self::data_as), for example as a
/// [`Product`](crate::products::Product) for the events of products. A developer account cannot
/// receive deliveries, so this follows the documentation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct WebhookPayload {
    /// The hook.
    pub idhook: HookId,
    /// Your own name for the hook.
    pub name: Option<String>,
    /// The event.
    pub event: Reported<WebhookEvent>,
    /// When the event happened.
    pub event_triggered_at: Option<DateTime>,
    /// All data about the object of the event.
    pub data: Value,
}

impl WebhookPayload {
    /// Reads the body of a delivery. Check its signature first, with the raw body.
    pub fn from_slice(body: &[u8]) -> ApiResult<Self> {
        serde_json::from_slice(body).map_err(|source| Error::Decode {
            source,
            body: crate::error::truncate(&String::from_utf8_lossy(body)),
        })
    }

    /// Reads `data` as one type.
    pub fn data_as<T: DeserializeOwned>(&self) -> ApiResult<T> {
        serde_json::from_value(self.data.clone()).map_err(|source| Error::Decode {
            source,
            body: crate::error::truncate(&self.data.to_string()),
        })
    }
}

pub(crate) fn list(page: Page) -> Call<Vec<Hook>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"hooks"]))))
}

pub(crate) fn get(id: HookId) -> Call<Hook> {
    Call::new(Request::new(Method::GET, segments(&[&"hooks", &id])))
}

pub(crate) fn create(hook: &NewHook) -> ApiResult<Call<Hook>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"hooks"])).json_body(&hook.to_value())?,
    ))
}

pub(crate) fn deactivate(id: HookId) -> NoContent {
    NoContent::new(Request::new(Method::DELETE, segments(&[&"hooks", &id])))
}

pub(crate) fn reactivate(id: HookId) -> Call<Hook> {
    Call::new(Request::new(
        Method::POST,
        segments(&[&"hooks", &id, &"reactivate"]),
    ))
}

macro_rules! hooks_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of hooks, the inactive ones too.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::webhooks::Hook>> {
            self.client.execute($crate::webhooks::list(page)) $(.$await)?
        }

        /// Lists all hooks, following the pages.
        pub $($async)? fn list_all(&self) -> $crate::error::Result<Vec<$crate::webhooks::Hook>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::webhooks::list(page)) $(.$await)?)
        }

        /// Fetches a hook, active or not.
        pub $($async)? fn get(
            &self,
            id: $crate::types::HookId,
        ) -> $crate::error::Result<$crate::webhooks::Hook> {
            self.client.execute($crate::webhooks::get(id)) $(.$await)?
        }

        /// Creates a hook. Picqer refuses one for an event and an address that have a hook already,
        /// whatever its name, with code 5.
        pub $($async)? fn create(
            &self,
            hook: &$crate::webhooks::NewHook,
        ) -> $crate::error::Result<$crate::webhooks::Hook> {
            self.client.execute($crate::webhooks::create(hook)?) $(.$await)?
        }

        /// Deactivates a hook. Picqer does not delete it, and it stays in the list.
        pub $($async)? fn deactivate(&self, id: $crate::types::HookId) -> $crate::error::Result<()> {
            self.client.execute($crate::webhooks::deactivate(id)) $(.$await)?
        }

        /// Activates a hook again. Picqer refuses one that is active with code 21.
        pub $($async)? fn reactivate(
            &self,
            id: $crate::types::HookId,
        ) -> $crate::error::Result<$crate::webhooks::Hook> {
            self.client.execute($crate::webhooks::reactivate(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use hooks_methods;

/// Operations on hooks.
#[derive(Debug, Clone, Copy)]
pub struct Hooks<'a> {
    pub(crate) client: &'a Client,
}

impl Hooks<'_> {
    hooks_methods!(async; await);
}

/// The name of the header that carries the signature of a webhook.
pub const SIGNATURE_HEADER: &str = "X-Picqer-Signature";

/// The secret of a webhook, used to verify that a delivery comes from Picqer.
///
/// It is wiped from memory on drop and never shown by `Debug`.
#[derive(Clone)]
pub struct WebhookSecret(Zeroizing<String>);

impl WebhookSecret {
    /// Wraps a secret, rejecting an empty one.
    pub fn new(secret: impl Into<String>) -> Result<Self, ValidationError> {
        let secret = Zeroizing::new(secret.into());
        if secret.is_empty() {
            return Err(ValidationError::new("webhook secret", "must not be empty"));
        }
        Ok(Self(secret))
    }
}

impl fmt::Debug for WebhookSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WebhookSecret([redacted])")
    }
}

/// Why a signature was not accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SignatureError {
    /// The signature is not base64.
    #[error("the signature is not valid base64")]
    Malformed,
    /// The signature does not belong to this body and secret.
    #[error("the signature does not match the body")]
    Mismatch,
}

/// Verifies the signature of a webhook delivery.
///
/// `body` must be the raw body exactly as it was received, before any parsing, and `signature` the
/// value of the [`SIGNATURE_HEADER`] header. The signature is the base64 encoded HMAC-SHA256 of
/// the body with the secret as key. The comparison takes the same time whatever the signature is.
pub fn verify_signature(
    secret: &WebhookSecret,
    body: &[u8],
    signature: &str,
) -> Result<(), SignatureError> {
    let expected = STANDARD
        .decode(signature.trim())
        .map_err(|_| SignatureError::Malformed)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.0.as_bytes())
        .expect("hmac accepts keys of any length");
    mac.update(body);
    mac.verify_slice(&expected)
        .map_err(|_| SignatureError::Mismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &[u8] = br#"{"idhook":1}"#;
    const SIGNATURE: &str = "lRq74hARLejUtG5be5LjZWDibr09/G4R42V1dza9Ri8=";

    fn secret(value: &str) -> WebhookSecret {
        WebhookSecret::new(value).unwrap()
    }

    #[test]
    fn a_matching_signature_is_accepted() {
        assert_eq!(verify_signature(&secret("secret"), BODY, SIGNATURE), Ok(()));
    }

    #[test]
    fn surrounding_whitespace_in_the_header_is_ignored() {
        let padded = format!(" {SIGNATURE}\r\n");
        assert_eq!(verify_signature(&secret("secret"), BODY, &padded), Ok(()));
    }

    #[test]
    fn another_body_or_secret_is_refused() {
        assert_eq!(
            verify_signature(&secret("secret"), br#"{"idhook":2}"#, SIGNATURE),
            Err(SignatureError::Mismatch)
        );
        assert_eq!(
            verify_signature(&secret("other"), BODY, SIGNATURE),
            Err(SignatureError::Mismatch)
        );
    }

    #[test]
    fn a_signature_that_is_not_base64_is_malformed() {
        assert_eq!(
            verify_signature(&secret("secret"), BODY, "not base64!"),
            Err(SignatureError::Malformed)
        );
    }

    #[test]
    fn a_signature_of_the_wrong_length_is_refused() {
        assert_eq!(
            verify_signature(&secret("secret"), BODY, "c2hvcnQ="),
            Err(SignatureError::Mismatch)
        );
    }

    #[test]
    fn the_secret_is_not_empty_and_not_shown() {
        assert!(WebhookSecret::new("").is_err());
        assert_eq!(
            format!("{:?}", secret("secret")),
            "WebhookSecret([redacted])"
        );
    }
}

#[cfg(test)]
mod hook_tests {
    use super::*;

    #[test]
    fn every_event_round_trips() {
        for event in WebhookEvent::ALL {
            assert_eq!(WebhookEvent::from_name(event.as_str()), Some(*event));
        }
        assert_eq!(WebhookEvent::ALL.len(), 49);
        assert_eq!(
            WebhookEvent::PicklistsShipmentsCreated.as_str(),
            "picklists.shipments.created"
        );
    }

    #[test]
    fn a_hook_needs_an_http_address() {
        let name = || HookName::new("Stock is changed").unwrap();
        let ok = NewHook::new(
            name(),
            WebhookEvent::ProductsFreeStockChanged,
            Url::parse("https://example.com/triggers/stock").unwrap(),
        )
        .unwrap();
        assert_eq!(
            ok.to_value(),
            json!({
                "name": "Stock is changed", "event": "products.free_stock_changed",
                "address": "https://example.com/triggers/stock"
            })
        );
        assert!(
            NewHook::new(
                name(),
                WebhookEvent::OrdersCreated,
                Url::parse("ftp://example.com/x").unwrap()
            )
            .is_err()
        );
        assert!(HookName::new("").is_err());
    }

    #[test]
    fn a_secret_is_sent_but_not_shown() {
        let hook = NewHook::new(
            HookName::new("Signed").unwrap(),
            WebhookEvent::OrdersCreated,
            Url::parse("http://example.com/x").unwrap(),
        )
        .unwrap()
        .secret(WebhookSecret::new("my-secret-key").unwrap());
        assert_eq!(hook.to_value()["secret"], "my-secret-key");
        assert!(!format!("{hook:?}").contains("my-secret-key"));
    }

    #[test]
    fn a_delivery_is_read_from_its_body() {
        let body = br#"{"idhook":228,"name":"Stock change notifications","event":"products.free_stock_changed","event_triggered_at":"2018-12-01 02:09:12","data":{"idproduct":138,"productcode":"6531-RB-7-9"}}"#;
        let payload = WebhookPayload::from_slice(body).unwrap();
        assert_eq!(payload.idhook, HookId(228));
        assert_eq!(
            payload.event,
            Reported::Known(WebhookEvent::ProductsFreeStockChanged)
        );
        assert_eq!(
            payload.event_triggered_at.unwrap().to_string(),
            "2018-12-01 02:09:12"
        );
        #[derive(Deserialize)]
        struct Partial {
            idproduct: i64,
        }
        assert_eq!(payload.data_as::<Partial>().unwrap().idproduct, 138);
        assert!(WebhookPayload::from_slice(b"not json").is_err());
    }

    #[test]
    fn an_event_that_is_not_known_is_kept() {
        let body = br#"{"idhook":1,"event":"orders.teleported","data":{}}"#;
        let payload = WebhookPayload::from_slice(body).unwrap();
        assert!(matches!(payload.event, Reported::Unknown(_)));
    }
}
