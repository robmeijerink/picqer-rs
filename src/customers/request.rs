use serde::Serialize;

use crate::country::CountryCode;
use crate::customers::response::Language;
use crate::error::ValidationError;
use crate::types::{CustomerFieldId, EmailAddress, FulfilmentCustomerId, TemplateId};

bounded_string! {
    /// The name of a customer.
    CustomerName, "customer name", 1, usize::MAX
}

bounded_string! {
    /// Your own number for a customer, unique among your customers.
    CustomerNumber, "customer number", 1, usize::MAX
}

bounded_string! {
    /// The name on an address. Picqer replaces an empty name with the name of the customer, which
    /// is not what an empty name asks for, so it must not be empty.
    AddressName, "address name", 1, usize::MAX
}

/// A discount in percent, from 0 up to and including 100.
///
/// Picqer stores any number, even a negative one or one above 100. Only what makes sense for a
/// discount can be sent, but what Picqer reports is read as it is.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DiscountPercentage(f64);

impl DiscountPercentage {
    /// Validates a discount.
    pub fn new(percent: f64) -> Result<Self, ValidationError> {
        if !(0.0..=100.0).contains(&percent) {
            return Err(ValidationError::new(
                "discount",
                format!("{percent} is not between 0 and 100"),
            ));
        }
        Ok(Self(percent))
    }

    /// The discount in percent.
    pub const fn get(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
struct Details {
    #[serde(skip_serializing_if = "Option::is_none")]
    customerid: Option<CustomerNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contactname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    telephone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emailaddress: Option<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discount: Option<DiscountPercentage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vatnumber: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    calculatevat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remarks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_order_remarks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<Language>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_split: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idtemplate: Option<TemplateId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idfulfilment_customer: Option<FulfilmentCustomerId>,
}

macro_rules! customer_setters {
    () => {
        detail_setters! {
            /// Sets your own number for the customer.
            customerid: CustomerNumber,
            /// Sets the contact person.
            contactname: String,
            /// Sets the phone number.
            telephone: String,
            /// Sets the email address.
            emailaddress: EmailAddress,
            /// Sets the discount the customer gets by default.
            discount: DiscountPercentage,
            /// Sets the VAT number.
            vatnumber: String,
            /// Sets whether the customer pays VAT. Picqer assumes so.
            calculatevat: bool,
            /// Sets internal remarks.
            remarks: String,
            /// Sets the remarks that are added to every new order.
            default_order_remarks: String,
            /// Sets the language of the customer.
            language: Language,
            /// Sets whether orders of the customer are split automatically.
            auto_split: bool,
            /// Sets the template used for the customer.
            idtemplate: TemplateId,
            /// Sets the fulfilment customer, with Picqer Fulfilment.
            idfulfilment_customer: FulfilmentCustomerId,
        }
    };
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
struct AddressDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    contactname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    zipcode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    defaultinvoice: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    defaultdelivery: Option<bool>,
}

macro_rules! address_setters {
    () => {
        detail_setters! {
            /// Sets the contact person.
            contactname: String,
            /// Sets the street and number.
            address: String,
            /// Sets the second address line.
            address2: String,
            /// Sets the postal code.
            zipcode: String,
            /// Sets the city.
            city: String,
            /// Sets the region, state or province.
            region: String,
            /// Sets whether this is the address used for invoices on a new order. Only one address
            /// can be, so Picqer takes the role from the address that had it.
            defaultinvoice: bool,
            /// Sets whether this is the address used for delivery on a new order. Only one
            /// address can be, so Picqer takes the role from the address that had it.
            defaultdelivery: bool,
        }
    };
}

/// The value of a customer field on a customer, to set or to clear.
///
/// Picqer reports the value as text, also when a number was sent. It does not check the value of a
/// radio, select, checkbox or multicheckbox field against the values of the field, and stores any
/// text. An empty text is stored as an empty value; [`cleared`](Self::cleared) removes the value.
/// [`CustomerField::value`] checks that a radio, select or checkbox value is a listed one, and
/// [`CustomerField::choices`] builds a multicheckbox value, so build one from the field when you can.
///
/// [`CustomerField::value`]: crate::customer_fields::CustomerField::value
/// [`CustomerField::choices`]: crate::customer_fields::CustomerField::choices
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CustomerFieldValue {
    idcustomerfield: CustomerFieldId,
    value: Option<String>,
}

impl CustomerFieldValue {
    /// Sets a customer field to a value, without checking it against the field.
    pub fn new(idcustomerfield: CustomerFieldId, value: impl Into<String>) -> Self {
        Self {
            idcustomerfield,
            value: Some(value.into()),
        }
    }

    /// Removes the value of a customer field from the customer.
    pub fn cleared(idcustomerfield: CustomerFieldId) -> Self {
        Self {
            idcustomerfield,
            value: None,
        }
    }
}

/// An address to add to a customer.
///
/// The documentation calls the country optional, but Picqer refuses to change an address that has
/// none, so it is required here.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NewCustomerAddress {
    name: AddressName,
    country: CountryCode,
    #[serde(flatten)]
    details: AddressDetails,
}

impl NewCustomerAddress {
    /// An address with the fields that are needed to use it later.
    pub fn new(name: AddressName, country: CountryCode) -> Self {
        Self {
            name,
            country,
            details: AddressDetails::default(),
        }
    }

