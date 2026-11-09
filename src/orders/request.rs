use std::num::NonZeroU32;

use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::country::CountryCode;
use crate::customers::{AddressName, DiscountPercentage, Language};
use crate::order_fields::OrderFieldValue;
use crate::types::{
    CustomerId, Date, EmailAddress, FulfilmentCustomerId, Money, NonEmpty, ProductCode, ProductId,
    ShippingProviderProfileId, TemplateId,
};

/// A product to order, by id or by code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
enum ProductRef {
    Id { idproduct: ProductId },
    Code { productcode: ProductCode },
}

/// A line of an order to create.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NewOrderProduct {
    #[serde(flatten)]
    product: ProductRef,
    amount: NonZeroU32,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remarks: Option<String>,
}

impl NewOrderProduct {
    fn new(product: ProductRef, amount: NonZeroU32) -> Self {
        Self {
            product,
            amount,
            price: None,
            name: None,
            remarks: None,
        }
    }

    /// A line for a product by its id. Picqer fills in the name and the price.
    pub fn by_id(idproduct: ProductId, amount: NonZeroU32) -> Self {
        Self::new(ProductRef::Id { idproduct }, amount)
    }

    /// A line for a product by its code. Picqer fills in the name and the price.
    pub fn by_code(productcode: ProductCode, amount: NonZeroU32) -> Self {
        Self::new(ProductRef::Code { productcode }, amount)
    }

    /// Overrides the price per piece, excluding VAT.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    /// Overrides the name of the product on this line.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets remarks about the line.
    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = Some(remarks.into());
        self
    }
}

/// A line to add to an order that is still a concept.
///
/// Adding a product the order already has adds to the amount of its line. Picqer only looks the
/// product up by its id here.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NewOrderLine {
    idproduct: ProductId,
    amount: NonZeroU32,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remarks: Option<String>,
}

impl NewOrderLine {
    /// Adds `amount` of a product. Picqer fills in the name and the price.
    pub fn new(idproduct: ProductId, amount: NonZeroU32) -> Self {
        Self {
            idproduct,
            amount,
            price: None,
            name: None,
            remarks: None,
        }
    }

    /// Overrides the price per piece, excluding VAT.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    /// Overrides the name of the product on the line.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets remarks about the line.
    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = Some(remarks.into());
        self
    }
}

/// Changes to a line of an order. Only what is set is sent.
///
/// The amount can only be changed while the order is a concept.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct OrderLineChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<NonZeroU32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remarks: Option<String>,
}

impl OrderLineChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the amount. Take the line out of the order to have none.
    pub fn amount(mut self, amount: NonZeroU32) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Changes the price per piece, excluding VAT.
    pub fn price(mut self, price: Money) -> Self {
        self.price = Some(price);
        self
    }

    /// Changes the name of the product on the line.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Changes the remarks.
    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = Some(remarks.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Delivery,
    Invoice,
}

impl Side {
    fn prefix(self) -> &'static str {
        match self {
            Self::Delivery => "delivery",
            Self::Invoice => "invoice",
        }
    }
}

macro_rules! address_fields {
    () => {
        /// Sets the contact person.
        pub fn contactname(mut self, contactname: impl Into<String>) -> Self {
            self.fields
                .insert("contactname".into(), json!(contactname.into()));
            self
        }

        /// Sets the street and number.
        pub fn address(mut self, address: impl Into<String>) -> Self {
            self.fields.insert("address".into(), json!(address.into()));
            self
        }

        /// Sets the second address line.
        pub fn address2(mut self, address2: impl Into<String>) -> Self {
            self.fields
                .insert("address2".into(), json!(address2.into()));
            self
        }

        /// Sets the postal code.
        pub fn zipcode(mut self, zipcode: impl Into<String>) -> Self {
            self.fields.insert("zipcode".into(), json!(zipcode.into()));
            self
        }

        /// Sets the city.
        pub fn city(mut self, city: impl Into<String>) -> Self {
            self.fields.insert("city".into(), json!(city.into()));
            self
        }

        /// Sets the region, state or province.
        pub fn region(mut self, region: impl Into<String>) -> Self {
            self.fields.insert("region".into(), json!(region.into()));
            self
        }
    };
}

/// An address on an order, to deliver to or to invoice.
///
/// Picqer requires a name for a guest order and the documentation marks the country as required
/// then too, so both are required here.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderAddress {
    fields: Map<String, Value>,
}

impl OrderAddress {
    /// An address with a name and a country.
    pub fn new(name: AddressName, country: CountryCode) -> Self {
        let mut fields = Map::new();
        fields.insert("name".into(), json!(name.as_str()));
        fields.insert("country".into(), json!(country.as_str()));
        Self { fields }
    }

    address_fields!();
}

/// Changes to an address on an order. Only what is set is sent.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OrderAddressChanges {
    fields: Map<String, Value>,
}

