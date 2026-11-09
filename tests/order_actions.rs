mod common;

use std::num::NonZeroU32;

use common::{created, respond, setup};
use picqer_rs::country::CountryCode;
use picqer_rs::customers::{AddressName, DiscountPercentage, Language};
use picqer_rs::orders::{
    NewOrder, NewOrderLine, NewOrderProduct, OrderAddress, OrderAddressChanges, OrderChanges,
    OrderLineChanges,
};
use picqer_rs::pagination::Page;
use picqer_rs::types::{
    CustomerId, FulfilmentCustomerId, Money, NonEmpty, OrderId, OrderProductId, PauseReason,
    ProductCode, ProductId, TagId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

fn amount(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

fn lines() -> NonEmpty<NewOrderProduct> {
    NonEmpty::new(vec![
        NewOrderProduct::by_id(ProductId(633), amount(1)),
        NewOrderProduct::by_code(ProductCode::new("A-1").unwrap(), amount(2))
            .price(Money::from_cents(999).unwrap()),
    ])
    .unwrap()
}

#[tokio::test]
async fn an_order_for_a_customer_is_created_with_its_lines() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/orders"))
        .and(body_json(json!({
            "idcustomer": 5633,
            "reference": "R1",
            "language": "en",
            "discount": 10.0,
            "products": [
                {"idproduct": 633, "amount": 1},
                {"productcode": "A-1", "amount": 2, "price": 9.99}
            ]
        })))
        .respond_with(created("order"))
        .expect(1)
        .mount(&server)
        .await;

    let order = NewOrder::for_customer(CustomerId(5633), lines())
        .reference("R1")
        .language(Language::En)
        .discount(DiscountPercentage::new(10.0).unwrap());
    client.orders().create(&order).await.unwrap();
}

#[tokio::test]
async fn a_guest_order_carries_both_addresses() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/orders"))
        .and(body_json(json!({
            "idcustomer": null,
            "idfulfilment_customer": 7,
            "deliveryname": "John",
            "deliverycountry": "NL",
            "deliveryzipcode": "3928 DM",
            "invoicename": "John B.V.",
            "invoicecountry": "BE",
            "products": [
                {"idproduct": 633, "amount": 1},
                {"productcode": "A-1", "amount": 2, "price": 9.99}
            ]
        })))
        .respond_with(created("order"))
        .expect(1)
        .mount(&server)
        .await;

    let name = |text| AddressName::new(text).unwrap();
    let order = NewOrder::guest(
        OrderAddress::new(name("John"), CountryCode::NL).zipcode("3928 DM"),
        OrderAddress::new(name("John B.V."), CountryCode::BE),
        lines(),
    )
    .idfulfilment_customer(FulfilmentCustomerId(7));
    client.orders().create(&order).await.unwrap();
}

#[tokio::test]
async fn an_order_is_changed_with_only_what_was_set() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/orders/5"))
        .and(body_json(
            json!({"idcustomer": null, "deliverycity": "Utrecht", "reference": "R2"}),
        ))
        .respond_with(respond("order"))
        .expect(1)
        .mount(&server)
        .await;

    let changes = OrderChanges::new()
        .without_customer()
        .delivery(OrderAddressChanges::new().city("Utrecht"))
        .reference("R2");
    client.orders().update(OrderId(5), &changes).await.unwrap();
}

