//! What your own endpoint does when Picqer calls it as a custom or external shipping provider:
//! describe the options, read a shipment request and answer with labels or errors.
//!
//! The functions take and return bytes, so they fit in any web framework. `main` runs them with
//! made-up input and does not contact Picqer.
//!
//! ```sh
//! cargo run --example shipping_provider_endpoint
//! ```

use picqer_rs::shipments::{DocumentFormat, DocumentSize};
use picqer_rs::shipping_integration::{
    ConfigItem, ConfigOptions, ErrorMessage, OptionKey, OptionLabel, ResultDocument,
    ResultDocumentType, ResultParcel, ShipmentErrors, ShipmentRequest, ShipmentResult,
};

fn config_options() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let options = ConfigOptions::new(vec![
        ConfigItem::text(
            OptionKey::new("service_point")?,
            OptionLabel::new("Service point")?,
        )
        .description("The code of the service point")
        .required(true),
    ])
    .multi_colli_supported(false);
    Ok(serde_json::to_vec(&options)?)
}

fn make_shipment(body: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let request = ShipmentRequest::from_slice(body)?;
    let Some(service_point) = request.option("service_point") else {
        let errors = ShipmentErrors::new(vec![ErrorMessage::new("The service point is missing")?])?;
        return Ok(serde_json::to_vec(&errors)?);
    };
    println!("service point {service_point}");

    let label = ResultDocument::new(
        ResultDocumentType::Label,
        DocumentSize::A6,
        DocumentFormat::Pdf,
        b"%PDF-1.4 the bytes of a real label",
    )?;
    let parcel = ResultParcel::new(vec![label])?
        .tracking_code("TRACK123")
        .tracking_url("https://example.com/track/TRACK123");
    Ok(serde_json::to_vec(&ShipmentResult::new(vec![parcel])?)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", String::from_utf8(config_options()?)?);

    let request = |profile: serde_json::Value| {
        serde_json::json!({
            "version": "1.0",
            "data": {
                "profile": profile,
                "shipment": {
                    "recipient": {
                        "name": "Picqer B.V.", "address": "Meander 251", "zipcode": "6825MC",
                        "city": "Arnhem", "country": "nl"
                    },
                    "parcels": [{"idpackaging": 3314543, "weight": 29388}]
                }
            }
        })
        .to_string()
    };
    let with = request(serde_json::json!([{"key": "service_point", "value": "NL123"}]));
    let without = request(serde_json::json!([]));
    println!("{}", String::from_utf8(make_shipment(with.as_bytes())?)?);
    println!("{}", String::from_utf8(make_shipment(without.as_bytes())?)?);
    Ok(())
}
