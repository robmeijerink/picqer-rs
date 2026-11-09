mod common;

use std::num::NonZeroU32;

use common::{created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::locations::LocationKind;
use picqer_rs::products::{LotNumber, NewProductLocation, ProductLocationChanges};
use picqer_rs::stock::{StockMove, StockPlace, StockReason};
use picqer_rs::types::{LocationId, PicklistId, ProductId, WarehouseId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn the_locations_of_a_product_come_with_its_stock_there() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products/52947932/locations"))
        .respond_with(respond("product_locations"))
        .mount(&server)
        .await;

    let locations = client
        .products()
        .locations(ProductId(52947932))
        .await
        .unwrap();
    assert_eq!(locations.len(), 1);
    let linked = &locations[0];
    assert_eq!(
        linked.location.kind,
        Some(Reported::Known(LocationKind::Location))
    );
    assert_eq!(linked.location.idlocation, LocationId(12817277));
    assert!(!linked.location.name.is_empty());
    let stock = linked.stock_for_product.as_ref().unwrap();
    assert_eq!(stock.stock, Some(22));
    assert_eq!(stock.stock_reserved_picklists, Some(17));
    assert_eq!(linked.note, None);
    assert_eq!(linked.is_preferred, Some(false));
    assert_eq!(linked.lot_number, None);
    assert_eq!(linked.expiration_date, None);
}

#[tokio::test]
async fn a_location_is_linked_changed_and_unlinked() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/products/9/locations"))
        .and(body_json(
            json!({"idlocation": 5, "note": "top shelf", "is_preferred": true}),
        ))
        .respond_with(created("product_location_linked"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/products/9/locations/5"))
        .and(body_json(json!({
            "note": null, "is_preferred": false,
            "lot_number": "LOT-2026-001", "expiration_date": "2027-01-15"
        })))
        .respond_with(respond("product_location_linked"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/products/9/locations/5"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let products = client.products();
    let linked = products
        .link_location(
            ProductId(9),
            &NewProductLocation::new(LocationId(5))
                .note("top shelf")
                .preferred(),
        )
        .await
        .unwrap();
    assert_eq!(linked.note.as_deref(), Some("top shelf"));
    assert_eq!(linked.is_preferred, Some(true));
    assert_eq!(linked.stock_for_product.unwrap().stock, None);
    products
        .update_location(
            ProductId(9),
            LocationId(5),
            &ProductLocationChanges::new()
                .without_note()
                .not_preferred()
                .lot_number(LotNumber::new("LOT-2026-001").unwrap())
                .expiration_date("2027-01-15".parse().unwrap()),
        )
        .await
        .unwrap();
    products
        .unlink_location(ProductId(9), LocationId(5))
        .await
        .unwrap();
}

#[tokio::test]
async fn stock_is_moved_and_the_answer_has_both_ends() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/products/9/move-stock"))
        .and(body_json(json!({
            "amount": 1,
            "from": {"idlocation": 12818304},
            "to": {"idlocation": 12818305},
            "move_claims_of_idpicklist": 197985866,
            "reason": "tidy up"
        })))
        .respond_with(respond("stock_moved"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/products/9/move-stock"))
        .and(body_json(json!({
            "amount": 2,
            "from": {"idlocation": null, "idwarehouse": 14153},
            "to": {"idlocation": 12818305}
        })))
        .respond_with(respond("stock_moved"))
        .expect(1)
        .mount(&server)
        .await;

    let one = NonZeroU32::new(1).unwrap();
    let moved = client
        .products()
        .move_stock(
            ProductId(9),
            &StockMove::new(
                one,
                StockPlace::Location(LocationId(12818304)),
                StockPlace::Location(LocationId(12818305)),
            )
            .move_claims_of(PicklistId(197985866))
            .reason(StockReason::new("tidy up").unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(moved.amount, 1);
    let from = moved.from.location.as_ref().unwrap();
    assert_eq!(from.idlocation, LocationId(12818304));
    assert_eq!(moved.from.warehouse.level.idwarehouse, WarehouseId(14153));
    assert_eq!(moved.from.warehouse, moved.to.warehouse);
    assert_eq!(
        moved.to.location.as_ref().unwrap().idlocation,
        LocationId(12818305)
    );
    assert_eq!(moved.from.warehouse.locations.len(), 2);

    client
        .products()
        .move_stock(
            ProductId(9),
            &StockMove::new(
                NonZeroU32::new(2).unwrap(),
                StockPlace::NoSpecificLocation(WarehouseId(14153)),
                StockPlace::Location(LocationId(12818305)),
            ),
        )
        .await
        .unwrap();
}
