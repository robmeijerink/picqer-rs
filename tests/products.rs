mod common;

use common::{AUTH, created, error, fixture, json, respond, setup};
use picqer_rs::country::CountryCode;
use picqer_rs::pagination::Page;
use picqer_rs::pricelists::PricelistPrice;
use picqer_rs::products::{
    NewProduct, ProductChanges, ProductFieldValue, ProductQuery, ProductStatus, ProductType,
};
use picqer_rs::types::{
    Grams, Money, NonEmpty, PricelistId, ProductCode, ProductFieldId, ProductId, ProductName,
    TagId, VatGroupId, WarehouseId,
};
use picqer_rs::{Reported, products::AbcClassification};
use serde_json::{Value, json as value};
use wiremock::matchers::{body_json, header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn a_list_is_read_with_credentials_and_without_a_query() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products"))
        .and(header("authorization", AUTH))
        .and(query_param_is_missing("offset"))
        .and(query_param_is_missing("inactive"))
        .respond_with(respond("products"))
        .expect(1)
        .mount(&server)
        .await;

    let products = client
        .products()
        .list(&ProductQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(products.len(), 2);
    assert!(products.iter().all(|product| !product.name.is_empty()));
}

#[tokio::test]
async fn a_product_with_every_field_is_read_completely() {
    let (server, client) = setup().await;
    Mock::given(path("/products/1"))
        .respond_with(respond("product_full"))
        .mount(&server)
        .await;

    let product = client.products().get(ProductId(1)).await.unwrap();
    assert_eq!(product.price, Money::from_cents(1250).unwrap());
    assert_eq!(
        product.fixedstockprice,
        Some(Money::from_cents(325).unwrap())
    );
    assert_eq!(product.weight, Some(Grams::new(250)));
    assert_eq!(
        product.country_of_origin,
        Some(Reported::Known(CountryCode::NL))
    );
    assert_eq!(
        product.product_type,
        Some(Reported::Known(ProductType::Normal))
    );
    assert_eq!(product.created.to_string(), "2026-11-09 13:47:28");
    assert!(!product.tags["lightspeed"].inherit);
    assert_eq!(
        product.pricelists,
        vec![PricelistPrice {
            idpricelist: PricelistId(2003),
            price: Money::from_cents(999).unwrap()
        }]
    );
    assert_eq!(product.suppliers.unwrap()[0].is_default, Some(true));
    assert_eq!(product.stock.len(), 1);
    assert_eq!(product.stock[0].idwarehouse, WarehouseId(14153));
    assert!(product.images.is_empty());
    assert_eq!(product.analysis_abc_classification, None);
}

#[tokio::test]
async fn the_values_of_product_fields_are_read_as_text() {
    let (server, client) = setup().await;
    Mock::given(path("/products/1"))
        .respond_with(respond("product_with_fields"))
        .mount(&server)
        .await;

    let product = client.products().get(ProductId(1)).await.unwrap();
    assert_eq!(product.productfields.len(), 2);
    assert_eq!(
        product.productfields[0].idproductfield,
        ProductFieldId(6242)
    );
    assert_eq!(product.productfields[0].title.as_deref(), Some("Season"));
    assert_eq!(product.productfields[0].value.as_deref(), Some("created"));
    assert_eq!(product.productfields[1].value.as_deref(), Some("many"));
    assert_eq!(product.pricelists.len(), 1);
    assert_eq!(product.pricelists[0].price, Money::from_cents(199).unwrap());
}

#[tokio::test]
async fn the_choices_of_a_multicheckbox_are_read_apart() {
    let (server, client) = setup().await;
    Mock::given(path("/products/1"))
        .respond_with(respond("product_multicheckbox"))
        .mount(&server)
        .await;

    let product = client.products().get(ProductId(1)).await.unwrap();
    let entry = product
        .productfields
        .iter()
        .find(|entry| entry.idproductfield == ProductFieldId(6246))
        .unwrap();
    assert_eq!(
        entry.value.as_deref(),
        Some("netherlands;belgium;luxembourg")
    );
    assert_eq!(entry.choices(), ["netherlands", "belgium", "luxembourg"]);
    let suppliers = product.suppliers.as_ref().unwrap();
    assert_eq!(suppliers.len(), 1);
    assert_eq!(suppliers[0].idsupplier, None);
    assert_eq!(suppliers[0].is_default, Some(true));
    assert!(product.productfields[0].choices().len() == 1);
}

#[tokio::test]
async fn a_product_is_created_with_pricelist_prices_and_field_values() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/products"))
        .and(body_json(value!({
            "idvatgroup": 7,
            "productcode": "A-1",
            "name": "A product",
            "price": 2.5,
            "pricelists": [{"idpricelist": 2004, "price": 1.99}],
            "productfields": [
                {"idproductfield": 6242, "value": "created"},
                {"idproductfield": 6243, "value": null}
            ]
        })))
        .respond_with(created("product_with_fields"))
        .expect(1)
        .mount(&server)
        .await;

    let product = NewProduct::new(
        VatGroupId(7),
        ProductCode::new("A-1").unwrap(),
        ProductName::new("A product").unwrap(),
        Money::from_cents(250).unwrap(),
    )
    .pricelists(vec![PricelistPrice {
        idpricelist: PricelistId(2004),
        price: Money::from_cents(199).unwrap(),
    }])
    .productfields(vec![
        ProductFieldValue::new(ProductFieldId(6242), "created"),
        ProductFieldValue::cleared(ProductFieldId(6243)),
    ]);
    client.products().create(&product).await.unwrap();
}

