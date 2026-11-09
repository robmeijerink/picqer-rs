mod common;

use common::{AUTH, created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::fields::FieldType;
use picqer_rs::pagination::Page;
use picqer_rs::tags::{HexColor, NewTag, TagChanges, TagQuery, TagTitle};
use picqer_rs::types::{PricelistId, ProductFieldId, TagId, VatGroupId, WarehouseId};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn warehouses_are_read() {
    let (server, client) = setup().await;
    Mock::given(path("/warehouses"))
        .and(header("authorization", AUTH))
        .respond_with(respond("warehouses"))
        .mount(&server)
        .await;
    Mock::given(path("/warehouses/14153"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(common::json("warehouses")[0].clone()),
        )
        .mount(&server)
        .await;

    let all = client.warehouses().list_all().await.unwrap();
    assert_eq!(all.len(), 1);
    assert!(all[0].active);
    let one = client.warehouses().get(WarehouseId(14153)).await.unwrap();
    assert_eq!(one, all[0]);
}

#[tokio::test]
async fn the_stock_of_a_warehouse_lists_products() {
    let (server, client) = setup().await;
    Mock::given(path("/warehouses/14153/stock"))
        .respond_with(respond("warehouse_stock"))
        .mount(&server)
        .await;

    let stock = client
        .warehouses()
        .stock(WarehouseId(14153), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(stock.len(), 2);
    assert!(!stock[0].productcode.is_empty());
    assert_eq!(stock[0].stock.idwarehouse, WarehouseId(14153));
}

#[tokio::test]
async fn vat_groups_are_read_with_whole_and_fractional_rates() {
    let (server, client) = setup().await;
    Mock::given(path("/vatgroups"))
        .respond_with(respond("vat_groups"))
        .mount(&server)
        .await;
    Mock::given(path("/vatgroups/5"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"idvatgroup": 5, "name": "Half", "percentage": 8.5})),
        )
        .mount(&server)
        .await;

    let all = client.vat_groups().list_all().await.unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].percentage.get(), 21.0);
    let half = client.vat_groups().get(VatGroupId(5)).await.unwrap();
    assert_eq!(half.percentage.get(), 8.5);
    assert_eq!(half.standard, None);
}

#[tokio::test]
async fn tags_can_be_listed_searched_created_changed_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .and(query_param("search", "magento"))
        .respond_with(respond("tags"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/tags"))
        .and(body_json(
            json!({"title": "Shopify", "color": "#823882", "inherit": false}),
        ))
        .respond_with(created("tag"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/tags/4"))
        .and(body_json(json!({"title": "Mosley"})))
        .respond_with(respond("tag"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/tags/4"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let tags = client.tags();
    let found = tags
        .list_all(&TagQuery::new().search("magento"))
        .await
        .unwrap();
    assert_eq!(found.len(), 4);
    let new = NewTag::new(
        TagTitle::new("Shopify").unwrap(),
        HexColor::new("#823882").unwrap(),
        false,
    );
    tags.create(&new).await.unwrap();
    let changes = TagChanges::new().title(TagTitle::new("Mosley").unwrap());
    tags.update(TagId(4), &changes).await.unwrap();
    tags.delete(TagId(4)).await.unwrap();
}

#[tokio::test]
async fn pricelists_are_read() {
    let (server, client) = setup().await;
    Mock::given(path("/pricelists"))
        .respond_with(respond("pricelists"))
        .mount(&server)
        .await;
    Mock::given(path("/pricelists/2004"))
        .respond_with(respond("pricelist"))
        .mount(&server)
        .await;

    let all = client.pricelists().list_all().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "Export");
    let one = client.pricelists().get(PricelistId(2004)).await.unwrap();
    assert_eq!(one, all[0]);
}

#[tokio::test]
async fn product_fields_of_each_type_are_read() {
    let (server, client) = setup().await;
    Mock::given(path("/productfields"))
        .respond_with(respond("product_fields"))
        .mount(&server)
        .await;
    Mock::given(path("/productfields/6247"))
        .respond_with(respond("product_field"))
        .mount(&server)
        .await;

    let all = client.product_fields().list_all().await.unwrap();
    let types: Vec<_> = all.iter().map(|field| field.field_type.clone()).collect();
    assert_eq!(
        types,
        [
            FieldType::Text,
            FieldType::Radio,
            FieldType::Textarea,
            FieldType::Checkbox,
            FieldType::Multicheckbox,
            FieldType::Select,
        ]
        .map(|kind| Some(Reported::Known(kind)))
    );
    assert_eq!(all[0].required, Some(true));
    assert_eq!(all[0].values, Some(vec![]));
    assert_eq!(
        all[4].values.as_deref(),
        Some(
            &[
                "netherlands".to_owned(),
                "belgium".to_owned(),
                "luxembourg".to_owned()
            ][..]
        )
    );
    let one = client
        .product_fields()
        .get(ProductFieldId(6247))
        .await
        .unwrap();
    assert_eq!(one, all[5]);
}

#[tokio::test]
async fn an_unknown_field_type_is_kept() {
    let (server, client) = setup().await;
    let mut field = common::json("product_field");
    field["type"] = json!("barcode");
    Mock::given(path("/productfields/6243"))
        .respond_with(ResponseTemplate::new(200).set_body_json(field))
        .mount(&server)
        .await;

    let field = client
        .product_fields()
        .get(ProductFieldId(6243))
        .await
        .unwrap();
    assert!(matches!(field.field_type, Some(Reported::Unknown(_))));
    assert!(field.value("anything").is_ok());
}

#[tokio::test]
async fn a_value_is_checked_against_the_values_of_the_field() {
    let (server, client) = setup().await;
    Mock::given(path("/productfields/6247"))
        .respond_with(respond("product_field"))
        .mount(&server)
        .await;

    let field = client
        .product_fields()
        .get(ProductFieldId(6247))
        .await
        .unwrap();
    assert!(field.value("low").is_ok());
    assert!(field.value("extreme").is_err());
}
