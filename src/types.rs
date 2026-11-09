//! Validated primitives shared by the requests of the API.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::PrimitiveDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::error::ValidationError;

const MAX_SUBDOMAIN_LEN: usize = 63;
const ENV_SUBDOMAIN: &str = "PICQER_SUBDOMAIN";

/// The subdomain of a Picqer account: `example` in `https://example.picqer.com`.
///
/// It must be a valid DNS label: 1 to 63 letters, digits or hyphens, not starting or ending with a
/// hyphen.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subdomain(String);

impl Subdomain {
    /// Validates a subdomain.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_SUBDOMAIN_LEN {
            return Err(ValidationError::new(
                "subdomain",
                format!("length {} is outside 1..={MAX_SUBDOMAIN_LEN}", value.len()),
            ));
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(ValidationError::new(
                "subdomain",
                "must only contain letters, digits and hyphens",
            ));
        }
        if value.starts_with('-') || value.ends_with('-') {
            return Err(ValidationError::new(
                "subdomain",
                "must not start or end with a hyphen",
            ));
        }
        Ok(Self(value))
    }

    /// Reads the subdomain from `PICQER_SUBDOMAIN`.
    pub fn from_env() -> Result<Self, ValidationError> {
        match std::env::var(ENV_SUBDOMAIN) {
            Ok(value) if !value.is_empty() => Self::new(value),
            _ => Err(ValidationError::new(
                "subdomain",
                format!("{ENV_SUBDOMAIN} is not set"),
            )),
        }
    }

    /// The validated value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Subdomain {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for Subdomain {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Subdomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The user agent of an integration.
///
/// Picqer requires the name of the application together with a website or an email address, so it
/// can reach whoever is behind the requests. The crate appends its own name and version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAgent(String);

