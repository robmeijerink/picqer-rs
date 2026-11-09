//! The fixtures of these tests are the examples of the documentation: a developer account has no
//! webshop integration, so there are no real responses.

mod common;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::pagination::Page;
use picqer_rs::types::{CompanyWebshopId, OrderId, WebshopOrderId};
use picqer_rs::webshop_orders::{WebshopOrderQuery, WebshopOrderStatus};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn webshop_orders_are_listed_with_their_import_status() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/webshoporders"))
        .respond_with(respond("webshop_orders"))
        .expect(1)
        .mount(&server)
        .await;

    let orders = client
        .webshop_orders()
        .list(&WebshopOrderQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(orders.len(), 2);
    assert_eq!(
        orders[0].status,
        Reported::Known(WebshopOrderStatus::Imported)
    );
    assert_eq!(orders[0].idorder, Some(OrderId(5001)));
    assert_eq!(orders[0].foreign_id, "WEB12345");
    assert_eq!(orders[0].foreign_number.as_deref(), Some("ORDER-2025-001"));
    assert_eq!(
        orders[1].status,
        Reported::Known(WebshopOrderStatus::Expected)
    );
    assert_eq!(orders[1].idorder, None);
    assert_eq!(orders[1].reason, None);
    assert_eq!(orders[1].ordered.to_string(), "2025-07-12 09:30:00");
}

#[tokio::test]
async fn a_webshop_order_is_fetched_and_a_status_that_is_not_known_is_kept() {
    let (server, client) = setup().await;
    let mut other = common::json("webshop_order");
    other["status"] = serde_json::json!("on_hold");
    other["foreign_id"] = serde_json::json!(12345);
    other["foreign_number"] = serde_json::json!(20250001);
    Mock::given(method("GET"))
        .and(path("/webshoporders/101"))
        .respond_with(respond("webshop_order"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/webshoporders/102"))
        .respond_with(ResponseTemplate::new(200).set_body_json(other))
        .mount(&server)
        .await;

    let one = client
        .webshop_orders()
        .get(WebshopOrderId(101))
        .await
        .unwrap();
    assert_eq!(one.foreign_status.as_deref(), Some("paid"));
    let odd = client
        .webshop_orders()
        .get(WebshopOrderId(102))
        .await
        .unwrap();
    assert!(matches!(odd.status, Reported::Unknown(_)));
    assert_eq!(odd.foreign_id, "12345");
    assert_eq!(odd.foreign_number.as_deref(), Some("20250001"));
}

#[tokio::test]
async fn filters_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/webshoporders"))
        .and(query_param("foreign_id", "ord_1234"))
        .and(query_param("foreign_number", "20250001"))
        .and(query_param("idorder", "362231"))
        .and(query_param("idcompany_webshop", "1"))
        .respond_with(respond("webshop_orders"))
        .expect(1)
        .mount(&server)
        .await;

    let query = WebshopOrderQuery::new()
        .foreign_id("ord_1234")
        .foreign_number("20250001")
        .idorder(OrderId(362231))
        .idcompany_webshop(CompanyWebshopId(1));
    client
        .webshop_orders()
        .list(&query, Page::FIRST)
        .await
        .unwrap();
}