    address_setters!();
}

/// Changes to an address of a customer. Only what is set is sent.
///
/// Picqer refuses a change to an address that has no country, whatever the change is, unless the
/// change sets one.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AddressChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<AddressName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<CountryCode>,
    #[serde(flatten)]
    details: AddressDetails,
}

impl AddressChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the name on the address.
    pub fn name(mut self, name: AddressName) -> Self {
        self.name = Some(name);
        self
    }

    /// Changes the country.
    pub fn country(mut self, country: CountryCode) -> Self {
        self.country = Some(country);
        self
    }

    address_setters!();
}

/// A customer to create.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NewCustomer {
    name: CustomerName,
    #[serde(flatten)]
    details: Details,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    addresses: Vec<NewCustomerAddress>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    customerfields: Vec<CustomerFieldValue>,
}

impl NewCustomer {
    /// A customer with the field Picqer requires.
    pub fn new(name: CustomerName) -> Self {
        Self {
            name,
            details: Details::default(),
            addresses: Vec::new(),
            customerfields: Vec::new(),
        }
    }

    /// Sets the values of customer fields.
    pub fn customerfields(mut self, customerfields: Vec<CustomerFieldValue>) -> Self {
        self.customerfields = customerfields;
        self
    }

    /// Adds an address to the customer.
    pub fn address(mut self, address: NewCustomerAddress) -> Self {
        self.addresses.push(address);
        self
    }

    customer_setters!();
}

/// Changes to a customer. Only what is set is sent.
///
/// Addresses are changed through their own calls: Picqer ignores them here.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CustomerChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<CustomerName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    customerfields: Option<Vec<CustomerFieldValue>>,
    #[serde(flatten)]
    details: Details,
}

impl CustomerChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the name.
    pub fn name(mut self, name: CustomerName) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the values of customer fields.
    pub fn customerfields(mut self, customerfields: Vec<CustomerFieldValue>) -> Self {
        self.customerfields = Some(customerfields);
        self
    }

    customer_setters!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_discount_is_a_percentage_between_zero_and_a_hundred() {
        for accepted in [0.0, 10.5, 100.0] {
            assert!(DiscountPercentage::new(accepted).is_ok(), "{accepted}");
        }
        for refused in [-0.1, 100.1, 150.0, f64::NAN, f64::INFINITY] {
            assert!(DiscountPercentage::new(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn a_new_customer_sends_what_was_set_and_its_addresses() {
        let customer = NewCustomer::new(CustomerName::new("Example B.V.").unwrap())
            .language(Language::En)
            .discount(DiscountPercentage::new(12.5).unwrap())
            .address(
                NewCustomerAddress::new(AddressName::new("Home").unwrap(), CountryCode::NL)
                    .city("Amsterdam".to_owned())
                    .defaultinvoice(true),
            );
        assert_eq!(
            serde_json::to_value(customer).unwrap(),
            serde_json::json!({
                "name": "Example B.V.",
                "language": "en",
                "discount": 12.5,
                "addresses": [{
                    "name": "Home", "country": "NL", "city": "Amsterdam", "defaultinvoice": true
                }]
            })
        );
    }

    #[test]
    fn changes_send_only_what_is_set() {
        assert_eq!(
            serde_json::to_string(&CustomerChanges::new()).unwrap(),
            "{}"
        );
        assert_eq!(
            serde_json::to_string(&AddressChanges::new().city("Utrecht".to_owned())).unwrap(),
            r#"{"city":"Utrecht"}"#
        );
    }

    #[test]
    fn names_and_numbers_are_not_empty() {
        assert!(CustomerName::new("").is_err());
        assert!(CustomerNumber::new("").is_err());
        assert!(AddressName::new("").is_err());
        assert!(CustomerNumber::new("ABC-1").is_ok());
    }
}
