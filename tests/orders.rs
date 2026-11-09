mod common;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::country::CountryCode;
use picqer_rs::orders::{OrderQuery, OrderStatus};
use picqer_rs::pagination::Page;
use picqer_rs::types::{CustomerId, DateTime, Money, OrderId, OrderNumber, ProductId, WarehouseId};
use serde_json::json;
use wiremock::matchers::{method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn orders_are_listed_with_their_lines_and_tags() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/orders"))
        .and(query_param_is_missing("offset"))
        .respond_with(respond("orders"))
        .expect(1)
        .mount(&server)
        .await;

    let orders = client
        .orders()
        .list(&OrderQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(orders.len(), 3);
    assert_eq!(orders[0].status, Reported::Known(OrderStatus::Concept));
    assert!(orders[0].orderid.starts_with('O'));
    assert_eq!(orders[0].via_portal, Some(false));
    assert!(!orders[0].products.is_empty());
    assert_eq!(orders[2].status, Reported::Known(OrderStatus::Processing));
    assert_eq!(orders[2].picklists.len(), 1);
    assert!(orders[2].picklists[0].products.is_empty());
    assert_eq!(orders[2].picklists[0].idorder, Some(orders[2].idorder));
}

#[tokio::test]
async fn an_order_is_read_completely() {
    let (server, client) = setup().await;
    Mock::given(path("/orders/5"))
        .respond_with(respond("order"))
        .mount(&server)
        .await;

    let order = client.orders().get(OrderId(5)).await.unwrap();
    assert_eq!(
        order.deliverycountry,
        Some(Reported::Known(CountryCode::NL))
    );
    assert_eq!(order.deliveryname.as_deref(), Some("Example Customer"));
    assert_eq!(order.products.len(), 3);
    let line = &order.products[0];
    assert!(line.price.unwrap() > Money::from_cents(0).unwrap());
    assert_eq!(line.amount_cancelled, Some(0));
    assert_eq!(line.partof_idorder_product, None);
    assert_eq!(order.created.unwrap().to_string(), "2026-11-09 22:23:55");
    assert!(order.updated.is_some());
    assert!(order.tags.contains_key("lightspeed"));
    assert_eq!(order.discount.unwrap().get(), 0.0);
    assert!(order.preferred_delivery_date.is_none());
}

#[tokio::test]
async fn an_order_without_tags_has_an_empty_list_instead_of_an_object() {
    let (server, client) = setup().await;
    Mock::given(path("/orders"))
        .respond_with(respond("orders"))
        .mount(&server)
        .await;

    let orders = client
        .orders()
        .list(&OrderQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert!(orders[1].tags.is_empty());
    assert!(!orders[0].tags.is_empty());
}

#[tokio::test]
async fn an_unknown_status_is_kept() {
    let (server, client) = setup().await;
    let mut order = common::json("order");
    order["status"] = json!("on_the_moon");
    Mock::given(path("/orders/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(order))
        .mount(&server)
        .await;

    let order = client.orders().get(OrderId(5)).await.unwrap();
    assert!(matches!(order.status, Reported::Unknown(_)));
}

#[tokio::test]
async fn filters_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(path("/orders"))
        .and(query_param("status", "processing"))
        .and(query_param("idcustomer", "4"))
        .and(query_param("idwarehouse", "9"))
        .and(query_param("sincedate", "2026-11-01 00:00:00"))
        .and(query_param("offset", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let since: DateTime = "2026-11-01 00:00:00".parse().unwrap();
    let query = OrderQuery::new()
        .status(OrderStatus::Processing)
        .idcustomer(CustomerId(4))
        .idwarehouse(WarehouseId(9))
        .since(since);
    let orders = client.orders().list(&query, Page::new(1)).await.unwrap();
    assert!(orders.is_empty());
    let _ = ProductId(1);
}

#[tokio::test]
async fn an_order_is_found_by_its_number() {
    let (server, client) = setup().await;
    Mock::given(path("/orders"))
        .and(query_param("orderid", "O2026-1598"))
        .respond_with(respond("orders"))
        .expect(1)
        .mount(&server)
        .await;

    let number = OrderNumber::new("O2026-1598").unwrap();
    let found = client.orders().find_by_number(&number).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn the_answer_to_creating_an_order_leaves_some_fields_empty() {
    use common::created;
    use picqer_rs::orders::{NewOrder, NewOrderProduct};
    use picqer_rs::types::NonEmpty;

    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/orders"))
        .respond_with(created("order_created"))
        .expect(1)
        .mount(&server)
        .await;

    let line = NewOrderProduct::by_id(ProductId(1), std::num::NonZeroU32::new(1).unwrap());
    let order = NewOrder::for_customer(CustomerId(5), NonEmpty::new(vec![line]).unwrap());
    let created = client.orders().create(&order).await.unwrap();
    assert_eq!(created.status, Reported::Known(OrderStatus::Concept));
    assert_eq!(created.via_portal, None);
    assert_eq!(created.invoiced, None);
    assert!(created.picklists.is_empty());
}