impl OrderAddressChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the name.
    pub fn name(mut self, name: AddressName) -> Self {
        self.fields.insert("name".into(), json!(name.as_str()));
        self
    }

    /// Changes the country.
    pub fn country(mut self, country: CountryCode) -> Self {
        self.fields
            .insert("country".into(), json!(country.as_str()));
        self
    }

    address_fields!();
}

fn add_address(into: &mut Map<String, Value>, side: Side, fields: &Map<String, Value>) {
    for (name, value) in fields {
        into.insert(format!("{}{name}", side.prefix()), value.clone());
    }
}

macro_rules! order_fields {
    () => {
        /// Sets your own reference for the order.
        pub fn reference(mut self, reference: impl Into<String>) -> Self {
            self.fields
                .insert("reference".into(), json!(reference.into()));
            self
        }

        /// Sets remarks of the customer.
        pub fn customer_remarks(mut self, remarks: impl Into<String>) -> Self {
            self.fields
                .insert("customer_remarks".into(), json!(remarks.into()));
            self
        }

        /// Sets whether the order may be delivered in parts.
        pub fn partialdelivery(mut self, partial: bool) -> Self {
            self.fields.insert("partialdelivery".into(), json!(partial));
            self
        }

        /// Sets the discount, in percent.
        pub fn discount(mut self, discount: DiscountPercentage) -> Self {
            self.fields.insert("discount".into(), json!(discount.get()));
            self
        }

        /// Sets the language of the order.
        pub fn language(mut self, language: Language) -> Self {
            self.fields
                .insert("language".into(), json!(language.as_str()));
            self
        }

        /// Sets the preferred delivery date.
        pub fn preferred_delivery_date(mut self, date: Date) -> Self {
            self.fields
                .insert("preferred_delivery_date".into(), json!(date.to_string()));
            self
        }

        /// Sets the email address. Picqer does not check it on an order, this crate does.
        pub fn emailaddress(mut self, emailaddress: EmailAddress) -> Self {
            self.fields
                .insert("emailaddress".into(), json!(emailaddress.as_str()));
            self
        }

        /// Sets the phone number.
        pub fn telephone(mut self, telephone: impl Into<String>) -> Self {
            self.fields
                .insert("telephone".into(), json!(telephone.into()));
            self
        }

        /// Sets whether VAT is calculated.
        pub fn calculatevat(mut self, calculate: bool) -> Self {
            self.fields.insert("calculatevat".into(), json!(calculate));
            self
        }

        /// Sets the template used for the order.
        pub fn idtemplate(mut self, idtemplate: TemplateId) -> Self {
            self.fields.insert("idtemplate".into(), json!(idtemplate));
            self
        }

        /// Sets the shipping provider profile.
        pub fn idshippingprovider_profile(mut self, profile: ShippingProviderProfileId) -> Self {
            self.fields
                .insert("idshippingprovider_profile".into(), json!(profile));
            self
        }

        /// Sets whether the order was invoiced.
        pub fn invoiced(mut self, invoiced: bool) -> Self {
            self.fields.insert("invoiced".into(), json!(invoiced));
            self
        }
    };
}

/// An order to create. Picqer creates it as a concept.
///
/// Picqer accepts an order without lines, although its documentation says they are required, so at
/// least one is required here.
#[derive(Debug, Clone, PartialEq)]
pub struct NewOrder {
    customer: Option<CustomerId>,
    products: NonEmpty<NewOrderProduct>,
    fields: Map<String, Value>,
}

impl NewOrder {
    /// An order for a customer. The addresses of the customer are used unless set here.
    pub fn for_customer(customer: CustomerId, products: NonEmpty<NewOrderProduct>) -> Self {
        Self {
            customer: Some(customer),
            products,
            fields: Map::new(),
        }
    }

    /// An order without a customer, with the address to deliver to and the address to invoice.
    ///
    /// The products must belong to the same fulfilment customer as the order, so set
    /// [`idfulfilment_customer`](Self::idfulfilment_customer) when they belong to one.
    pub fn guest(
        delivery: OrderAddress,
        invoice: OrderAddress,
        products: NonEmpty<NewOrderProduct>,
    ) -> Self {
        let mut fields = Map::new();
        add_address(&mut fields, Side::Delivery, &delivery.fields);
        add_address(&mut fields, Side::Invoice, &invoice.fields);
        Self {
            customer: None,
            products,
            fields,
        }
    }

    /// Sets the address to deliver to, instead of the one of the customer.
    pub fn delivery(mut self, address: OrderAddress) -> Self {
        add_address(&mut self.fields, Side::Delivery, &address.fields);
        self
    }

    /// Sets the address to invoice, instead of the one of the customer.
    pub fn invoice(mut self, address: OrderAddress) -> Self {
        add_address(&mut self.fields, Side::Invoice, &address.fields);
        self
    }

