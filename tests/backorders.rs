mod common;

use common::{respond, setup};
use picqer_rs::backorders::BackorderQuery;
use picqer_rs::pagination::Page;
use picqer_rs::types::{BackorderId, CustomerId, FulfilmentCustomerId, OrderId, ProductId};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn backorders_are_listed_without_what_only_a_single_one_has() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/backorders"))
        .respond_with(respond("backorders"))
        .expect(1)
        .mount(&server)
        .await;

    let backorders = client
        .backorders()
        .list(&BackorderQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(backorders.len(), 3);
    let first = &backorders[0];
    assert!(first.amount > 0);
    assert_eq!(first.amount_available, Some(0));
    assert_eq!(first.has_parts, Some(false));
    assert_eq!(first.part_of_idbackorder, None);
    assert_eq!(first.idreturn, None);
    assert!(first.created_at.is_some());
    assert_eq!(first.is_purchased, None);
    assert_eq!(first.date_available, None);
}

#[tokio::test]
async fn a_single_backorder_and_those_of_an_order_say_whether_they_are_purchased() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/backorders/502452454"))
        .respond_with(respond("backorder"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/orders/282382018/backorders"))
        .respond_with(respond("order_backorders"))
        .mount(&server)
        .await;

    let one = client
        .backorders()
        .get(BackorderId(502452454))
        .await
        .unwrap();
    assert_eq!(one.is_purchased, Some(false));
    assert_eq!(one.date_available, None);
    assert_eq!(one.idorder, OrderId(282382018));
    let of_order = client
        .backorders()
        .for_order(OrderId(282382018))
        .await
        .unwrap();
    assert_eq!(of_order, vec![one]);
}

#[tokio::test]
async fn the_date_of_a_backorder_is_kept_as_picqer_sends_it() {
    let (server, client) = setup().await;
    let mut body = common::json("backorder");
    body["is_purchased"] = serde_json::json!(true);
    body["date_available"] = serde_json::json!("2014-09-02");
    Mock::given(method("GET"))
        .and(path("/backorders/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let one = client.backorders().get(BackorderId(1)).await.unwrap();
    assert_eq!(one.is_purchased, Some(true));
    assert_eq!(one.date_available.as_deref(), Some("2014-09-02"));
}

#[tokio::test]
async fn filters_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/backorders"))
        .and(query_param("idorder", "2"))
        .and(query_param("idproduct", "13"))
        .and(query_param("idcustomer", "5"))
        .and(query_param("sinceid", "201992"))
        .and(query_param("sincedate", "2026-11-09 16:00:00"))
        .and(query_param("idfulfilment_customer", "623"))
        .respond_with(respond("backorders"))
        .expect(1)
        .mount(&server)
        .await;

    let query = BackorderQuery::new()
        .idorder(OrderId(2))
        .idproduct(ProductId(13))
        .idcustomer(CustomerId(5))
        .since_id(BackorderId(201992))
        .since("2026-11-09 16:00:00".parse().unwrap())
        .idfulfilment_customer(FulfilmentCustomerId(623));
    client.backorders().list(&query, Page::FIRST).await.unwrap();
}

#[tokio::test]
async fn a_backorder_is_deleted_and_the_backorders_are_processed() {
    let (server, client) = setup().await;
    Mock::given(method("DELETE"))
        .and(path("/backorders/5"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/backorders/process"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .expect(1)
        .mount(&server)
        .await;

    client.backorders().delete(BackorderId(5)).await.unwrap();
    client.backorders().process().await.unwrap();
}