impl UserAgent {
    /// Validates the name of an application and a website or email address to contact.
    ///
    /// Both must be printable ASCII without parentheses, which would break the
    /// `application (contact)` form, and neither may be empty or start or end with a space.
    pub fn new(
        application: impl AsRef<str>,
        contact: impl AsRef<str>,
    ) -> Result<Self, ValidationError> {
        let application = application.as_ref();
        let contact = contact.as_ref();
        Self::check("user agent application", application)?;
        Self::check("user agent contact", contact)?;
        Ok(Self(format!(
            "{application} ({contact}) {}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )))
    }

    fn check(field: &'static str, value: &str) -> Result<(), ValidationError> {
        let printable = value.chars().all(|c| c == ' ' || c.is_ascii_graphic());
        if value.is_empty() || value != value.trim() || !printable || value.contains(['(', ')']) {
            return Err(ValidationError::new(
                field,
                "must be printable ASCII without parentheses or surrounding spaces",
            ));
        }
        Ok(())
    }

    /// The header value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

id_type! {
    /// Identifier of a product.
    ProductId
}

id_type! {
    /// Identifier of a warehouse.
    WarehouseId
}

id_type! {
    /// Identifier of a VAT group.
    VatGroupId
}

id_type! {
    /// Identifier of a tag.
    TagId
}

id_type! {
    /// Identifier of a supplier.
    SupplierId
}

id_type! {
    /// Identifier of a pricelist.
    PricelistId
}

id_type! {
    /// Identifier of a product field.
    ProductFieldId
}

id_type! {
    /// Identifier of a fulfilment customer.
    FulfilmentCustomerId
}

id_type! {
    /// Identifier of the link between a composition and one of its parts.
    ProductPartId
}

id_type! {
    /// Identifier of an image of a product.
    ProductImageId
}

id_type! {
    /// Identifier of a purchase order.
    PurchaseOrderId
}

id_type! {
    /// Identifier of a line of a purchase order.
    PurchaseOrderProductId
}

id_type! {
    /// Identifier of a customer.
    CustomerId
}

id_type! {
    /// Identifier of an address of a customer.
    CustomerAddressId
}

id_type! {
    /// Identifier of a custom field of customers.
    CustomerFieldId
}

id_type! {
    /// Identifier of a template.
    TemplateId
}

id_type! {
    /// Identifier of a line of an order.
    OrderProductId
}

id_type! {
    /// Identifier of a custom field of orders.
    OrderFieldId
}

id_type! {
    /// Identifier of a shipping provider profile.
    ShippingProviderProfileId
}

id_type! {
    /// Identifier of a shipment.
    ShipmentId
}

id_type! {
    /// Identifier of a parcel of a shipment.
    ParcelId
}

id_type! {
    /// Identifier of a document of a parcel, such as its shipping label.
    ParcelDocumentId
}

id_type! {
    /// Identifier of a line of a picklist.
    PicklistProductId
}

id_type! {
    /// Identifier of the part of a picklist line that is picked at one location.
    PicklistProductLocationId
}

id_type! {
    /// Identifier of a packaging.
    PackagingId
}

id_type! {
    /// Identifier of a shipping provider.
    ShippingProviderId
}

id_type! {
    /// Identifier of a printer.
    PrinterId
}

id_type! {
    /// Identifier of a packing station.
    PackingStationId
}

id_type! {
    /// Identifier of a comment.
    CommentId
}

id_type! {
    /// Identifier of an order.
    OrderId
}

id_type! {
    /// Identifier of a return.
    ReturnId
}

id_type! {
    /// Identifier of a picklist.
    PicklistId
}

id_type! {
    /// Identifier of a picklist batch.
    PicklistBatchId
}

id_type! {
    /// Identifier of a location type.
    LocationTypeId
}

id_type! {
    /// Identifier of a task.
    TaskId
}

id_type! {
    /// Identifier of an order of a webshop integration.
    WebshopOrderId
}

id_type! {
    /// Identifier of a webshop integration.
    CompanyWebshopId
}

id_type! {
    /// Identifier of a webhook subscription, which Picqer calls a hook.
    HookId
}

id_type! {
    /// Identifier of a picking container.
    PickingContainerId
}

id_type! {
    /// Identifier of a product on a receipt.
    ReceiptProductId
}

id_type! {
    /// Identifier of a product that is returned.
    ReturnedProductId
}

id_type! {
    /// Identifier of a replacement product of a return.
    ReplacementProductId
}

id_type! {
    /// Identifier of a status of returns.
    ReturnStatusId
}

id_type! {
    /// Identifier of a reason for a return.
    ReturnReasonId
}

id_type! {
    /// Identifier of an entry in the logbook of a return.
    ReturnLogId
}

id_type! {
    /// Identifier of a backorder.
    BackorderId
}

id_type! {
    /// Identifier of an entry in the stock history of a location.
    LocationStockHistoryId
}

id_type! {
    /// Identifier of a preset to create picklist batches with.
    PicklistBatchPresetId
}

id_type! {
    /// Identifier of a receipt.
    ReceiptId
}

id_type! {
    /// Identifier of a location in a warehouse.
    LocationId
}

id_type! {
    /// Identifier of a user.
    UserId
}

id_type! {
    /// Identifier of an entry in the stock history.
    StockHistoryId
}

const DATE_TIME: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

/// A date and time in the `YYYY-MM-DD HH:MM:SS` format of the API.
///
/// Picqer does not report a time zone. The values are in the local time of the account; for an
/// account in the Netherlands that is `Europe/Amsterdam`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DateTime(PrimitiveDateTime);

impl DateTime {
    /// Wraps a date and time.
    pub const fn new(value: PrimitiveDateTime) -> Self {
        Self(value)
    }

    /// The wrapped value.
    pub const fn get(self) -> PrimitiveDateTime {
        self.0
    }
}

impl From<PrimitiveDateTime> for DateTime {
    fn from(value: PrimitiveDateTime) -> Self {
        Self(value)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.0.format(DATE_TIME).map_err(|_| fmt::Error)?;
        f.write_str(&text)
    }
}

impl FromStr for DateTime {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PrimitiveDateTime::parse(s, DATE_TIME)
            .map(Self)
            .map_err(|_| {
                ValidationError::new("date time", format!("`{s}` is not `YYYY-MM-DD HH:MM:SS`"))
            })
    }
}

impl Serialize for DateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

const DATE: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

/// A date in the `YYYY-MM-DD` format of the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date(time::Date);

impl Date {
    /// Wraps a date.
    pub const fn new(value: time::Date) -> Self {
        Self(value)
    }

    /// The wrapped value.
    pub const fn get(self) -> time::Date {
        self.0
    }
}

impl From<time::Date> for Date {
    fn from(value: time::Date) -> Self {
        Self(value)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.0.format(DATE).map_err(|_| fmt::Error)?;
        f.write_str(&text)
    }
}

impl FromStr for Date {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        time::Date::parse(s, DATE)
            .map(Self)
            .map_err(|_| ValidationError::new("date", format!("`{s}` is not `YYYY-MM-DD`")))
    }
}

impl Serialize for Date {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// An amount of euros, kept in cents.
///
/// Picqer stores two decimals and reports them as a whole number or a fraction, so `12` and `12.5`
/// are both read. The API accepts negative amounts, so this type does too. Amounts are sent as JSON
/// numbers, which cannot hold every cent exactly beyond 2^53, so that is the limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Money(i64);

impl Money {
    /// The largest number of cents, in either direction, that survives a trip as a JSON number.
    pub const MAX_CENTS: i64 = 1 << 53;

    /// An amount in cents.
    pub fn from_cents(cents: i64) -> Result<Self, ValidationError> {
        if cents.unsigned_abs() > Self::MAX_CENTS.unsigned_abs() {
            return Err(ValidationError::new(
                "amount",
                format!("{cents} cents is beyond {}", Self::MAX_CENTS),
            ));
        }
        Ok(Self(cents))
    }

    /// A whole number of euros.
    pub fn from_euros(euros: i64) -> Result<Self, ValidationError> {
        euros
            .checked_mul(100)
            .ok_or_else(|| ValidationError::new("amount", "is too large"))
            .and_then(Self::from_cents)
    }

    /// The amount in cents.
    pub const fn cents(self) -> i64 {
        self.0
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let cents = self.0.unsigned_abs();
        write!(f, "{sign}{}.{:02}", cents / 100, cents % 100)
    }
}

impl Serialize for Money {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0 as f64 / 100.0)
    }
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Number {
            Whole(i64),
            Fraction(f64),
        }

        let cents = match Number::deserialize(deserializer)? {
            Number::Whole(euros) => euros
                .checked_mul(100)
                .ok_or_else(|| serde::de::Error::custom("amount is too large"))?,
            Number::Fraction(euros) => {
                let scaled = euros * 100.0;
                let rounded = scaled.round();
                if !scaled.is_finite() || (scaled - rounded).abs() > 1e-6 {
                    return Err(serde::de::Error::custom(format!(
                        "{euros} does not have at most two decimals"
                    )));
                }
                if rounded.abs() > Self::MAX_CENTS as f64 {
                    return Err(serde::de::Error::custom("amount is too large"));
                }
                rounded as i64
            }
        };
        Self::from_cents(cents).map_err(serde::de::Error::custom)
    }
}

