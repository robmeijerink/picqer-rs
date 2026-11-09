mod common;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::pagination::Page;
use picqer_rs::products::{ImageType, NewProductImage, WarehouseSettingsChanges};
use picqer_rs::types::{Money, ProductId, ProductImageId, WarehouseId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0];

#[tokio::test]
async fn images_are_listed_added_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products/5/images"))
        .respond_with(respond("product_images"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/products/5/images"))
        .and(body_json(json!({"image": "iVBORw0KGgoAAA=="})))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_raw(common::fixture("product_image"), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/products/5/images/133288196"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let products = client.products();
    let images = products.images_all(ProductId(5)).await.unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].contenttype, Reported::Known(ImageType::Png));
    assert_eq!(images[0].size, 70);
    assert!(images[0].url.as_str().starts_with("https://"));

    let added = products
        .add_image(ProductId(5), &NewProductImage::new(PNG).unwrap())
        .await
        .unwrap();
    assert_eq!(added.idproduct_image, ProductImageId(133_288_196));
    products
        .remove_image(ProductId(5), added.idproduct_image)
        .await
        .unwrap();
    let _ = Page::FIRST;
}

#[tokio::test]
async fn warehouse_settings_are_read_and_changed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products/5/warehouses"))
        .respond_with(respond("product_warehouse_settings"))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/products/5/warehouses/14153"))
        .and(body_json(json!({
            "stock_level_order": 5,
            "picking_stock_replenish_trigger": null
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(common::json("product_warehouse_settings")[0].clone()),
        )
        .expect(1)
        .mount(&server)
        .await;

    let settings = client
        .products()
        .warehouse_settings(ProductId(5))
        .await
        .unwrap();
    assert_eq!(settings.len(), 1);
    assert_eq!(settings[0].idwarehouse, WarehouseId(14153));
    assert_eq!(settings[0].picking_stock_replenish_trigger, None);
    assert_eq!(settings[0].stock_location, None);

    let changes = WarehouseSettingsChanges::new()
        .stock_level_order(5)
        .picking_stock_replenish_trigger(None);
    client
        .products()
        .update_warehouse_settings(ProductId(5), WarehouseId(14153), &changes)
        .await
        .unwrap();
}

#[tokio::test]
async fn the_price_histories_are_read_as_amounts() {
    let (server, client) = setup().await;
    Mock::given(path("/products/5/pricehistory"))
        .respond_with(respond("product_price_history"))
        .mount(&server)
        .await;
    Mock::given(path("/products/5/fixedstockpricehistory"))
        .respond_with(respond("product_fixedstockprice_history"))
        .mount(&server)
        .await;

    let prices = client.products().price_history(ProductId(5)).await.unwrap();
    assert_eq!(prices[0].oldprice, Money::from_cents(100).unwrap());
    assert_eq!(prices[0].pricechange, Money::from_cents(23).unwrap());
    assert_eq!(prices[1].pricechange, Money::from_cents(-623).unwrap());
    assert_eq!(prices[0].date.unwrap().to_string(), "2026-11-09 13:47:55");
    assert!(prices.iter().all(|change| change.iduser.is_some()));

    let fixed = client
        .products()
        .fixed_stock_price_history(ProductId(5))
        .await
        .unwrap();
    assert_eq!(
        fixed[0].old_fixedstockprice,
        Money::from_cents(325).unwrap()
    );
    assert_eq!(
        fixed[0].fixedstockprice_change,
        Money::from_cents(225).unwrap()
    );
    assert_eq!(
        fixed[0].new_fixedstockprice,
        Money::from_cents(550).unwrap()
    );
}

#[tokio::test]
async fn expected_deliveries_have_a_date_or_none() {
    let (server, client) = setup().await;
    let mut lines = common::json("product_expected");
    lines[0]["delivery_date"] = serde_json::Value::Null;
    Mock::given(path("/products/5/expected"))
        .respond_with(ResponseTemplate::new(200).set_body_json(lines))
        .mount(&server)
        .await;

    let expected = client.products().expected(ProductId(5)).await.unwrap();
    assert_eq!(expected.len(), 3);
    assert_eq!(expected[0].delivery_date, None);
    assert_eq!(expected[1].delivery_date.unwrap().to_string(), "2026-11-17");
    assert!(expected.iter().all(|line| line.amount_to_receive > 0));
}
