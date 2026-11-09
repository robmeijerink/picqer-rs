use std::num::NonZeroU32;

use serde::Serialize;

use crate::customers::Language;
use crate::suppliers::SupplierName;
use crate::types::{Date, FulfilmentCustomerId, Money, ProductId, SupplierId, WarehouseId};

/// A line of a purchase order to create, or to add to one.
///
/// Picqer fills in the price from the product when none is given. It does not allow a product on
/// a purchase order twice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewPurchaseOrderProduct {
    idproduct: ProductId,
    amount: NonZeroU32,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delivery_date: Option<Date>,
}

impl NewPurchaseOrderProduct {
    /// Orders `amount` of a product.
    pub fn new(idproduct: ProductId, amount: NonZeroU32) -> Self {
        Self {
            idproduct,
            amount,
            price: None,
            delivery_date: None,
        }
    }

    /// Sets the purchase price per piece, excluding VAT.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    /// Sets when this line is expected.
    pub fn delivery_date(mut self, date: Date) -> Self {
        self.delivery_date = Some(date);
        self
    }
}

/// Changes to a line of a purchase order. Only what is set is sent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PurchaseOrderLineChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<NonZeroU32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Money>,
}

impl PurchaseOrderLineChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the amount.
    pub fn amount(mut self, amount: NonZeroU32) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Changes the purchase price per piece, excluding VAT.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
enum SupplierRef {
    Id { idsupplier: SupplierId },
    Name { supplier_name: String },
}

/// A purchase order to create. Picqer creates it as a concept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewPurchaseOrder {
    #[serde(flatten)]
    supplier: SupplierRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    idwarehouse: Option<WarehouseId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supplier_orderid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remarks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delivery_date: Option<Date>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<Language>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idfulfilment_customer: Option<FulfilmentCustomerId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    products: Vec<NewPurchaseOrderProduct>,
}

impl NewPurchaseOrder {
    fn new(supplier: SupplierRef) -> Self {
        Self {
            supplier,
            idwarehouse: None,
            supplier_orderid: None,
            remarks: None,
            delivery_date: None,
            language: None,
            idfulfilment_customer: None,
            products: Vec::new(),
        }
    }

    /// A purchase order for a supplier of Picqer.
    pub fn for_supplier(supplier: SupplierId) -> Self {
        Self::new(SupplierRef::Id {
            idsupplier: supplier,
        })
    }

    /// A purchase order for a supplier that is only known by its name.
    pub fn for_supplier_name(name: SupplierName) -> Self {
        Self::new(SupplierRef::Name {
            supplier_name: name.as_str().to_owned(),
        })
    }

    /// Sets the warehouse the products are delivered to. Picqer picks one when none is given.
    pub fn warehouse(mut self, warehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(warehouse);
        self
    }

    /// Sets the number the supplier gave the order.
    pub fn supplier_orderid(mut self, supplier_orderid: impl Into<String>) -> Self {
        self.supplier_orderid = Some(supplier_orderid.into());
        self
    }

    /// Sets remarks.
    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = Some(remarks.into());
        self
    }

    /// Sets when the delivery is expected.
    pub fn delivery_date(mut self, date: Date) -> Self {
        self.delivery_date = Some(date);
        self
    }

    /// Sets the language. Picqer replaces a language it does not offer with `en`, so only the two
    /// it offers can be sent.
    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Sets the fulfilment customer, with Picqer Fulfilment. The products must belong to it.
    pub fn idfulfilment_customer(mut self, customer: FulfilmentCustomerId) -> Self {
        self.idfulfilment_customer = Some(customer);
        self
    }

    /// Adds a line.
    pub fn product(mut self, product: NewPurchaseOrderProduct) -> Self {
        self.products.push(product);
        self
    }
}

/// Changes to a purchase order. Only what is set is sent.
///
/// Picqer only changes these fields and ignores the others, such as the status. A purchase order
/// can be changed after it was purchased.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PurchaseOrderChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    supplier_orderid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remarks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delivery_date: Option<Date>,
}

impl PurchaseOrderChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the number the supplier gave the order.
    pub fn supplier_orderid(mut self, supplier_orderid: impl Into<String>) -> Self {
        self.supplier_orderid = Some(supplier_orderid.into());
        self
    }

    /// Changes the remarks.
    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = Some(remarks.into());
        self
    }

    /// Changes when the delivery is expected.
    pub fn delivery_date(mut self, date: Date) -> Self {
        self.delivery_date = Some(date);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).unwrap()
    }

    #[test]
    fn an_order_for_a_supplier_names_it_by_id() {
        let order = NewPurchaseOrder::for_supplier(SupplierId(3))
            .warehouse(WarehouseId(18))
            .language(Language::En)
            .product(
                NewPurchaseOrderProduct::new(ProductId(147), amount(25))
                    .price(Money::from_cents(2695).unwrap()),
            );
        assert_eq!(
            serde_json::to_value(order).unwrap(),
            serde_json::json!({
                "idsupplier": 3,
                "idwarehouse": 18,
                "language": "en",
                "products": [{"idproduct": 147, "amount": 25, "price": 26.95}]
            })
        );
    }

    #[test]
    fn an_order_can_name_a_supplier_that_only_has_a_name() {
        let order = NewPurchaseOrder::for_supplier_name(SupplierName::new("Acme").unwrap());
        assert_eq!(
            serde_json::to_value(order).unwrap(),
            serde_json::json!({"supplier_name": "Acme"})
        );
    }

    #[test]
    fn changes_send_only_what_is_set() {
        assert_eq!(
            serde_json::to_string(&PurchaseOrderChanges::new()).unwrap(),
            "{}"
        );
        let date: Date = "2031-01-02".parse().unwrap();
        assert_eq!(
            serde_json::to_value(PurchaseOrderChanges::new().delivery_date(date).remarks("r"))
                .unwrap(),
            serde_json::json!({"remarks": "r", "delivery_date": "2031-01-02"})
        );
        assert_eq!(
            serde_json::to_value(PurchaseOrderLineChanges::new().amount(amount(7))).unwrap(),
            serde_json::json!({"amount": 7})
        );
    }
}