/// A weight in grams.
///
/// Building one rejects negative weights, but the API stores what it is given, so a weight it
/// reports can be negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Grams(i64);

impl Grams {
    /// A weight in grams.
    pub fn new(grams: u32) -> Self {
        Self(i64::from(grams))
    }

    /// The weight in grams.
    pub const fn get(self) -> i64 {
        self.0
    }
}

/// A length in centimeters, with the same leniency as [`Grams`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Centimeters(i64);

impl Centimeters {
    /// A length in centimeters.
    pub fn new(centimeters: u32) -> Self {
        Self(i64::from(centimeters))
    }

    /// The length in centimeters.
    pub const fn get(self) -> i64 {
        self.0
    }
}

/// A percentage, such as the rate of a VAT group. Picqer reports it as a whole number or a
/// fraction.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Percentage(f64);

impl Percentage {
    /// Wraps a percentage, rejecting values that are not finite.
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if !value.is_finite() {
            return Err(ValidationError::new("percentage", "must be finite"));
        }
        Ok(Self(value))
    }

    /// The percentage.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(f64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

bounded_string! {
    /// The code of a product, which Picqer requires to be unique.
    ProductCode, "product code", 1, usize::MAX
}

bounded_string! {
    /// The name of a product.
    ProductName, "product name", 1, usize::MAX
}

