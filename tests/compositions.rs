mod common;

use std::num::NonZeroU32;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::pagination::Page;
use picqer_rs::products::{NewProductPart, ProductType, Production};
use picqer_rs::stock::StockReason;
use picqer_rs::types::{ProductId, ProductPartId, WarehouseId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

fn amount(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

#[tokio::test]
async fn the_parts_of_a_composition_are_listed() {
    let (server, client) = setup().await;
    Mock::given(path("/products/5/parts"))
        .and(query_param_is_missing("nested"))
        .respond_with(respond("product_parts"))
        .expect(1)
        .mount(&server)
        .await;

    let parts = client.products().parts(ProductId(5)).await.unwrap();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].amount, 2);
    assert_eq!(parts[0].product_type, Reported::Known(ProductType::Normal));
    assert_eq!(
        parts[1].product_type,
        Reported::Known(ProductType::VirtualComposition)
    );
}

#[tokio::test]
async fn the_tree_of_parts_follows_the_virtual_compositions() {
    let (server, client) = setup().await;
    Mock::given(path("/products/5/parts"))
        .and(query_param("nested", "true"))
        .respond_with(respond("product_parts_nested"))
        .expect(1)
        .mount(&server)
        .await;

    let tree = client.products().parts_tree(ProductId(5)).await.unwrap();
    assert_eq!(tree.len(), 2);
    assert!(tree[0].parts.is_empty());
    assert_eq!(tree[1].parts.len(), 1);
    assert_eq!(tree[1].parts[0].part.amount, 3);
    assert_eq!(tree[1].part.amount, 1);
}

#[tokio::test]
async fn the_compositions_of_a_product_are_the_reverse() {
    let (server, client) = setup().await;
    Mock::given(path("/products/5/compositions"))
        .and(query_param_is_missing("nested"))
        .respond_with(respond("product_compositions"))
        .mount(&server)
        .await;
    Mock::given(path("/products/5/compositions"))
        .and(query_param("nested", "true"))
        .respond_with(respond("product_compositions_nested"))
        .mount(&server)
        .await;

    let flat = client
        .products()
        .compositions_all(ProductId(5))
        .await
        .unwrap();
    assert_eq!(flat.len(), 1);
    assert_eq!(flat[0].amount, 3);
    let tree = client
        .products()
        .compositions_tree(ProductId(5), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(tree[0].compositions.len(), 1);
    assert!(tree[0].compositions[0].compositions.is_empty());
}

#[tokio::test]
async fn one_part_is_fetched_added_changed_and_removed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products/5/parts/7"))
        .respond_with(respond("product_part"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/products/5/parts"))
        .and(body_json(json!({"idproduct": 9, "amount": 2})))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_raw(common::fixture("product_part"), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/products/5/parts/7"))
        .and(body_json(json!({"amount": 3})))
        .respond_with(respond("product_part"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/products/5/parts/7"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let products = client.products();
    let part = products.part(ProductId(5), ProductPartId(7)).await.unwrap();
    assert_eq!(part.productcode, "AI229209");
    products
        .add_part(ProductId(5), &NewProductPart::new(ProductId(9), amount(2)))
        .await
        .unwrap();
    products
        .update_part(ProductId(5), ProductPartId(7), amount(3))
        .await
        .unwrap();
    products
        .remove_part(ProductId(5), ProductPartId(7))
        .await
        .unwrap();
}

#[tokio::test]
async fn production_is_registered_without_an_answer_body() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/products/5/produce"))
        .and(body_json(json!({
            "idwarehouse": 67, "amount": 3, "reason": "New production at table 2"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let production = Production::new(
        WarehouseId(67),
        amount(3),
        StockReason::new("New production at table 2").unwrap(),
    );
    client
        .products()
        .produce(ProductId(5), &production)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_product_that_is_not_a_composition_has_no_parts() {
    let (server, client) = setup().await;
    Mock::given(path("/products/5/parts"))
        .respond_with(ResponseTemplate::new(404).set_body_json(
            json!({"error": true, "error_code": 20, "error_message": "Product not applicable"}),
        ))
        .mount(&server)
        .await;

    let error = client.products().parts(ProductId(5)).await.unwrap_err();
    assert!(error.is_not_found());
}
