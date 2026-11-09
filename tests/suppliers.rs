mod common;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::country::CountryCode;
use picqer_rs::customers::Language;
use picqer_rs::suppliers::{NewSupplier, SupplierChanges, SupplierName};
use picqer_rs::types::{EmailAddress, SupplierId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn suppliers_are_listed_and_fetched() {
    let (server, client) = setup().await;
    Mock::given(path("/suppliers"))
        .respond_with(respond("suppliers"))
        .mount(&server)
        .await;
    Mock::given(path("/suppliers/229503"))
        .respond_with(respond("supplier"))
        .mount(&server)
        .await;

    let all = client.suppliers().list_all().await.unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].country, Some(Reported::Known(CountryCode::NL)));
    assert_eq!(all[0].language, Some(Reported::Known(Language::Nl)));
    let one = client.suppliers().get(SupplierId(229_503)).await.unwrap();
    assert_eq!(one.name, "Example Supplier");
    assert_eq!(one.country, Some(Reported::Known(CountryCode::BE)));
    assert_eq!(one.language, Some(Reported::Known(Language::En)));
    assert_eq!(one.emailaddress.as_deref(), Some("supplier@example.com"));
}

#[tokio::test]
async fn what_picqer_stores_is_read_even_when_it_would_not_be_sent() {
    let (server, client) = setup().await;
    let mut supplier = common::json("supplier");
    supplier["country"] = json!("NLD");
    supplier["language"] = json!("de");
    Mock::given(path("/suppliers/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(supplier))
        .mount(&server)
        .await;

    let supplier = client.suppliers().get(SupplierId(1)).await.unwrap();
    assert!(matches!(supplier.country, Some(Reported::Unknown(_))));
    assert!(matches!(supplier.language, Some(Reported::Unknown(_))));
}

#[tokio::test]
async fn a_supplier_is_created_and_changed_with_only_what_was_set() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/suppliers"))
        .and(body_json(json!({
            "name": "Example Supplier",
            "country": "BE",
            "emailaddress": "supplier@example.com",
            "language": "en"
        })))
        .respond_with(respond("supplier"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/suppliers/5"))
        .and(body_json(json!({"name": "Renamed", "remarks": "r"})))
        .respond_with(respond("supplier"))
        .expect(1)
        .mount(&server)
        .await;

    let new = NewSupplier::new(SupplierName::new("Example Supplier").unwrap())
        .country(CountryCode::BE)
        .emailaddress(EmailAddress::new("supplier@example.com").unwrap())
        .language(Language::En);
    client.suppliers().create(&new).await.unwrap();
    let changes = SupplierChanges::new()
        .name(SupplierName::new("Renamed").unwrap())
        .remarks("r".to_owned());
    client
        .suppliers()
        .update(SupplierId(5), &changes)
        .await
        .unwrap();
}