bounded_string! {
    /// An email address.
    ///
    /// Only the shape `local@domain` is checked: both parts must be present, nothing may contain
    /// whitespace or control characters and the address is at most 254 characters long. Picqer
    /// checks more than that and refuses addresses that do not look valid to it.
    EmailAddress, "email address", 3, 254,
    |value| {
        if value.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Some("must not contain whitespace or control characters");
        }
        match value.rsplit_once('@') {
            Some((local, domain)) if !local.is_empty() && !domain.is_empty() => None,
            _ => Some("must look like local@domain"),
        }
    }
}

bounded_string! {
    /// The reason an order is paused. Picqer requires one that is not blank.
    PauseReason, "pause reason", 1, usize::MAX,
    |value| value.trim().is_empty().then_some("must not be blank")
}

bounded_string! {
    /// The number of a picklist, such as `P2026-1370`, as Picqer assigns it.
    PicklistNumber, "picklist number", 1, usize::MAX
}

bounded_string! {
    /// The number of an order, such as `O2026-1599`, as Picqer assigns it.
    OrderNumber, "order number", 1, usize::MAX
}

bounded_string! {
    /// The Manufacturer Identification Code of a product, at most 15 characters.
    MidCode, "mid code", 1, 15
}

/// A list with at least one and at most `MAX` items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NonEmpty<T, const MAX: usize = { usize::MAX }>(Vec<T>);

impl<T, const MAX: usize> NonEmpty<T, MAX> {
    /// Wraps a list, rejecting empty lists and lists that are too long.
    pub fn new(items: Vec<T>) -> Result<Self, ValidationError> {
        if items.is_empty() {
            return Err(ValidationError::new(
                "list",
                "must contain at least one item",
            ));
        }
        if items.len() > MAX {
            return Err(ValidationError::new(
                "list",
                format!("contains {} items, at most {MAX} are allowed", items.len()),
            ));
        }
        Ok(Self(items))
    }

    /// The items.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Iterates over the items.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    /// The number of items, which is at least one.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Always `false`; a [`NonEmpty`] holds at least one item.
    pub fn is_empty(&self) -> bool {
        false
    }
}

impl<T, const MAX: usize> TryFrom<Vec<T>> for NonEmpty<T, MAX> {
    type Error = ValidationError;

