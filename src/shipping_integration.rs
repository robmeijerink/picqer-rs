//! The protocol of custom and external shipping providers.
//!
//! When you connect your own shipping provider to Picqer, Picqer sends requests to endpoints of
//! yours and expects answers in a fixed format, version `1.0`. This module has the types for both
//! sides: what you answer is built with validating constructors and serialized, what Picqer sends
//! is read with [`ShipmentRequest::from_slice`].
//!
//! There are two endpoints. The optional configuration endpoint answers a `GET` with
//! [`ConfigOptions`], which become the options of a shipping profile. The shipment endpoint receives
//! a `POST` with a [`ShipmentRequest`] and answers with a [`ShipmentResult`] within 20 seconds, or
//! with a status in the 4xx range and [`ShipmentErrors`], whose messages Picqer shows to the user.
//! Both use HTTP Basic authentication and HTTPS. Nothing here calls Picqer: the account this crate
//! was checked against has no endpoint of its own, so these types follow the documentation and its
//! examples.
//!
//! The older "JSON push" method is not covered: Picqer recommends this protocol for new
//! integrations.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::customers::Language;
use crate::error::{Error, Result, ValidationError};
use crate::reported::Reported;
use crate::shipments::{CarrierKey, DocumentFormat, DocumentSize};
use crate::types::{Date, Grams, Money, PackagingId};

/// The version of the protocol, which every request and answer carries.
pub const PROTOCOL_VERSION: &str = "1.0";

bounded_string! {
    /// The key of a configuration option, with which Picqer sends the choice back.
    OptionKey, "option key", 1, usize::MAX
}

bounded_string! {
    /// The text Picqer shows for a configuration option or one of its values.
    OptionLabel, "option label", 1, usize::MAX
}

bounded_string! {
    /// A message about an error that Picqer shows to the user.
    ErrorMessage, "error message", 1, usize::MAX
}

string_enum! {
    /// How the user fills in a configuration option.
    pub enum OptionType {
        /// A free text field.
        Text = "text",
        /// A choice from a list.
        List = "list",
    }
}

/// One value the user can choose for a list option.
///
/// The documentation does not say what type a value has. Its examples use whole numbers, so that is
/// what the examples of this crate use, and any JSON value is accepted.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PossibleValue {
    value: Value,
    label: OptionLabel,
}

impl PossibleValue {
    /// A value with the text shown for it.
    pub fn new(value: impl Into<Value>, label: OptionLabel) -> Self {
        Self {
            value: value.into(),
            label,
        }
    }
}

/// A configuration option of a shipping profile.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConfigItem {
    key: OptionKey,
    label: OptionLabel,
    description: Option<String>,
    #[serde(rename = "type")]
    option_type: OptionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    possible_values: Option<Vec<PossibleValue>>,
    default_value: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
}

impl ConfigItem {
    /// A free text option.
    pub fn text(key: OptionKey, label: OptionLabel) -> Self {
        Self {
            key,
            label,
            description: None,
            option_type: OptionType::Text,
            possible_values: None,
            default_value: Value::Null,
            required: None,
        }
    }

    /// An option with a list of values to choose from, which needs at least one.
    pub fn list(
        key: OptionKey,
        label: OptionLabel,
        possible_values: Vec<PossibleValue>,
    ) -> std::result::Result<Self, ValidationError> {
        if possible_values.is_empty() {
            return Err(ValidationError::new(
                "possible values",
                "a list option needs at least one value",
            ));
        }
        Ok(Self {
            possible_values: Some(possible_values),
            option_type: OptionType::List,
            ..Self::text(key, label)
        })
    }

    /// Explains the option to the user.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The value Picqer uses when the user chooses nothing.
    pub fn default_value(mut self, value: impl Into<Value>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Marks the option as required or not. The documentation uses this in an example without
    /// describing it.
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }
}

/// The answer of the configuration endpoint.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConfigOptions {
    version: &'static str,
    data: ConfigData,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ConfigData {
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<ConfigFlags>,
    items: Vec<ConfigItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ConfigFlags {
    multi_colli_supported: bool,
}

impl ConfigOptions {
    /// The options of a shipping profile.
    pub fn new(items: Vec<ConfigItem>) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            data: ConfigData {
                options: None,
                items,
            },
        }
    }

    /// Tells Picqer whether the provider can make a shipment of several parcels, each with its own
    /// label and tracking code. Without this Picqer sends one parcel.
    pub fn multi_colli_supported(mut self, supported: bool) -> Self {
        self.data.options = Some(ConfigFlags {
            multi_colli_supported: supported,
        });
        self
    }
}

