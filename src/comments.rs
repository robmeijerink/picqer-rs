//! Comments on orders, products, customers and other resources.

use reqwest::Method;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::types::{
    CommentId, CustomerId, DateTime, FulfilmentCustomerId, OrderId, PicklistBatchId, PicklistId,
    ProductId, PurchaseOrderId, ReceiptId, ReturnId, SupplierId, UserId,
};

bounded_string! {
    /// The text of a comment. Picqer refuses a comment without text, and one of only whitespace.
    CommentBody, "comment body", 1, usize::MAX,
    |value| value.trim().is_empty().then_some("must not be blank")
}

/// A user, as the author of a comment or as mentioned in one.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CommentUser {
    /// The user.
    pub iduser: UserId,
    /// The username.
    pub username: String,
    /// The full name.
    pub full_name: String,
    /// Where the avatar of the user can be downloaded.
    pub image_url: String,
}

/// A fulfilment customer, as the author of a comment or as mentioned in one.
///
/// The developer account this crate was checked against has no comments by fulfilment customers,
/// so this follows the documentation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CommentFulfilmentCustomer {
    /// The fulfilment customer.
    pub idfulfilment_customer: FulfilmentCustomerId,
    /// The username.
    pub username: String,
    /// The name.
    pub name: String,
}

/// Who wrote a comment, or who is mentioned in one.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Person {
    /// A user.
    User(CommentUser),
    /// A fulfilment customer.
    FulfilmentCustomer(CommentFulfilmentCustomer),
    /// A kind of person this crate does not know, as Picqer sent it.
    Other {
        /// The type Picqer reported.
        kind: String,
        /// The person as Picqer sent it.
        raw: Value,
    },
}

impl Person {
    fn from_parts(kind: String, raw: Value) -> Result<Self, serde_json::Error> {
        match kind.as_str() {
            "user" => serde_json::from_value(raw).map(Self::User),
            "fulfilment_customer" => serde_json::from_value(raw).map(Self::FulfilmentCustomer),
            _ => Ok(Self::Other { kind, raw }),
        }
    }
}

/// A user or fulfilment customer that is mentioned in a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Mention {
    /// The text of the comment that caused the mention, such as `@Alice Example`.
    pub text: String,
    /// Who is mentioned.
    pub mentioned: Person,
}

/// What a comment was added to.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommentSource {
    /// An order.
    Order {
        /// The order.
        idorder: OrderId,
        /// The number of the order.
        orderid: String,
    },
    /// A return.
    Return {
        /// The return.
        idreturn: ReturnId,
        /// The number of the return.
        returnid: String,
    },
    /// A picklist.
    Picklist {
        /// The picklist.
        idpicklist: PicklistId,
        /// The number of the picklist.
        picklistid: String,
    },
    /// A picklist batch.
    PicklistBatch {
        /// The batch.
        idpicklist_batch: PicklistBatchId,
        /// The number of the batch.
        picklist_batchid: String,
    },
    /// A supplier.
    Supplier {
        /// The supplier.
        idsupplier: SupplierId,
        /// The name of the supplier.
        name: String,
    },
    /// A purchase order.
    PurchaseOrder {
        /// The purchase order.
        idpurchaseorder: PurchaseOrderId,
        /// The number of the purchase order.
        purchaseorderid: String,
    },
    /// A receipt.
    Receipt {
        /// The receipt.
        idreceipt: ReceiptId,
        /// The number of the receipt.
        receiptid: String,
    },
    /// A product.
    Product {
        /// The product.
        idproduct: ProductId,
        /// The code of the product.
        productcode: String,
        /// The name of the product.
        name: String,
    },
    /// A customer.
    Customer {
        /// The customer.
        idcustomer: CustomerId,
        /// Your own number for the customer, when it has one.
        customerid: Option<String>,
        /// The name of the customer.
        name: String,
    },
    /// A kind of resource this crate does not know, as Picqer sent it.
    Other {
        /// The type Picqer reported.
        kind: String,
        /// The source as Picqer sent it.
        raw: Value,
    },
}

