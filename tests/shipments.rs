mod common;

use common::{created, error, respond, setup};
use picqer_rs::Reported;
use picqer_rs::pagination::Page;
use picqer_rs::shipments::{
    CarrierKey, DocumentFormat, DocumentSize, DocumentType, NewParcel, NewShipment, ShipmentQuery,
};
use picqer_rs::types::{
    Grams, NonEmpty, PackagingId, PicklistId, ShipmentId, ShippingProviderProfileId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn shipments_are_listed_with_their_parcels_and_documents() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/shipments"))
        .respond_with(respond("shipments"))
        .expect(1)
        .mount(&server)
        .await;

    let shipments = client
        .shipments()
        .list(&ShipmentQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(shipments.len(), 3);
    let shipment = &shipments[0];
    assert!(!shipment.cancelled);
    assert_eq!(
        shipment.carrier_key,
        Some(Reported::Known(CarrierKey::Postal))
    );
    assert_eq!(shipment.provider.as_deref(), Some("Manual"));
    assert_eq!(shipment.idreturn, None);
    assert_eq!(shipment.weight, Some(Grams::new(900)));
    assert_eq!(shipment.idpackaging, Some(PackagingId(3132)));
    let parcel = &shipment.parcels[0];
    assert_eq!(parcel.tracking_code.as_deref(), Some("ABC"));
    assert_eq!(
        parcel.tracking_url.as_deref(),
        Some("https://example.com/t/ABC")
    );
    let document = &parcel.documents[0];
    assert_eq!(
        document.document_type,
        Some(Reported::Known(DocumentType::Label))
    );
    assert_eq!(document.size, Some(Reported::Known(DocumentSize::A6)));
    assert_eq!(document.format, Some(Reported::Known(DocumentFormat::Pdf)));
    assert!(
        document
            .url
            .as_ref()
            .unwrap()
            .as_str()
            .ends_with("label.pdf")
    );
    let without_tracking = &shipments[1];
    assert_eq!(without_tracking.parcels[0].tracking_code, None);
    assert_eq!(without_tracking.idpackaging, None);
}

#[tokio::test]
async fn filters_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/shipments"))
        .and(query_param("search", "ABC"))
        .and(query_param("sincedate", "2026-11-09 17:20:00"))
        .and(query_param("untildate", "2026-11-10 00:00:00"))
        .respond_with(respond("shipments"))
        .expect(1)
        .mount(&server)
        .await;

    let query = ShipmentQuery::new()
        .search("ABC")
        .since("2026-11-09 17:20:00".parse().unwrap())
        .until("2026-11-10 00:00:00".parse().unwrap());
    client.shipments().list(&query, Page::FIRST).await.unwrap();
}

#[tokio::test]
async fn a_shipment_and_the_shipments_of_a_picklist_are_fetched() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/shipments/9"))
        .respond_with(respond("shipment"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/picklists/5/shipments"))
        .respond_with(respond("shipments"))
        .mount(&server)
        .await;

    let shipment = client.shipments().get(ShipmentId(9)).await.unwrap();
    assert_eq!(shipment.idpicklist, Some(PicklistId(198029214)));
    assert!(shipment.created.is_some());
    assert!(shipment.updated.is_some());
    let all = client
        .shipments()
        .for_picklist(PicklistId(5))
        .await
        .unwrap();
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn the_shipping_methods_of_a_picklist_say_what_they_ask_for() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picklists/5/shippingmethods"))
        .respond_with(respond("shipping_methods"))
        .mount(&server)
        .await;

    let methods = client.shipments().methods(PicklistId(5)).await.unwrap();
    assert_eq!(methods.len(), 2);
    assert_eq!(methods[0].selected, Some(true));
    assert_eq!(methods[0].multi_colli_supported, Some(true));
    assert!(methods[0].extra_fields.is_empty());
    let manual = &methods[1];
    assert_eq!(
        manual.idshippingprovider_profile,
        ShippingProviderProfileId(53795)
    );
    assert_eq!(manual.multi_colli_supported, Some(false));
    assert_eq!(manual.extra_fields[0].key, "code");
    assert_eq!(manual.extra_fields[0].is_required, Some(false));
    assert_eq!(manual.extra_fields[0].default_value, None);
    assert_eq!(manual.extra_fields[1].key, "tracking_url");
}

#[tokio::test]
async fn a_shipment_is_created_and_returned() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picklists/5/shipments"))
        .and(query_param("return", "shipment"))
        .and(body_json(json!({
            "idshippingprofile": 53795,
            "idpackaging": 3132,
            "weight": 900,
            "extrafields": {"code": "ABC", "tracking_url": "https://example.com/t/ABC"}
        })))
        .respond_with(created("shipment"))
        .expect(1)
        .mount(&server)
        .await;

    let shipment = NewShipment::new(ShippingProviderProfileId(53795))
        .idpackaging(PackagingId(3132))
        .weight(Grams::new(900))
        .extrafield("code", "ABC")
        .extrafield("tracking_url", "https://example.com/t/ABC");
    let made = client
        .shipments()
        .create(PicklistId(5), &shipment)
        .await
        .unwrap();
    assert_eq!(made.parcels[0].tracking_code.as_deref(), Some("ABC"));
}

#[tokio::test]
async fn parcels_are_sent_as_the_documentation_shows() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picklists/5/shipments"))
        .and(body_json(json!({
            "idshippingprofile": 1,
            "parcels": [
                {"idpackaging": 3132, "weight": 500},
                {"idpackaging": 3132, "weight": 600}
            ]
        })))
        .respond_with(created("shipment"))
        .expect(1)
        .mount(&server)
        .await;

    let shipment = NewShipment::new(ShippingProviderProfileId(1)).parcels(
        NonEmpty::new(vec![
            NewParcel::new(PackagingId(3132), Grams::new(500)),
            NewParcel::new(PackagingId(3132), Grams::new(600)),
        ])
        .unwrap(),
    );
    client
        .shipments()
        .create(PicklistId(5), &shipment)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_failing_provider_comes_back_as_an_api_error_with_the_code() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picklists/5/shipments"))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(json!({
                "error": true,
                "error_code": 26,
                "error_message": "Cannot create shipment: The selected shipping provider does not support multiple colli"
            })),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/shipments/1"))
        .respond_with(error(404, "error_not_found"))
        .mount(&server)
        .await;

    let failed = client
        .shipments()
        .create(
            PicklistId(5),
            &NewShipment::new(ShippingProviderProfileId(1)),
        )
        .await
        .unwrap_err();
    assert_eq!(
        failed.as_api().unwrap().code,
        Some(Reported::Known(picqer_rs::ErrorCode::CannotCreateShipment))
    );
    assert!(
        client
            .shipments()
            .get(ShipmentId(1))
            .await
            .unwrap_err()
            .is_not_found()
    );
}