/// The choice of the user for one option of the profile.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ProfileOption {
    /// The key of the option.
    pub key: String,
    /// What was chosen. The documentation shows whole numbers and `null`, and does not say more.
    pub value: Value,
}

/// A party of a shipment.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Party {
    /// The name.
    pub name: String,
    /// The contact person.
    pub contactname: Option<String>,
    /// The first line of the address.
    pub address: Option<String>,
    /// The second line of the address.
    pub address2: Option<String>,
    /// The zip code.
    pub zipcode: Option<String>,
    /// The city.
    pub city: Option<String>,
    /// The region.
    pub region: Option<String>,
    /// The country code. The documentation shows it in lower case in one example and in upper case
    /// in another, so it is not checked.
    pub country: Option<String>,
    /// The email address.
    pub emailaddress: Option<String>,
    /// The telephone number.
    pub telephone: Option<String>,
}

/// The references of a shipment, all optional.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ShipmentReferences {
    /// The reference to send to the receiver.
    pub reference: Option<String>,
    /// The order.
    pub idorder: Option<i64>,
    /// The return.
    pub idreturn: Option<i64>,
    /// The picklist.
    pub idpicklist: Option<i64>,
    /// The order number.
    pub orderid: Option<String>,
    /// The return number.
    pub returnid: Option<String>,
    /// The picklist number.
    pub picklistid: Option<String>,
}

/// A parcel to make a label for.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RequestedParcel {
    /// The packaging.
    pub idpackaging: Option<PackagingId>,
    /// The weight in grams.
    pub weight: Option<Grams>,
    /// The length of the packaging in centimetres.
    pub length: Option<i64>,
    /// The width of the packaging in centimetres.
    pub width: Option<i64>,
    /// The height of the packaging in centimetres.
    pub height: Option<i64>,
}

/// A product that goes through customs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomsProduct {
    /// The description.
    pub description: String,
    /// How many there are.
    pub amount: i64,
    /// The weight in grams.
    pub weight: Option<Grams>,
    /// The value of one.
    pub value: Option<Money>,
    /// The harmonized system code.
    pub hs_code: Option<String>,
    /// The country of origin, as sent.
    pub country_of_origin: Option<String>,
    /// The manufacturer identification code.
    pub mid_code: Option<String>,
}

/// What customs needs. Picqer sends it when the recipient is outside the EU customs union.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomsData {
    /// The products of the shipment.
    #[serde(default)]
    pub products: Vec<CustomsProduct>,
}

/// The shipment to make a label for.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct RequestedShipment {
    /// The weight in grams. Only the example with customs information has it.
    pub weight: Option<Grams>,
    /// What the customer paid for the products, excluding VAT.
    pub total_paid: Option<Money>,
    /// The purchase value of the products, excluding VAT, often used for insurance.
    pub value_of_goods: Option<Money>,
    /// The recipient.
    pub recipient: Party,
    /// The references.
    pub references: Option<ShipmentReferences>,
    /// The delivery date the customer asked for.
    pub preferred_delivery_date: Option<Date>,
    /// What depends on the carrier. The documentation says an array or an object, and that it is an
    /// empty array when there is none, which differs from the orders endpoint.
    #[serde(default)]
    pub pickup_point_data: Value,
    /// The language to talk to the customer in.
    pub language: Option<Reported<Language>>,
    /// The parcels: more than one only when the provider supports that. The example with customs
    /// information has none.
    #[serde(default)]
    pub parcels: Vec<RequestedParcel>,
    /// The customs information.
    pub customs_data: Option<CustomsData>,
}

/// The data of a request to make a shipment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ShipmentRequestData {
    /// What the user chose for each option of the profile.
    #[serde(default)]
    pub profile: Vec<ProfileOption>,
    /// The shipment.
    pub shipment: RequestedShipment,
}

/// What Picqer sends to the shipment endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct ShipmentRequest {
    /// The version of the protocol.
    pub version: String,
    /// The data.
    pub data: ShipmentRequestData,
}