impl CommentSource {
    fn from_parts(kind: String, raw: Value) -> Result<Self, serde_json::Error> {
        #[derive(Deserialize)]
        struct Order {
            idorder: OrderId,
            orderid: String,
        }
        #[derive(Deserialize)]
        struct Return {
            idreturn: ReturnId,
            returnid: String,
        }
        #[derive(Deserialize)]
        struct Picklist {
            idpicklist: PicklistId,
            picklistid: String,
        }
        #[derive(Deserialize)]
        struct PicklistBatch {
            idpicklist_batch: PicklistBatchId,
            #[serde(deserialize_with = "crate::serde_util::text")]
            picklist_batchid: String,
        }
        #[derive(Deserialize)]
        struct Supplier {
            idsupplier: SupplierId,
            name: String,
        }
        #[derive(Deserialize)]
        struct PurchaseOrder {
            idpurchaseorder: PurchaseOrderId,
            purchaseorderid: String,
        }
        #[derive(Deserialize)]
        struct Receipt {
            idreceipt: ReceiptId,
            receiptid: String,
        }
        #[derive(Deserialize)]
        struct Product {
            idproduct: ProductId,
            productcode: String,
            name: String,
        }
        #[derive(Deserialize)]
        struct Customer {
            idcustomer: CustomerId,
            customerid: Option<String>,
            name: String,
        }

        Ok(match kind.as_str() {
            "order" => {
                let s: Order = serde_json::from_value(raw)?;
                Self::Order {
                    idorder: s.idorder,
                    orderid: s.orderid,
                }
            }
            "return" => {
                let s: Return = serde_json::from_value(raw)?;
                Self::Return {
                    idreturn: s.idreturn,
                    returnid: s.returnid,
                }
            }
            "picklist" => {
                let s: Picklist = serde_json::from_value(raw)?;
                Self::Picklist {
                    idpicklist: s.idpicklist,
                    picklistid: s.picklistid,
                }
            }
            "picklist_batch" => {
                let s: PicklistBatch = serde_json::from_value(raw)?;
                Self::PicklistBatch {
                    idpicklist_batch: s.idpicklist_batch,
                    picklist_batchid: s.picklist_batchid,
                }
            }
            "supplier" => {
                let s: Supplier = serde_json::from_value(raw)?;
                Self::Supplier {
                    idsupplier: s.idsupplier,
                    name: s.name,
                }
            }
            "purchaseorder" => {
                let s: PurchaseOrder = serde_json::from_value(raw)?;
                Self::PurchaseOrder {
                    idpurchaseorder: s.idpurchaseorder,
                    purchaseorderid: s.purchaseorderid,
                }
            }
            "receipt" => {
                let s: Receipt = serde_json::from_value(raw)?;
                Self::Receipt {
                    idreceipt: s.idreceipt,
                    receiptid: s.receiptid,
                }
            }
            "product" => {
                let s: Product = serde_json::from_value(raw)?;
                Self::Product {
                    idproduct: s.idproduct,
                    productcode: s.productcode,
                    name: s.name,
                }
            }
            "customer" => {
                let s: Customer = serde_json::from_value(raw)?;
                Self::Customer {
                    idcustomer: s.idcustomer,
                    customerid: s.customerid,
                    name: s.name,
                }
            }
            _ => Self::Other { kind, raw },
        })
    }
}

/// A comment.
///
/// Picqer also sends `images` and `reactions`, which the documentation does not describe and which
/// are not read here.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Comment {
    /// The comment.
    pub idcomment: CommentId,
    /// The text.
    pub body: String,
    /// Who wrote the comment. For the API this is the user the API key belongs to.
    pub author: Person,
    /// What the comment was added to.
    pub source: CommentSource,
    /// The users and fulfilment customers mentioned in the comment.
    pub mentions: Vec<Mention>,
    /// Whether the comment is shown on related resources, such as the picklists of an order.
    pub show_at_related: Option<bool>,
    /// Whether the fulfilment customer can see the comment, with Picqer Fulfilment.
    pub is_visible_fulfilment: Option<bool>,
    /// When the comment was added.
    pub created_at: DateTime,
    /// When the comment was last changed.
    pub updated_at: DateTime,
}

impl<'de> Deserialize<'de> for Comment {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct RawMention {
            text: String,
            mentioned_type: String,
            mentioned: Value,
        }
        #[derive(Deserialize)]
        struct Raw {
            idcomment: CommentId,
            body: String,
            author_type: String,
            author: Value,
            source_type: String,
            source: Value,
            #[serde(default)]
            mentions: Vec<RawMention>,
            show_at_related: Option<bool>,
            is_visible_fulfilment: Option<bool>,
            created_at: DateTime,
            updated_at: DateTime,
        }

        let raw = Raw::deserialize(deserializer)?;
        let custom = serde::de::Error::custom;
        let mentions = raw
            .mentions
            .into_iter()
            .map(|mention| {
                Ok(Mention {
                    text: mention.text,
                    mentioned: Person::from_parts(mention.mentioned_type, mention.mentioned)?,
                })
            })
            .collect::<Result<_, serde_json::Error>>()
            .map_err(custom)?;
        Ok(Self {
            idcomment: raw.idcomment,
            body: raw.body,
            author: Person::from_parts(raw.author_type, raw.author).map_err(custom)?,
            source: CommentSource::from_parts(raw.source_type, raw.source).map_err(custom)?,
            mentions,
            show_at_related: raw.show_at_related,
            is_visible_fulfilment: raw.is_visible_fulfilment,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
        })
    }
}