#[tokio::test]
async fn an_order_is_cancelled_normally_or_by_force() {
    let (server, client) = setup().await;
    Mock::given(method("DELETE"))
        .and(path("/orders/5"))
        .and(query_param_is_missing("force"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/orders/6"))
        .and(query_param("force", "true"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    client.orders().cancel(OrderId(5)).await.unwrap();
    client.orders().force_cancel(OrderId(6)).await.unwrap();
}

#[tokio::test]
async fn every_status_change_is_a_post_to_its_own_path() {
    let (server, client) = setup().await;
    for action in [
        "process",
        "resume",
        "change-to-concept",
        "reopen",
        "undo-cancellation",
        "allocate",
        "deallocate",
        "prioritise",
        "anonymize",
    ] {
        Mock::given(method("POST"))
            .and(path(format!("/orders/5/{action}")))
            .respond_with(respond("order"))
            .expect(1)
            .mount(&server)
            .await;
    }

    let orders = client.orders();
    let id = OrderId(5);
    orders.process(id).await.unwrap();
    orders.resume(id).await.unwrap();
    orders.change_to_concept(id).await.unwrap();
    orders.reopen(id).await.unwrap();
    orders.undo_cancellation(id).await.unwrap();
    orders.allocate(id).await.unwrap();
    orders.deallocate(id).await.unwrap();
    orders.prioritise(id).await.unwrap();
    orders.anonymize(id).await.unwrap();
}

#[tokio::test]
async fn pausing_needs_a_reason_and_backorders_can_prefer_bulk_locations() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/orders/5/pause"))
        .and(body_json(json!({"reason": "Wrong address"})))
        .respond_with(respond("order"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/orders/5/process-backorders"))
        .and(query_param("prefer_bulk_locations", "true"))
        .respond_with(respond("order"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/orders/6/process-backorders"))
        .and(query_param_is_missing("prefer_bulk_locations"))
        .respond_with(respond("order"))
        .expect(1)
        .mount(&server)
        .await;

    assert!(PauseReason::new("  ").is_err());
    let reason = PauseReason::new("Wrong address").unwrap();
    client.orders().pause(OrderId(5), &reason).await.unwrap();
    client
        .orders()
        .process_backorders(OrderId(5), true)
        .await
        .unwrap();
    client
        .orders()
        .process_backorders(OrderId(6), false)
        .await
        .unwrap();
}

#[tokio::test]
async fn the_status_of_each_line_is_listed() {
    let (server, client) = setup().await;
    Mock::given(path("/orders/5/productstatus"))
        .respond_with(respond("order_product_status"))
        .mount(&server)
        .await;

    let status = client.orders().product_status(OrderId(5)).await.unwrap();
    assert_eq!(status.len(), 2);
    assert_eq!(status[0].delivered, Some(0));
    assert_eq!(status[0].picklist, Some(0));
    assert!(status[0].productcode.is_some());
}

#[tokio::test]
async fn lines_are_listed_added_changed_and_removed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/orders/5/products"))
        .respond_with(respond("order_lines"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/orders/5/products"))
        .and(body_json(
            json!({"idproduct": 9, "amount": 2, "remarks": "handle with care"}),
        ))
        .respond_with(created("order_line"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/orders/5/products/7"))
        .and(body_json(json!({"amount": 4, "price": 3.25})))
        .respond_with(respond("order_line"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/orders/5/products/7"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let orders = client.orders();
    assert_eq!(orders.lines(OrderId(5)).await.unwrap().len(), 2);
    let new = NewOrderLine::new(ProductId(9), amount(2)).remarks("handle with care");
    let line = orders.add_line(OrderId(5), &new).await.unwrap();
    assert_eq!(line.remarks.as_deref(), Some("handle with care"));
    let changes = OrderLineChanges::new()
        .amount(amount(4))
        .price(Money::from_cents(325).unwrap());
    orders
        .update_line(OrderId(5), OrderProductId(7), &changes)
        .await
        .unwrap();
    orders
        .remove_line(OrderId(5), OrderProductId(7))
        .await
        .unwrap();
}

#[tokio::test]
async fn tags_of_an_order() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/orders/5/tags"))
        .respond_with(respond("customer_tags"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/orders/5/tags"))
        .and(body_json(json!({"idtag": 9})))
        .respond_with(created("tag"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/orders/5/tags/9"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let orders = client.orders();
    assert_eq!(orders.tags(OrderId(5), Page::FIRST).await.unwrap().len(), 1);
    orders.add_tag(OrderId(5), TagId(9)).await.unwrap();
    orders.remove_tag(OrderId(5), TagId(9)).await.unwrap();
}