impl ShipmentRequest {
    /// Reads the body of a request.
    pub fn from_slice(body: &[u8]) -> Result<Self> {
        serde_json::from_slice(body).map_err(|source| Error::Decode {
            source,
            body: crate::error::truncate(&String::from_utf8_lossy(body)),
        })
    }

    /// The choice for an option of the profile.
    pub fn option(&self, key: &str) -> Option<&Value> {
        self.data
            .profile
            .iter()
            .find(|option| option.key == key)
            .map(|option| &option.value)
    }
}

string_enum! {
    /// The kind of a document in an answer. Picqer only supports these two.
    pub enum ResultDocumentType {
        /// A shipping label.
        Label = "label",
        /// A customs document.
        Customs = "customs",
    }
}

/// A document of a parcel: a label, or a customs document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultDocument {
    #[serde(rename = "type")]
    document_type: ResultDocumentType,
    size: DocumentSize,
    format: DocumentFormat,
    content: String,
}

impl ResultDocument {
    /// A document with the bytes of the file, which are encoded as Picqer expects. Picqer only
    /// supports `A4` for what does not fit on `A6`, such as customs documents.
    pub fn new(
        document_type: ResultDocumentType,
        size: DocumentSize,
        format: DocumentFormat,
        content: &[u8],
    ) -> std::result::Result<Self, ValidationError> {
        if content.is_empty() {
            return Err(ValidationError::new("document", "must not be empty"));
        }
        Ok(Self {
            document_type,
            size,
            format,
            content: STANDARD.encode(content),
        })
    }

    /// The kind of document.
    pub fn document_type(&self) -> ResultDocumentType {
        self.document_type
    }

    /// The paper size.
    pub fn size(&self) -> DocumentSize {
        self.size
    }

    /// The file format.
    pub fn format(&self) -> DocumentFormat {
        self.format
    }

    /// The content, base64 encoded.
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// A parcel in the answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultParcel {
    tracking_code: Option<String>,
    tracking_url: Option<String>,
    #[serde(deserialize_with = "flat_or_nested")]
    documents: Vec<ResultDocument>,
}

fn flat_or_nested<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Vec<ResultDocument>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Item {
        One(ResultDocument),
        Many(Vec<ResultDocument>),
    }

    let mut documents = Vec::new();
    for item in Vec::<Item>::deserialize(deserializer)? {
        match item {
            Item::One(document) => documents.push(document),
            Item::Many(many) => documents.extend(many),
        }
    }
    Ok(documents)
}

impl ResultParcel {
    /// A parcel with its documents, which must include the label as a PDF: Picqer always needs it.
    pub fn new(documents: Vec<ResultDocument>) -> std::result::Result<Self, ValidationError> {
        let has_pdf_label = documents.iter().any(|document| {
            document.document_type == ResultDocumentType::Label
                && document.format == DocumentFormat::Pdf
        });
        if !has_pdf_label {
            return Err(ValidationError::new(
                "documents",
                "a parcel needs its label as a PDF",
            ));
        }
        Ok(Self {
            tracking_code: None,
            tracking_url: None,
            documents,
        })
    }

    /// The tracking code, optional but desired.
    pub fn tracking_code(mut self, code: impl Into<String>) -> Self {
        self.tracking_code = Some(code.into());
        self
    }

    /// The public status page of the parcel.
    pub fn tracking_url(mut self, url: impl Into<String>) -> Self {
        self.tracking_url = Some(url.into());
        self
    }

    /// The documents of the parcel.
    pub fn documents(&self) -> &[ResultDocument] {
        &self.documents
    }
}

/// The data of an answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ResultData {
    carrier_key: Option<Reported<CarrierKey>>,
    carrier_name: Option<String>,
    parcels: Vec<ResultParcel>,
}

/// The answer of the shipment endpoint when the shipment was made.
///
/// The documents of a parcel are written as a list of documents, as the documentation describes
/// them. Its example with customs information has a list of lists for them, so reading accepts both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipmentResult {
    version: String,
    data: ResultData,
}

impl ShipmentResult {
    /// An answer with the parcels, which needs at least one.
    pub fn new(parcels: Vec<ResultParcel>) -> std::result::Result<Self, ValidationError> {
        if parcels.is_empty() {
            return Err(ValidationError::new(
                "parcels",
                "an answer needs at least one parcel",
            ));
        }
        Ok(Self {
            version: PROTOCOL_VERSION.to_owned(),
            data: ResultData {
                carrier_key: None,
                carrier_name: None,
                parcels,
            },
        })
    }