/// What a comment is added to, or listed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommentTarget {
    /// An order.
    Order(OrderId),
    /// A return.
    Return(ReturnId),
    /// A picklist.
    Picklist(PicklistId),
    /// A picklist batch.
    PicklistBatch(PicklistBatchId),
    /// A supplier.
    Supplier(SupplierId),
    /// A purchase order.
    PurchaseOrder(PurchaseOrderId),
    /// A receipt.
    Receipt(ReceiptId),
    /// A product.
    Product(ProductId),
    /// A customer.
    Customer(CustomerId),
}

impl CommentTarget {
    fn segments(self) -> Vec<String> {
        match self {
            Self::Order(id) => segments(&[&"orders", &id, &"comments"]),
            Self::Return(id) => segments(&[&"returns", &id, &"comments"]),
            Self::Picklist(id) => segments(&[&"picklists", &id, &"comments"]),
            Self::PicklistBatch(id) => segments(&[&"picklists", &"batches", &id, &"comments"]),
            Self::Supplier(id) => segments(&[&"suppliers", &id, &"comments"]),
            Self::PurchaseOrder(id) => segments(&[&"purchaseorders", &id, &"comments"]),
            Self::Receipt(id) => segments(&[&"receipts", &id, &"comments"]),
            Self::Product(id) => segments(&[&"products", &id, &"comments"]),
            Self::Customer(id) => segments(&[&"customers", &id, &"comments"]),
        }
    }
}

/// A user or a fulfilment customer, to filter comments by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Commenter {
    /// A user.
    User(UserId),
    /// A fulfilment customer.
    FulfilmentCustomer(FulfilmentCustomerId),
}

impl Commenter {
    fn pairs(self, id: &'static str, kind: &'static str) -> Vec<(&'static str, String)> {
        match self {
            Self::User(user) => vec![(id, user.to_string())],
            Self::FulfilmentCustomer(customer) => vec![
                (id, customer.to_string()),
                (kind, "fulfilment_customer".to_owned()),
            ],
        }
    }
}

/// Filters for listing comments.
///
/// Picqer answers a filter on a user or fulfilment customer it does not know with a not found
/// error.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommentQuery {
    author: Option<Commenter>,
    mentioned: Option<Commenter>,
}

impl CommentQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the comments written by a user or fulfilment customer.
    pub fn author(mut self, author: Commenter) -> Self {
        self.author = Some(author);
        self
    }

    /// Lists the comments that mention a user or fulfilment customer.
    pub fn mentioned(mut self, mentioned: Commenter) -> Self {
        self.mentioned = Some(mentioned);
        self
    }

    fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(author) = self.author {
            pairs.extend(author.pairs("idauthor", "author_type"));
        }
        if let Some(mentioned) = self.mentioned {
            pairs.extend(mentioned.pairs("idmentioned", "mentioned_type"));
        }
        pairs
    }
}

/// A comment to add.
///
/// Prefix the full name of a user with `@` in the body to mention them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewComment {
    body: CommentBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_at_related: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_visible_fulfilment: Option<bool>,
}

impl NewComment {
    /// A comment with a text.
    pub fn new(body: CommentBody) -> Self {
        Self {
            body,
            show_at_related: None,
            is_visible_fulfilment: None,
        }
    }

    /// Sets whether the comment is shown on related resources. Picqer shows comments on orders
    /// and returns on their picklists unless this is `false`, and shows comments on purchase orders
    /// on their receipts only when it is `true`.
    pub fn show_at_related(mut self, show: bool) -> Self {
        self.show_at_related = Some(show);
        self
    }

    /// Sets whether the fulfilment customer can see the comment, with Picqer Fulfilment.
    pub fn is_visible_fulfilment(mut self, visible: bool) -> Self {
        self.is_visible_fulfilment = Some(visible);
        self
    }
}

