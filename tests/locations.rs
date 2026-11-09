mod common;

use common::{created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::location_stock_history::{LocationChangeType, LocationStockHistoryQuery};
use picqer_rs::location_types::{LocationTypeChanges, LocationTypeName, NewLocationType};
use picqer_rs::locations::{
    LocationChanges, LocationKind, LocationName, LocationQuery, NewContainer, NewLocation,
};
use picqer_rs::pagination::Page;
use picqer_rs::tags::HexColor;
use picqer_rs::types::{
    LocationId, LocationStockHistoryId, LocationTypeId, Money, ProductId, UserId, WarehouseId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn location_types_are_listed_fetched_made_changed_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/location_types"))
        .respond_with(respond("location_types"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/location_types/9380"))
        .respond_with(respond("location_type"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/location_types"))
        .and(body_json(json!({"name": "Bulk", "color": "#32CD32"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"idlocation_type": 5, "name": "Bulk", "default": null, "color": "#32CD32"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/location_types/5"))
        .and(body_json(json!({"color": "#00FF00"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"idlocation_type": 5, "name": "Bulk", "default": false, "color": "#00FF00"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/location_types/5/default"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"idlocation_type": 5, "name": "Bulk", "default": true, "color": "#00FF00"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/location_types/5"))
        .respond_with(ResponseTemplate::new(200).set_body_string("null"))
        .expect(1)
        .mount(&server)
        .await;

    let types = client.location_types();
    let all = types.list_all().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].default, Some(true));
    assert_eq!(all[0].color, "#0000f0");
    assert_eq!(types.get(LocationTypeId(9380)).await.unwrap(), all[0]);

    let made = types
        .create(&NewLocationType::new(
            LocationTypeName::new("Bulk").unwrap(),
            HexColor::new("#32CD32").unwrap(),
        ))
        .await
        .unwrap();
    assert_eq!(made.default, None);
    let changed = types
        .update(
            made.idlocation_type,
            &LocationTypeChanges::new().color(HexColor::new("#00FF00").unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(changed.default, Some(false));
    assert_eq!(
        types
            .set_default(made.idlocation_type)
            .await
            .unwrap()
            .default,
        Some(true)
    );
    types.delete(made.idlocation_type).await.unwrap();
}

#[tokio::test]
async fn locations_and_containers_are_read_with_what_they_have() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/locations"))
        .and(query_param_is_missing("type"))
        .respond_with(respond("locations"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/locations/12816667"))
        .respond_with(respond("location"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/locations/12818099"))
        .respond_with(respond("container"))
        .mount(&server)
        .await;

    let all = client
        .locations()
        .list(&LocationQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(all.len(), 4);
    let nested = &all[1];
    assert_eq!(nested.kind, Some(Reported::Known(LocationKind::Location)));
    assert_eq!(nested.parent_idlocation, Some(LocationId(12816666)));
    assert_eq!(nested.unlink_on_empty, Some(false));
    assert_eq!(nested.location_type.as_ref().unwrap().name, "Standaard");
    assert_eq!(all[0].parent_idlocation, None);
    let container = all
        .iter()
        .find(|l| l.kind == Some(Reported::Known(LocationKind::Container)))
        .unwrap();
    assert!(container.name.starts_with("Container"));
    assert_eq!(container.location_type, None);
    assert_eq!(container.is_bulk_location, None);

    assert_eq!(
        client.locations().get(LocationId(12816667)).await.unwrap(),
        *nested
    );
    assert_eq!(
        client.locations().get(LocationId(12818099)).await.unwrap(),
        *container
    );
}

#[tokio::test]
async fn filters_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/locations"))
        .and(query_param("type", "container"))
        .and(query_param("idwarehouse", "14153"))
        .and(query_param("name", "Container 1"))
        .and(query_param("idlocation_type", "9380"))
        .and(query_param("only_available", "true"))
        .respond_with(respond("locations"))
        .expect(1)
        .mount(&server)
        .await;

    let query = LocationQuery::new()
        .kind(LocationKind::Container)
        .idwarehouse(WarehouseId(14153))
        .name("Container 1")
        .idlocation_type(LocationTypeId(9380))
        .only_available();
    client.locations().list(&query, Page::FIRST).await.unwrap();
}

#[tokio::test]
async fn a_location_is_made_changed_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/locations"))
        .and(query_param_is_missing("auto_link_to_parent"))
        .and(body_json(json!({"name": "ZZ-probe", "idwarehouse": 14153})))
        .respond_with(created("location_created"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/locations"))
        .and(query_param("auto_link_to_parent", "1"))
        .and(body_json(json!({"name": "ZZ9.8.7", "idwarehouse": 14153})))
        .respond_with(created("location_created"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/locations"))
        .and(body_json(
            json!({"idwarehouse": 14153, "type": "container"}),
        ))
        .respond_with(created("container"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/locations/12818218"))
        .and(body_json(
            json!({"remarks": null, "parent_idlocation": 12816666}),
        ))
        .respond_with(respond("location"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/locations/12818218"))
        .respond_with(ResponseTemplate::new(200).set_body_string("null"))
        .expect(1)
        .mount(&server)
        .await;

    let locations = client.locations();
    let made = locations
        .create(&NewLocation::new(
            LocationName::new("ZZ-probe").unwrap(),
            WarehouseId(14153),
        ))
        .await
        .unwrap();
    assert_eq!(made.unlink_on_empty, None);
    assert_eq!(made.is_bulk_location, None);
    locations
        .create(
            &NewLocation::new(LocationName::new("ZZ9.8.7").unwrap(), WarehouseId(14153))
                .auto_link_to_parent(),
        )
        .await
        .unwrap();
    let container = locations
        .create_container(NewContainer::new(WarehouseId(14153)))
        .await
        .unwrap();
    assert_eq!(
        container.kind,
        Some(Reported::Known(LocationKind::Container))
    );
    locations
        .update(
            made.idlocation,
            &LocationChanges::new()
                .without_remarks()
                .parent_idlocation(LocationId(12816666)),
        )
        .await
        .unwrap();
    locations.delete(made.idlocation).await.unwrap();
}

#[tokio::test]
async fn the_products_of_a_location_and_the_intent_of_a_container() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/locations/12817277/products"))
        .respond_with(respond("location_products"))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/locations/7/products/52947932/intent"))
        .and(body_json(json!({"idlocation": 12817277})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"type": "to_location", "idlocation": 12817277})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/locations/7/products/52947932/intent"))
        .and(body_json(json!({"idlocation": null})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"type": "to_location", "idlocation": null})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let products = client
        .locations()
        .products_all(LocationId(12817277))
        .await
        .unwrap();
    assert_eq!(products.len(), 1);
    let product = &products[0];
    assert_eq!(product.idproduct, ProductId(52947932));
    assert!(product.price.unwrap() > Money::from_cents(0).unwrap());
    let stock = product.stock.as_ref().unwrap();
    assert_eq!(stock.idwarehouse, Some(WarehouseId(14153)));
    assert!(stock.freestock.is_some());
    assert_eq!(product.intent, None);

    let intent = client
        .locations()
        .set_intent(
            LocationId(7),
            ProductId(52947932),
            Some(LocationId(12817277)),
        )
        .await
        .unwrap();
    assert_eq!(intent.idlocation, Some(LocationId(12817277)));
    assert_eq!(
        intent.intent_type,
        Some(Reported::Known(
            picqer_rs::locations::IntentType::ToLocation
        ))
    );
    let cleared = client
        .locations()
        .set_intent(LocationId(7), ProductId(52947932), None)
        .await
        .unwrap();
    assert_eq!(cleared.idlocation, None);
}

#[tokio::test]
async fn the_stock_history_of_locations_is_read_with_its_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/location-stock-history"))
        .and(query_param("idproduct", "52947932"))
        .and(query_param("idlocation", "12817277"))
        .and(query_param("iduser", "1"))
        .and(query_param("sincedate", "2026-11-09 16:00:00"))
        .and(query_param("untildate", "2026-11-10 00:00:00"))
        .respond_with(respond("location_stock_history"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/location-stock-history/386101640"))
        .respond_with(respond("location_stock_history_entry"))
        .mount(&server)
        .await;

    let query = LocationStockHistoryQuery::new()
        .idproduct(ProductId(52947932))
        .idlocation(LocationId(12817277))
        .iduser(UserId(1))
        .since("2026-11-09 16:00:00".parse().unwrap())
        .until("2026-11-10 00:00:00".parse().unwrap());
    let entries = client
        .location_stock_history()
        .list(&query, Page::FIRST)
        .await
        .unwrap();
    assert_eq!(entries.len(), 5);
    let entry = &entries[0];
    assert_eq!(
        entry.change_type,
        Some(Reported::Known(LocationChangeType::Mutation))
    );
    assert_eq!(entry.contra_idlocation, None);
    assert_eq!(
        entry.new_stock,
        Some(entry.old_stock.unwrap() + entry.stock_change.unwrap())
    );
    assert_eq!(
        client
            .location_stock_history()
            .get(LocationStockHistoryId(386101640))
            .await
            .unwrap(),
        *entry
    );
}