#[tokio::test]
async fn unknown_enum_values_are_kept_not_refused() {
    let (server, client) = setup().await;
    let mut product = json("product_full");
    product["type"] = value!("something_new");
    product["analysis_abc_classification"] = value!("D");
    product["country_of_origin"] = value!("ZZ");
    Mock::given(path("/products/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(product))
        .mount(&server)
        .await;

    let product = client.products().get(ProductId(1)).await.unwrap();
    assert_eq!(
        product.product_type,
        Some(Reported::Unknown(Value::from("something_new")))
    );
    assert_eq!(
        product.analysis_abc_classification,
        Some(Reported::Unknown(Value::from("D")))
    );
    assert_eq!(
        product.country_of_origin,
        Some(Reported::Unknown(Value::from("ZZ")))
    );
    let _ = AbcClassification::A;
}

#[tokio::test]
async fn the_pick_amount_is_read_as_a_number_or_as_text() {
    let (server, client) = setup().await;
    let mut product = json("product_full");
    product["analysis_pick_amount_per_day"] = value!("0.036");
    Mock::given(path("/products/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(product))
        .mount(&server)
        .await;

    let product = client.products().get(ProductId(1)).await.unwrap();
    assert_eq!(product.analysis_pick_amount_per_day, Some(0.036));
}

#[tokio::test]
async fn filters_and_pages_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(path("/products"))
        .and(query_param("inactive", "true"))
        .and(query_param("type", "normal"))
        .and(query_param("idproducts", "1,22"))
        .and(query_param("offset", "200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(value!([])))
        .expect(1)
        .mount(&server)
        .await;

    let query = ProductQuery::new()
        .status(ProductStatus::Inactive)
        .product_type(ProductType::Normal)
        .idproducts(NonEmpty::new(vec![ProductId(1), ProductId(22)]).unwrap());
    let products = client.products().list(&query, Page::new(2)).await.unwrap();
    assert!(products.is_empty());
}

#[tokio::test]
async fn all_pages_are_followed_until_one_is_short() {
    let (server, client) = setup().await;
    let item = json("products")[0].clone();
    let full: Vec<Value> = (0..100).map(|_| item.clone()).collect();
    let last: Vec<Value> = (0..7).map(|_| item.clone()).collect();
    Mock::given(path("/products"))
        .and(query_param_is_missing("offset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&full))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/products"))
        .and(query_param("offset", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&full))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/products"))
        .and(query_param("offset", "200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&last))
        .expect(1)
        .mount(&server)
        .await;

    let products = client
        .products()
        .list_all(&ProductQuery::new())
        .await
        .unwrap();
    assert_eq!(products.len(), 207);
}

#[tokio::test]
async fn a_product_is_created_with_only_what_was_set() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/products"))
        .and(header("content-type", "application/json"))
        .and(body_json(value!({
            "idvatgroup": 7,
            "productcode": "A-1",
            "name": "A product",
            "price": 12.5,
            "type": "normal",
            "weight": 250,
            "country_of_origin": "NL"
        })))
        .respond_with(created("product_full"))
        .expect(1)
        .mount(&server)
        .await;

    let product = NewProduct::new(
        VatGroupId(7),
        ProductCode::new("A-1").unwrap(),
        ProductName::new("A product").unwrap(),
        Money::from_cents(1250).unwrap(),
    )
    .product_type(ProductType::Normal)
    .weight(Grams::new(250))
    .country_of_origin(CountryCode::NL);
    client.products().create(&product).await.unwrap();
}

#[tokio::test]
async fn a_product_is_changed_with_only_what_was_set() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/products/5"))
        .and(body_json(value!({
            "name": "Renamed",
            "pricelists": [{"idpricelist": 3, "price": 9.99}]
        })))
        .respond_with(respond("product_full"))
        .expect(1)
        .mount(&server)
        .await;

    let changes = ProductChanges::new()
        .name(ProductName::new("Renamed").unwrap())
        .pricelists(vec![PricelistPrice {
            idpricelist: PricelistId(3),
            price: Money::from_cents(999).unwrap(),
        }]);
    client
        .products()
        .update(ProductId(5), &changes)
        .await
        .unwrap();
}

#[tokio::test]
async fn products_are_activated_and_inactivated_with_a_post() {
    let (server, client) = setup().await;
    for action in ["activate", "inactivate"] {
        Mock::given(method("POST"))
            .and(path(format!("/products/5/{action}")))
            .respond_with(respond("product_full"))
            .expect(1)
            .mount(&server)
            .await;
    }

    client.products().activate(ProductId(5)).await.unwrap();
    client.products().inactivate(ProductId(5)).await.unwrap();
}

#[tokio::test]
async fn a_product_is_found_by_code_among_the_inactive_ones_too() {
    let (server, client) = setup().await;
    let code = ProductCode::new("A-1").unwrap();
    Mock::given(path("/products"))
        .and(query_param("productcode", "A-1"))
        .and(query_param_is_missing("inactive"))
        .respond_with(ResponseTemplate::new(200).set_body_json(value!([])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/products"))
        .and(query_param("productcode", "A-1"))
        .and(query_param("inactive", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(value!([json("product_full")])))
        .expect(1)
        .mount(&server)
        .await;

    let found = client.products().find_by_productcode(&code).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn tags_and_pricelist_prices_of_a_product() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products/5/tags"))
        .respond_with(respond("product_tags"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/products/5/tags"))
        .and(body_json(value!({"idtag": 9})))
        .respond_with(created("tag"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/products/5/tags/9"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/products/5/pricelists/3"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let products = client.products();
    assert_eq!(products.tags(ProductId(5)).await.unwrap().len(), 1);
    products.add_tag(ProductId(5), TagId(9)).await.unwrap();
    products.remove_tag(ProductId(5), TagId(9)).await.unwrap();
    products
        .remove_pricelist_price(ProductId(5), PricelistId(3))
        .await
        .unwrap();
}

#[tokio::test]
async fn a_missing_product_is_a_not_found_error() {
    let (server, client) = setup().await;
    Mock::given(path("/products/99999999"))
        .respond_with(error(404, "error_not_found"))
        .mount(&server)
        .await;

    let error = client
        .products()
        .get(ProductId(99_999_999))
        .await
        .unwrap_err();
    assert!(error.is_not_found());
    assert_eq!(
        error.to_string(),
        "picqer api error (status 404, code 20): Item not found"
    );
}

#[tokio::test]
async fn missing_fields_come_back_as_an_api_error_with_the_code() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .respond_with(error(400, "error_required"))
        .mount(&server)
        .await;

    let product = NewProduct::new(
        VatGroupId(7),
        ProductCode::new("A-1").unwrap(),
        ProductName::new("A product").unwrap(),
        Money::from_euros(1).unwrap(),
    );
    let error = client.products().create(&product).await.unwrap_err();
    let api = error.as_api().unwrap();
    assert_eq!(api.status, 400);
    assert_eq!(
        api.code,
        Some(Reported::Known(picqer_rs::ErrorCode::MissingParameters))
    );
    let _ = fixture("error_required");
}