pub(crate) fn list(query: &CommentQuery, page: Page) -> Call<Vec<Comment>> {
    let mut request = Request::new(Method::GET, segments(&[&"comments"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: CommentId) -> Call<Comment> {
    Call::new(Request::new(Method::GET, segments(&[&"comments", &id])))
}

pub(crate) fn delete(id: CommentId) -> NoContent {
    NoContent::new(Request::new(Method::DELETE, segments(&[&"comments", &id])))
}

pub(crate) fn for_target(target: CommentTarget, page: Page) -> Call<Vec<Comment>> {
    Call::new(page.apply(Request::new(Method::GET, target.segments())))
}

pub(crate) fn add(
    target: CommentTarget,
    comment: &NewComment,
) -> crate::error::Result<Call<Comment>> {
    Ok(Call::new(
        Request::new(Method::POST, target.segments()).json_body(comment)?,
    ))
}

macro_rules! comments_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of the comments on all resources, newest first.
        pub $($async)? fn list(
            &self,
            query: &$crate::comments::CommentQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::comments::Comment>> {
            self.client.execute($crate::comments::list(query, page)) $(.$await)?
        }

        /// Lists all the comments that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::comments::CommentQuery,
        ) -> $crate::error::Result<Vec<$crate::comments::Comment>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::comments::list(query, page)) $(.$await)?)
        }

        /// Fetches a comment.
        pub $($async)? fn get(
            &self,
            id: $crate::types::CommentId,
        ) -> $crate::error::Result<$crate::comments::Comment> {
            self.client.execute($crate::comments::get(id)) $(.$await)?
        }

        /// Deletes a comment.
        pub $($async)? fn delete(&self, id: $crate::types::CommentId) -> $crate::error::Result<()> {
            self.client.execute($crate::comments::delete(id)) $(.$await)?
        }

        /// Lists one page of the comments on a resource.
        pub $($async)? fn for_target(
            &self,
            target: $crate::comments::CommentTarget,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::comments::Comment>> {
            self.client.execute($crate::comments::for_target(target, page)) $(.$await)?
        }

        /// Lists all the comments on a resource, following the pages.
        pub $($async)? fn for_target_all(
            &self,
            target: $crate::comments::CommentTarget,
        ) -> $crate::error::Result<Vec<$crate::comments::Comment>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::comments::for_target(target, page)) $(.$await)?)
        }

        /// Adds a comment to a resource and returns it.
        pub $($async)? fn add(
            &self,
            target: $crate::comments::CommentTarget,
            comment: &$crate::comments::NewComment,
        ) -> $crate::error::Result<$crate::comments::Comment> {
            self.client.execute($crate::comments::add(target, comment)?) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use comments_methods;

/// Operations on comments.
#[derive(Debug, Clone, Copy)]
pub struct Comments<'a> {
    pub(crate) client: &'a Client,
}

impl Comments<'_> {
    comments_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_comment_needs_text() {
        assert!(CommentBody::new("Hello").is_ok());
        assert!(CommentBody::new("").is_err());
        assert!(CommentBody::new("   \n").is_err());
    }

    #[test]
    fn targets_have_their_own_paths() {
        assert_eq!(
            CommentTarget::Order(OrderId(5)).segments(),
            ["orders", "5", "comments"]
        );
        assert_eq!(
            CommentTarget::PicklistBatch(PicklistBatchId(7)).segments(),
            ["picklists", "batches", "7", "comments"]
        );
        assert_eq!(
            CommentTarget::PurchaseOrder(PurchaseOrderId(9)).segments(),
            ["purchaseorders", "9", "comments"]
        );
    }

    #[test]
    fn a_fulfilment_customer_filter_names_its_type() {
        let query = CommentQuery::new()
            .author(Commenter::User(UserId(3)))
            .mentioned(Commenter::FulfilmentCustomer(FulfilmentCustomerId(8)));
        assert_eq!(
            query.pairs(),
            vec![
                ("idauthor", "3".to_owned()),
                ("idmentioned", "8".to_owned()),
                ("mentioned_type", "fulfilment_customer".to_owned()),
            ]
        );
    }

    #[test]
    fn a_new_comment_sends_only_what_was_set() {
        let comment = NewComment::new(CommentBody::new("Hi").unwrap()).show_at_related(false);
        assert_eq!(
            serde_json::to_value(comment).unwrap(),
            serde_json::json!({"body": "Hi", "show_at_related": false})
        );
    }

    #[test]
    fn unknown_kinds_are_kept() {
        let comment: Comment = serde_json::from_value(serde_json::json!({
            "idcomment": 1, "body": "x",
            "author_type": "robot", "author": {"id": 1},
            "source_type": "shipment", "source": {"idshipment": 9},
            "mentions": [{"text": "@x", "mentioned_type": "robot", "mentioned": {}}],
            "created_at": "2026-11-09 10:00:00", "updated_at": "2026-11-09 10:00:00"
        }))
        .unwrap();
        assert!(matches!(comment.author, Person::Other { .. }));
        assert!(matches!(comment.source, CommentSource::Other { .. }));
        assert!(matches!(
            comment.mentions[0].mentioned,
            Person::Other { .. }
        ));
    }
}