    /// The carrier, so Picqer can map the shipping provider to stores and marketplaces.
    pub fn carrier_key(mut self, key: CarrierKey) -> Self {
        self.data.carrier_key = Some(Reported::Known(key));
        self
    }

    /// The name of the provider as the end customer sees it.
    pub fn carrier_name(mut self, name: impl Into<String>) -> Self {
        self.data.carrier_name = Some(name.into());
        self
    }

    /// The parcels.
    pub fn parcels(&self) -> &[ResultParcel] {
        &self.data.parcels
    }
}

/// The answer with a status in the 4xx range, whose messages Picqer shows to the user. A status in
/// the 5xx range gets a generic message in Picqer instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShipmentErrors {
    version: &'static str,
    error_messages: Vec<ErrorMessage>,
}

impl ShipmentErrors {
    /// An answer with at least one message.
    pub fn new(messages: Vec<ErrorMessage>) -> std::result::Result<Self, ValidationError> {
        if messages.is_empty() {
            return Err(ValidationError::new(
                "error messages",
                "at least one message is needed",
            ));
        }
        Ok(Self {
            version: PROTOCOL_VERSION,
            error_messages: messages,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn key(text: &str) -> OptionKey {
        OptionKey::new(text).unwrap()
    }

    fn label(text: &str) -> OptionLabel {
        OptionLabel::new(text).unwrap()
    }

    #[test]
    fn config_options_match_the_documented_example() {
        let options = ConfigOptions::new(vec![
            ConfigItem::list(
                key("shipping_method"),
                label("Shipping method"),
                vec![
                    PossibleValue::new(29, label("DHLForYou")),
                    PossibleValue::new(928, label("DPD Global")),
                ],
            )
            .unwrap()
            .default_value(928)
            .required(true),
            ConfigItem::text(key("pickup_instructions"), label("Pickup instructions")),
        ])
        .multi_colli_supported(true);
        assert_eq!(
            serde_json::to_value(&options).unwrap(),
            json!({
                "version": "1.0",
                "data": {
                    "options": {"multi_colli_supported": true},
                    "items": [
                        {
                            "key": "shipping_method", "label": "Shipping method",
                            "description": null, "type": "list",
                            "possible_values": [
                                {"value": 29, "label": "DHLForYou"},
                                {"value": 928, "label": "DPD Global"}
                            ],
                            "default_value": 928, "required": true
                        },
                        {
                            "key": "pickup_instructions", "label": "Pickup instructions",
                            "description": null, "type": "text", "default_value": null
                        }
                    ]
                }
            })
        );
        assert!(ConfigItem::list(key("a"), label("A"), vec![]).is_err());
    }

    #[test]
    fn a_request_is_read_with_its_options_and_parcels() {
        let body = json!({
            "version": "1.0",
            "data": {
                "profile": [
                    {"key": "shipping_method", "value": 29},
                    {"key": "pickup_instructions", "value": null}
                ],
                "shipment": {
                    "total_paid": 291.29, "value_of_goods": 199.99,
                    "recipient": {
                        "name": "Picqer B.V.", "contactname": null, "address": "Meander 251",
                        "address2": null, "zipcode": "6825MC", "city": "Arnhem",
                        "region": "Gelderland", "country": "nl",
                        "emailaddress": "info@picqer.com", "telephone": null
                    },
                    "references": {
                        "reference": "wintersjaal #8928283", "idorder": 2832123, "idreturn": null,
                        "idpicklist": 63832123, "orderid": "O2018-29230", "returnid": null,
                        "picklistid": "P2018-29382"
                    },
                    "preferred_delivery_date": "2018-09-09",
                    "pickup_point_data": [],
                    "language": "nl",
                    "parcels": [
                        {"idpackaging": 3314543, "weight": 29388, "length": 40, "width": 30, "height": 10}
                    ]
                }
            }
        });
        let request = ShipmentRequest::from_slice(body.to_string().as_bytes()).unwrap();
        assert_eq!(request.option("shipping_method"), Some(&json!(29)));
        assert_eq!(request.option("pickup_instructions"), Some(&Value::Null));
        assert_eq!(request.option("missing"), None);
        let shipment = &request.data.shipment;
        assert_eq!(shipment.total_paid.unwrap().cents(), 29129);
        assert_eq!(shipment.recipient.country.as_deref(), Some("nl"));
        assert_eq!(shipment.pickup_point_data, json!([]));
        assert_eq!(shipment.parcels[0].length, Some(40));
        assert_eq!(
            shipment.language.as_ref().unwrap().known(),
            Some(&Language::Nl)
        );
        assert!(shipment.customs_data.is_none());
    }

    #[test]
    fn customs_information_is_read_without_parcels() {
        let body = json!({
            "version": "1.0",
            "data": {
                "profile": [],
                "shipment": {
                    "weight": 29388,
                    "recipient": {"name": "Picqer B.V.", "country": "CH"},
                    "customs_data": {"products": [{
                        "description": "Philips Hue Duo Pack", "amount": 2, "weight": 350,
                        "value": 29.99, "hs_code": "288392", "country_of_origin": "CN",
                        "mid_code": null
                    }]}
                }
            }
        });
        let request = ShipmentRequest::from_slice(body.to_string().as_bytes()).unwrap();
        let customs = request.data.shipment.customs_data.unwrap();
        assert_eq!(customs.products[0].value.unwrap().cents(), 2999);
        assert_eq!(customs.products[0].mid_code, None);
        assert!(request.data.shipment.parcels.is_empty());
    }

    #[test]
    fn an_answer_needs_a_pdf_label_and_is_written_flat() {
        let pdf = ResultDocument::new(
            ResultDocumentType::Label,
            DocumentSize::A6,
            DocumentFormat::Pdf,
            b"%PDF",
        )
        .unwrap();
        let zpl = ResultDocument::new(
            ResultDocumentType::Label,
            DocumentSize::A6,
            DocumentFormat::Zpl,
            b"^XA",
        )
        .unwrap();
        assert!(ResultParcel::new(vec![zpl.clone()]).is_err());
        assert!(
            ResultDocument::new(
                ResultDocumentType::Label,
                DocumentSize::A6,
                DocumentFormat::Pdf,
                b""
            )
            .is_err()
        );
        let result = ShipmentResult::new(vec![
            ResultParcel::new(vec![pdf, zpl])
                .unwrap()
                .tracking_code("3S9393838-1")
                .tracking_url("https://example.com/3S9393838-1"),
        ])
        .unwrap()
        .carrier_key(CarrierKey::Postnl)
        .carrier_name("PostNL");
        let written = serde_json::to_value(&result).unwrap();
        assert_eq!(written["data"]["carrier_key"], "postnl");
        assert_eq!(
            written["data"]["parcels"][0]["documents"][0]["content"],
            "JVBERg=="
        );
        assert_eq!(
            written["data"]["parcels"][0]["documents"][1]["format"],
            "zpl"
        );
        assert_eq!(
            serde_json::from_value::<ShipmentResult>(written).unwrap(),
            result
        );
        assert!(ShipmentResult::new(vec![]).is_err());
    }

    #[test]
    fn an_answer_with_nested_documents_is_read() {
        let nested = json!({
            "version": "1.0",
            "data": {
                "carrier_key": "postnl", "carrier_name": "PostNL",
                "parcels": [{
                    "tracking_code": "3S9393838-1", "tracking_url": null,
                    "documents": [[
                        {"type": "label", "size": "A6", "format": "pdf", "content": "AAAA"},
                        {"type": "customs", "size": "A4", "format": "pdf", "content": "BBBB"}
                    ]]
                }]
            }
        });
        let result: ShipmentResult = serde_json::from_value(nested).unwrap();
        assert_eq!(result.parcels()[0].documents().len(), 2);
        assert_eq!(
            result.parcels()[0].documents()[1].document_type(),
            ResultDocumentType::Customs
        );
    }

    #[test]
    fn errors_are_written_with_their_messages() {
        let errors = ShipmentErrors::new(vec![
            ErrorMessage::new("Zip code invalid for country NL").unwrap(),
        ])
        .unwrap();
        assert_eq!(
            serde_json::to_value(&errors).unwrap(),
            json!({"version": "1.0", "error_messages": ["Zip code invalid for country NL"]})
        );
        assert!(ShipmentErrors::new(vec![]).is_err());
    }
}
