mod common;

use common::{respond, setup};
use picqer_rs::customers::Language;
use picqer_rs::fulfilment::{
    FulfilmentCustomerChanges, FulfilmentCustomerName, FulfilmentPassword, FulfilmentUsername,
    NewFulfilmentCustomer, ReportPeriod,
};
use picqer_rs::pagination::Page;
use picqer_rs::types::{FulfilmentCustomerId, ShippingProviderProfileId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn customers_are_listed_without_their_shipping_profiles() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/fulfilment/customers"))
        .respond_with(respond("fulfilment_customers"))
        .expect(1)
        .mount(&server)
        .await;

    let customers = client
        .fulfilment_customers()
        .list(Page::FIRST)
        .await
        .unwrap();
    assert_eq!(customers.len(), 2);
    assert!(customers[0].allowed_shipping_profiles.is_empty());
    assert_eq!(customers[1].username, None);
    assert_eq!(customers[1].idtemplate, None);
}

#[tokio::test]
async fn a_customer_has_its_profiles_and_last_login() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/fulfilment/customers/11014"))
        .respond_with(respond("fulfilment_customer"))
        .expect(1)
        .mount(&server)
        .await;

    let customer = client
        .fulfilment_customers()
        .get(FulfilmentCustomerId(11014))
        .await
        .unwrap();
    assert_eq!(customer.name, "Fulfilling Partners");
    assert_eq!(customer.language.unwrap().known(), Some(&Language::Nl));
    assert_eq!(
        customer.allowed_shipping_profiles,
        vec![ShippingProviderProfileId(53795)]
    );
    assert_eq!(
        customer.last_login_at.unwrap().to_string(),
        "2026-11-02 09:30:00"
    );
}

#[tokio::test]
async fn creating_sends_the_language_and_the_login() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/fulfilment/customers"))
        .and(body_json(json!({
            "name": "Fulfilling Partners", "language": "nl", "can_login": true,
            "username": "partners", "password": "secret-pass"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            include_str!("fixtures/fulfilment_customer.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let new = NewFulfilmentCustomer::new(
        FulfilmentCustomerName::new("Fulfilling Partners").unwrap(),
        Language::Nl,
    )
    .can_login(true)
    .username(FulfilmentUsername::new("partners").unwrap())
    .password(&FulfilmentPassword::new("secret-pass").unwrap());
    let created = client.fulfilment_customers().create(&new).await.unwrap();
    assert_eq!(created.idfulfilment_customer, FulfilmentCustomerId(11014));
}

#[tokio::test]
async fn updating_sends_only_the_changes_and_nulls() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/fulfilment/customers/11014"))
        .and(body_json(json!({"name": "Renamed", "username": null})))
        .respond_with(respond("fulfilment_customer"))
        .expect(1)
        .mount(&server)
        .await;

    client
        .fulfilment_customers()
        .update(
            FulfilmentCustomerId(11014),
            &FulfilmentCustomerChanges::new()
                .name(FulfilmentCustomerName::new("Renamed").unwrap())
                .without_username(),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn a_report_covers_the_period() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/fulfilment/customers/11014/report"))
        .and(query_param("start-date", "2026-11-01"))
        .and(query_param("end-date", "2026-11-09"))
        .respond_with(respond("fulfilment_report"))
        .expect(1)
        .mount(&server)
        .await;

    let report = client
        .fulfilment_customers()
        .report(
            FulfilmentCustomerId(11014),
            ReportPeriod::new()
                .from("2026-11-01".parse().unwrap())
                .until("2026-11-09".parse().unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(
        report.inbound.unwrap().receipts_products_total_amount,
        Some(40)
    );
    let outbound = report.outbound.unwrap();
    assert_eq!(outbound.shipments_count, Some(3));
    assert!(outbound.shipments.is_empty());
    let storage = report.storage_usage.unwrap();
    assert_eq!(storage.stock_total, Some(12));
    assert_eq!(
        storage.stock_locations_in_use[0].total_items.as_deref(),
        Some("12")
    );
    assert_eq!(
        storage.stock_location_types_in_use[0].unique_locations,
        Some(1)
    );
    assert_eq!(report.returns.unwrap().returns_count, Some(0));
}
