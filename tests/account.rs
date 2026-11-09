mod common;

use common::{created, error, respond, setup};
use picqer_rs::Reported;
use picqer_rs::packagings::{NewPackaging, PackagingChanges, PackagingName, PackagingStatus};
use picqer_rs::types::{PackagingId, TemplateId, UserId};
use picqer_rs::users::UserQuery;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn shipping_providers_come_with_their_profiles() {
    let (server, client) = setup().await;
    Mock::given(path("/shippingproviders"))
        .respond_with(respond("shipping_providers"))
        .mount(&server)
        .await;

    let providers = client.shipping_providers().list_all().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].provider.as_deref(), Some("PostNL"));
    assert_eq!(providers[0].active, Some(true));
    assert_eq!(providers[0].profiles.len(), 1);
    assert_eq!(
        providers[0].profiles[0].name.as_deref(),
        Some("Testprofile")
    );
}

#[tokio::test]
async fn templates_are_read() {
    let (server, client) = setup().await;
    Mock::given(path("/templates"))
        .respond_with(respond("templates"))
        .mount(&server)
        .await;
    Mock::given(path("/templates/14879"))
        .respond_with(respond("template"))
        .mount(&server)
        .await;

    let all = client.templates().list_all().await.unwrap();
    assert_eq!(all[0].default, Some(true));
    assert_eq!(all[0].custom_css, None);
    let one = client.templates().get(TemplateId(14879)).await.unwrap();
    assert_eq!(one, all[0]);
}

#[tokio::test]
async fn users_have_rights_when_fetched_alone() {
    let (server, client) = setup().await;
    Mock::given(path("/users"))
        .and(query_param("active", "true"))
        .respond_with(respond("users"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/users/me"))
        .respond_with(respond("user"))
        .mount(&server)
        .await;
    Mock::given(path("/users/43968"))
        .respond_with(respond("user"))
        .mount(&server)
        .await;

    let users = client
        .users()
        .list_all(UserQuery::new().active_only())
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert!(users[0].rights.is_empty());
    assert_eq!(users[0].emailaddress.as_deref(), Some("alice@example.com"));
    let me = client.users().me().await.unwrap();
    assert_eq!(me.iduser, UserId(43968));
    assert_eq!(me.admin, Some(true));
    assert_eq!(me.rights.len(), 28);
    assert!(
        me.rights
            .iter()
            .any(|right| right.right == "orders.edit" && right.granted)
    );
    assert_eq!(client.users().get(UserId(43968)).await.unwrap(), me);
}

#[tokio::test]
async fn packagings_are_listed_by_status_and_fetched() {
    let (server, client) = setup().await;
    Mock::given(path("/packagings"))
        .and(query_param_is_missing("inactive"))
        .respond_with(respond("packagings"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/packagings"))
        .and(query_param("inactive", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/packagings/3132"))
        .respond_with(respond("packaging"))
        .mount(&server)
        .await;

    let packagings = client.packagings();
    let active = packagings.list_all(PackagingStatus::Active).await.unwrap();
    assert_eq!(active[0].name, "Karton");
    assert_eq!(active[0].barcode.as_deref(), Some("karton123"));
    assert_eq!(
        (active[0].length, active[0].width, active[0].height),
        (Some(33), Some(43), Some(120))
    );
    assert!(active[0].active);
    assert!(
        packagings
            .list_all(PackagingStatus::Inactive)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(packagings.get(PackagingId(3132)).await.unwrap(), active[0]);
}

#[tokio::test]
async fn a_packaging_is_created_and_changed_with_only_what_was_set() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/packagings"))
        .and(body_json(json!({
            "name": "Karton", "barcode": "karton123", "length": 33, "width": 43, "height": 120
        })))
        .respond_with(created("packaging"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/packagings/3132"))
        .and(body_json(json!({"barcode": null, "active": false})))
        .respond_with(respond("packaging"))
        .expect(1)
        .mount(&server)
        .await;

    let new = NewPackaging::new(PackagingName::new("Karton").unwrap())
        .barcode("karton123")
        .dimensions(33, 43, 120);
    client.packagings().create(&new).await.unwrap();
    let changes = PackagingChanges::new().without_barcode().active(false);
    client
        .packagings()
        .update(PackagingId(3132), &changes)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_validation_error_without_a_code_still_says_what_is_wrong() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/packagings"))
        .respond_with(error(422, "error_validation"))
        .mount(&server)
        .await;

    let new = NewPackaging::new(PackagingName::new("x").unwrap());
    let failure = client.packagings().create(&new).await.unwrap_err();
    let api = failure.as_api().unwrap();
    assert_eq!(api.status, 422);
    assert!(api.code.is_none());
    assert!(api.errors.is_some());
    assert!(failure.to_string().contains("naam"));
    let _: Option<Reported<picqer_rs::ErrorCode>> = None;
}

#[tokio::test]
async fn packing_stations_follow_the_documented_example() {
    use picqer_rs::packing_stations::PrinterFormat;
    use picqer_rs::types::PackingStationId;

    let (server, client) = setup().await;
    let station = json!({
        "idpacking_station": 1,
        "name": "Paktafel 1",
        "printer_packinglists": null,
        "printer_shipping_labels": null,
        "printer_shipping_documents": null,
        "printer_product_labels": {
            "idprinter": 1, "name": "Paktafel 1 barcode printer", "format": "custom",
            "custom_width": 95, "custom_height": 30, "zpl_support": false,
            "printnode_printerid": 10000001
        }
    });
    let other = json!({
        "idpacking_station": 2, "name": "Kantoor",
        "printer_packinglists": {"idprinter": 2, "name": "Kantoor printer", "format": "a6",
            "custom_width": null, "custom_height": null, "zpl_support": true,
            "printnode_printerid": 10000002}
    });
    Mock::given(path("/packingstations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([station, other])))
        .mount(&server)
        .await;
    Mock::given(path("/packingstations/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(station))
        .mount(&server)
        .await;

    let all = client.packing_stations().list_all().await.unwrap();
    assert_eq!(all.len(), 2);
    assert!(all[0].printer_packinglists.is_none());
    let label_printer = all[0].printer_product_labels.as_ref().unwrap();
    assert_eq!(
        label_printer.format,
        Some(Reported::Known(PrinterFormat::Custom))
    );
    assert_eq!(label_printer.custom_width, Some(95));
    assert!(matches!(
        all[1].printer_packinglists.as_ref().unwrap().format,
        Some(Reported::Unknown(_))
    ));
    let one = client
        .packing_stations()
        .get(PackingStationId(1))
        .await
        .unwrap();
    assert_eq!(one, all[0]);
}
