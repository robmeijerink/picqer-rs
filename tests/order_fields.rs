mod common;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::fields::FieldType;
use picqer_rs::order_fields::OrderFieldValue;
use picqer_rs::orders::{NewOrder, NewOrderProduct};
use picqer_rs::types::{CustomerId, NonEmpty, OrderFieldId, OrderId, ProductId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn order_fields_are_listed_and_fetched() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/orderfields"))
        .respond_with(respond("order_fields"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/orderfields/4741"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            common::json("order_fields")[0].to_string(),
            "application/json",
        ))
        .mount(&server)
        .await;

    let all = client.order_fields().list_all().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].field_type, Some(Reported::Known(FieldType::Text)));
    assert_eq!(all[0].only_accessible_via_api, Some(true));
    assert_eq!(all[0].required, Some(false));
    let one = client.order_fields().get(OrderFieldId(4741)).await.unwrap();
    assert_eq!(one, all[0]);
}

#[tokio::test]
async fn the_values_on_an_order_are_read_set_and_removed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/orders/5/orderfields"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!([{"idorderfield": 4741, "title": "Reference", "value": "12"}]),
            ),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/orders/5/orderfields/4741"))
        .and(body_json(json!({"value": "12"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"idorderfield": 4741, "title": "Reference", "value": 12})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/orders/5/orderfields/4741"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let orders = client.orders();
    let entries = orders.fields(OrderId(5)).await.unwrap();
    assert_eq!(entries[0].value.as_deref(), Some("12"));
    let set = orders
        .set_field(OrderId(5), &OrderFieldValue::new(OrderFieldId(4741), "12"))
        .await
        .unwrap();
    assert_eq!(set.value.as_deref(), Some("12"));
    orders
        .remove_field(OrderId(5), OrderFieldId(4741))
        .await
        .unwrap();
}

#[tokio::test]
async fn an_order_is_created_with_field_values() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/orders"))
        .and(body_json(json!({
            "idcustomer": 7,
            "products": [{"idproduct": 3, "amount": 1}],
            "orderfields": [{"idorderfield": 4741, "value": "abc"}]
        })))
        .respond_with(common::created("order_created"))
        .expect(1)
        .mount(&server)
        .await;

    let order = NewOrder::for_customer(
        CustomerId(7),
        NonEmpty::new(vec![NewOrderProduct::by_id(
            ProductId(3),
            std::num::NonZeroU32::new(1).unwrap(),
        )])
        .unwrap(),
    )
    .orderfields(vec![OrderFieldValue::new(OrderFieldId(4741), "abc")]);
    client.orders().create(&order).await.unwrap();
}
