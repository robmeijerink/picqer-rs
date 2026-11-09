mod common;

use std::num::NonZeroI64;

use common::{json, respond, setup};
use picqer_rs::Reported;
use picqer_rs::pagination::Page;
use picqer_rs::stock::{StockReason, StockTarget, StockUpdate};
use picqer_rs::stock_history::{ChangeType, StockHistoryQuery};
use picqer_rs::types::{DateTime, LocationId, ProductId, StockHistoryId, WarehouseId};
use serde_json::{Value, json as value};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

fn reason() -> StockReason {
    StockReason::new("Purchased inventory").unwrap()
}

#[tokio::test]
async fn the_stock_of_a_product_is_listed_per_warehouse() {
    let (server, client) = setup().await;
    Mock::given(path("/products/5/stock"))
        .respond_with(respond("product_stock"))
        .mount(&server)
        .await;

    let stock = client.products().stock(ProductId(5)).await.unwrap();
    assert_eq!(stock.len(), 1);
    assert_eq!(stock[0].idwarehouse, WarehouseId(14153));
}

#[tokio::test]
async fn the_stock_in_a_warehouse_comes_with_its_locations() {
    let (server, client) = setup().await;
    Mock::given(path("/products/5/stock/14153"))
        .respond_with(respond("stock_detail"))
        .mount(&server)
        .await;

    let detail = client
        .products()
        .stock_in_warehouse(ProductId(5), WarehouseId(14153))
        .await
        .unwrap();
    assert_eq!(detail.level.stock, 14);
    assert_eq!(detail.level.freestock, 8);
    let location = &detail.locations[0];
    assert_eq!(location.name, "A.4.8.3");
    assert_eq!(location.reserved_picklists, 6);
    assert!(!location.is_bulk_location);
    assert!(location.lot_number.is_none() && location.expiration_date.is_none());
}

#[tokio::test]
async fn a_lot_and_expiration_date_are_read_when_traceability_reports_them() {
    let (server, client) = setup().await;
    let mut detail = json("stock_detail");
    detail["locations"][0]["lot_number"] = value!("LOT-2026-001");
    detail["locations"][0]["expiration_date"] = value!("2027-01-15");
    Mock::given(path("/products/5/stock/14153"))
        .respond_with(ResponseTemplate::new(200).set_body_json(detail))
        .mount(&server)
        .await;

    let detail = client
        .products()
        .stock_in_warehouse(ProductId(5), WarehouseId(14153))
        .await
        .unwrap();
    let location = &detail.locations[0];
    assert_eq!(location.lot_number.as_deref(), Some("LOT-2026-001"));
    assert_eq!(location.expiration_date.unwrap().to_string(), "2027-01-15");
}

#[tokio::test]
async fn stock_is_adjusted_and_set_with_the_documented_bodies() {
    let (server, client) = setup().await;
    for body in [
        value!({"idlocation": 5, "change": 6, "reason": "Purchased inventory"}),
        value!({"idlocation": null, "change": -2, "reason": "Purchased inventory"}),
        value!({"amount": 0, "reason": "Purchased inventory"}),
    ] {
        Mock::given(method("POST"))
            .and(path("/products/5/stock/14153"))
            .and(body_json(body))
            .respond_with(respond("stock_detail"))
            .expect(1)
            .mount(&server)
            .await;
    }

    let updates = [
        StockUpdate::adjust(
            StockTarget::Location(LocationId(5)),
            NonZeroI64::new(6).unwrap(),
            reason(),
        ),
        StockUpdate::adjust(
            StockTarget::NoSpecificLocation,
            NonZeroI64::new(-2).unwrap(),
            reason(),
        ),
        StockUpdate::set(0, reason()),
    ];
    for update in &updates {
        client
            .products()
            .update_stock(ProductId(5), WarehouseId(14153), update)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn the_stock_history_is_read_and_filtered() {
    let (server, client) = setup().await;
    Mock::given(path("/stockhistory"))
        .and(query_param("sincedate", "2026-11-09 14:00:00"))
        .and(query_param("idproduct", "7"))
        .respond_with(respond("stock_history"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/stockhistory/1"))
        .respond_with(respond("stock_history_item"))
        .mount(&server)
        .await;

    let since: DateTime = "2026-11-09 14:00:00".parse().unwrap();
    let query = StockHistoryQuery::new()
        .since(since)
        .idproduct(ProductId(7));
    let entries = client
        .stock_history()
        .list(&query, Page::FIRST)
        .await
        .unwrap();
    assert_eq!(entries.len(), 4);
    assert_eq!(
        entries[0].change_type,
        Some(Reported::Known(ChangeType::InitialImport))
    );
    let api = entries
        .iter()
        .find(|entry| entry.change_type == Some(Reported::Known(ChangeType::Api)))
        .unwrap();
    assert!(api.iduser.is_some());
    assert!(api.idlocation.is_none());
    assert_eq!(api.new_stock, api.old_stock + api.stock_change);

    let item = client.stock_history().get(StockHistoryId(1)).await.unwrap();
    assert_eq!(
        item.idproduct_stock_history,
        entries[0].idproduct_stock_history
    );
}

#[tokio::test]
async fn an_unknown_change_type_is_kept() {
    let (server, client) = setup().await;
    let mut item = json("stock_history_item");
    item["change_type"] = value!("something_new");
    Mock::given(path("/stockhistory/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item))
        .mount(&server)
        .await;

    let item = client.stock_history().get(StockHistoryId(1)).await.unwrap();
    assert_eq!(
        item.change_type,
        Some(Reported::Unknown(Value::from("something_new")))
    );
}