    fn try_from(items: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdomains_are_dns_labels() {
        assert!(Subdomain::new("example").is_ok());
        assert!(Subdomain::new("my-shop-2").is_ok());
        assert!(Subdomain::new("").is_err());
        assert!(Subdomain::new("-shop").is_err());
        assert!(Subdomain::new("shop-").is_err());
        assert!(Subdomain::new("my.shop").is_err());
        assert!(Subdomain::new("shop/evil").is_err());
        assert!(Subdomain::new("shop@evil").is_err());
        assert!(Subdomain::new("a".repeat(64)).is_err());
        assert!(Subdomain::new("a".repeat(63)).is_ok());
    }

    #[test]
    fn user_agent_names_the_application_and_the_crate() {
        let agent = UserAgent::new("My Shop", "dev@example.com").unwrap();
        assert_eq!(
            agent.as_str(),
            format!(
                "My Shop (dev@example.com) picqer-rs/{}",
                env!("CARGO_PKG_VERSION")
            )
        );
    }

    #[test]
    fn user_agent_accepts_the_form_of_the_documentation() {
        let documented =
            UserAgent::new("MyPicqerClient", "picqer.com/api - support@picqer.com").unwrap();
        assert!(
            documented
                .as_str()
                .starts_with("MyPicqerClient (picqer.com/api - support@picqer.com) ")
        );
        assert!(UserAgent::new("My Shop", "nobody").is_ok());
    }

    #[test]
    fn user_agent_rejects_unusable_parts() {
        assert!(UserAgent::new("", "dev@example.com").is_err());
        assert!(UserAgent::new(" My Shop", "dev@example.com").is_err());
        assert!(UserAgent::new("My (Shop)", "dev@example.com").is_err());
        assert!(UserAgent::new("My Shop", "").is_err());
        assert!(UserAgent::new("My Shop", "dev@example.com ").is_err());
        assert!(UserAgent::new("My Shop", "d\u{e9}v@example.com").is_err());
        assert!(UserAgent::new("My\tShop", "dev@example.com").is_err());
    }

    fn money(json: &str) -> Result<Money, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn money_reads_wholes_and_fractions() {
        assert_eq!(money("12").unwrap().cents(), 1200);
        assert_eq!(money("12.5").unwrap().cents(), 1250);
        assert_eq!(money("54.46").unwrap().cents(), 5446);
        assert_eq!(money("-5").unwrap().cents(), -500);
        assert_eq!(money("0").unwrap().cents(), 0);
    }

    #[test]
    fn money_refuses_what_it_cannot_hold_exactly() {
        assert!(money("1.234").is_err());
        assert!(money("99999999999999999999").is_err());
        assert!(money("1e300").is_err());
        assert!(money("\"12.50\"").is_err());
    }

    #[test]
    fn money_survives_a_round_trip_for_every_cent_in_a_wide_range() {
        for cents in -200_000..=200_000 {
            let value = Money::from_cents(cents).unwrap();
            let json = serde_json::to_string(&value).unwrap();
            assert_eq!(money(&json).unwrap(), value, "{json}");
        }
    }

    #[test]
    fn money_is_sent_as_a_decimal_number() {
        let json = |cents| serde_json::to_string(&Money::from_cents(cents).unwrap()).unwrap();
        assert_eq!(json(5446), "54.46");
        assert_eq!(json(1250), "12.5");
        assert_eq!(json(-500), "-5.0");
    }

    #[test]
    fn money_has_a_limit_and_prints_euros() {
        assert!(Money::from_cents(Money::MAX_CENTS).is_ok());
        assert!(Money::from_cents(Money::MAX_CENTS + 1).is_err());
        assert!(Money::from_cents(-Money::MAX_CENTS - 1).is_err());
        assert!(Money::from_euros(i64::MAX).is_err());
        assert_eq!(Money::from_cents(-1205).unwrap().to_string(), "-12.05");
        assert_eq!(Money::from_euros(3).unwrap().to_string(), "3.00");
    }

    #[test]
    fn date_times_round_trip() {
        let value: DateTime = "2026-11-09 13:47:28".parse().unwrap();
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            r#""2026-11-09 13:47:28""#
        );
        assert!("2026-11-09".parse::<DateTime>().is_err());
    }

    #[test]
    fn dates_round_trip() {
        let value: Date = "2027-01-15".parse().unwrap();
        assert_eq!(serde_json::to_string(&value).unwrap(), r#""2027-01-15""#);
        assert!("2027-13-01".parse::<Date>().is_err());
        assert!("2027-01-15 10:00:00".parse::<Date>().is_err());
    }

    #[test]
    fn units_read_what_the_api_stores() {
        assert_eq!(serde_json::from_str::<Grams>("-1").unwrap().get(), -1);
        assert_eq!(Grams::new(250).get(), 250);
        assert_eq!(Centimeters::new(12).get(), 12);
    }

    #[test]
    fn percentages_are_read_as_numbers() {
        assert_eq!(
            serde_json::from_str::<Percentage>("21").unwrap().get(),
            21.0
        );
        assert_eq!(
            serde_json::from_str::<Percentage>("8.5").unwrap().get(),
            8.5
        );
        assert!(Percentage::new(f64::NAN).is_err());
    }

    #[test]
    fn email_addresses_have_a_local_part_and_a_domain() {
        assert!(EmailAddress::new("jane@example.com").is_ok());
        for refused in ["nope", "@example.com", "jane@", "ja ne@example.com", ""] {
            assert!(EmailAddress::new(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn names_and_codes_are_not_empty() {
        assert!(ProductCode::new("A-1").is_ok());
        assert!(ProductCode::new("").is_err());
        assert!(ProductName::new("").is_err());
        assert!(MidCode::new("x".repeat(16)).is_err());
        assert!(MidCode::new("x".repeat(15)).is_ok());
    }

    #[test]
    fn non_empty_enforces_both_bounds() {
        assert!(NonEmpty::<u8, 2>::new(vec![]).is_err());
        assert!(NonEmpty::<u8, 2>::new(vec![1, 2]).is_ok());
        assert!(NonEmpty::<u8, 2>::new(vec![1, 2, 3]).is_err());
    }
}