    /// Sets the fulfilment customer, with Picqer Fulfilment.
    pub fn idfulfilment_customer(mut self, customer: FulfilmentCustomerId) -> Self {
        self.fields
            .insert("idfulfilment_customer".into(), json!(customer));
        self
    }

    /// Sets the values of order fields.
    pub fn orderfields(mut self, orderfields: Vec<OrderFieldValue>) -> Self {
        self.fields.insert("orderfields".into(), json!(orderfields));
        self
    }

    order_fields!();

    pub(crate) fn to_value(&self) -> Value {
        let mut body = self.fields.clone();
        body.insert("idcustomer".into(), json!(self.customer));
        body.insert("products".into(), json!(self.products));
        Value::Object(body)
    }
}

/// Changes to an order. Only what is set is sent.
///
/// Picqer only changes the fields listed here and ignores the others, such as the status and the
/// lines. A processed order can still be changed, but only a concept can have its lines changed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OrderChanges {
    fields: Map<String, Value>,
}

impl OrderChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Links the order to another customer.
    pub fn customer(mut self, customer: CustomerId) -> Self {
        self.fields.insert("idcustomer".into(), json!(customer));
        self
    }

    /// Unlinks the order from its customer.
    pub fn without_customer(mut self) -> Self {
        self.fields.insert("idcustomer".into(), Value::Null);
        self
    }

    /// Removes the reference.
    pub fn without_reference(mut self) -> Self {
        self.fields.insert("reference".into(), Value::Null);
        self
    }

    /// Changes the address to deliver to.
    pub fn delivery(mut self, changes: OrderAddressChanges) -> Self {
        add_address(&mut self.fields, Side::Delivery, &changes.fields);
        self
    }

    /// Changes the address to invoice.
    pub fn invoice(mut self, changes: OrderAddressChanges) -> Self {
        add_address(&mut self.fields, Side::Invoice, &changes.fields);
        self
    }

    order_fields!();

    pub(crate) fn to_value(&self) -> Value {
        Value::Object(self.fields.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).unwrap()
    }

    fn lines() -> NonEmpty<NewOrderProduct> {
        NonEmpty::new(vec![NewOrderProduct::by_id(ProductId(633), amount(1))]).unwrap()
    }

    fn address(name: &str) -> OrderAddress {
        OrderAddress::new(AddressName::new(name).unwrap(), CountryCode::NL)
    }

    #[test]
    fn an_order_for_a_customer_names_the_customer_and_its_lines() {
        let order = NewOrder::for_customer(CustomerId(5633), lines()).reference("R1");
        assert_eq!(
            order.to_value(),
            json!({
                "idcustomer": 5633,
                "reference": "R1",
                "products": [{"idproduct": 633, "amount": 1}]
            })
        );
    }

    #[test]
    fn a_guest_order_has_both_addresses_with_their_prefixes() {
        let order = NewOrder::guest(address("John").city("Zeist"), address("John B.V."), lines())
            .idfulfilment_customer(FulfilmentCustomerId(7));
        let body = order.to_value();
        assert_eq!(body["idcustomer"], Value::Null);
        assert_eq!(body["deliveryname"], "John");
        assert_eq!(body["deliverycity"], "Zeist");
        assert_eq!(body["deliverycountry"], "NL");
        assert_eq!(body["invoicename"], "John B.V.");
        assert_eq!(body["idfulfilment_customer"], 7);
    }

    #[test]
    fn a_line_can_name_its_product_by_code_and_override_the_price() {
        let line = NewOrderProduct::by_code(ProductCode::new("A-1").unwrap(), amount(2))
            .price(Money::from_cents(999).unwrap())
            .remarks("careful");
        assert_eq!(
            serde_json::to_value(line).unwrap(),
            json!({"productcode": "A-1", "amount": 2, "price": 9.99, "remarks": "careful"})
        );
    }

    #[test]
    fn changes_send_only_what_is_set_and_can_clear_the_customer_and_reference() {
        assert_eq!(OrderChanges::new().to_value(), json!({}));
        let changes = OrderChanges::new()
            .without_customer()
            .without_reference()
            .delivery(OrderAddressChanges::new().city("Utrecht"))
            .discount(DiscountPercentage::new(15.0).unwrap());
        assert_eq!(
            changes.to_value(),
            json!({
                "idcustomer": null, "reference": null, "deliverycity": "Utrecht", "discount": 15.0
            })
        );
    }

    #[test]
    fn a_line_change_sends_only_what_is_set() {
        let changes = OrderLineChanges::new().amount(amount(4)).name("Renamed");
        assert_eq!(
            serde_json::to_value(changes).unwrap(),
            json!({"amount": 4, "name": "Renamed"})
        );
        assert_eq!(
            serde_json::to_string(&OrderLineChanges::new()).unwrap(),
            "{}"
        );
    }
}
