mod common;

use common::{created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::products::{NewProductSupplier, ProductSupplierChanges, SupplierProductCode};
use picqer_rs::types::{ProductId, ProductSupplierId, SupplierId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn the_suppliers_of_a_product_are_listed_and_fetched() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products/9/suppliers"))
        .respond_with(respond("product_suppliers"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/products/9/suppliers/23180285"))
        .respond_with(respond("product_supplier"))
        .mount(&server)
        .await;

    let products = client.products();
    let suppliers = products.suppliers(ProductId(9)).await.unwrap();
    assert_eq!(suppliers.len(), 1);
    let supplier = &suppliers[0];
    assert_eq!(supplier.idproduct_supplier, ProductSupplierId(23180285));
    assert_eq!(supplier.idsupplier, Some(SupplierId(229481)));
    assert_eq!(supplier.productcode.as_deref(), Some("P-1"));
    assert_eq!(supplier.deliverytime, Some(3));
    assert_eq!(supplier.minimum_purchase_quantity, Some(10));
    assert_eq!(supplier.purchase_in_quantities_of, Some(5));
    assert_eq!(supplier.is_default, Some(true));

    let one = products
        .supplier(ProductId(9), ProductSupplierId(23180285))
        .await
        .unwrap();
    assert_eq!(&one, supplier);
}

#[tokio::test]
async fn purchase_details_without_a_supplier_have_no_idsupplier() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products/9/suppliers"))
        .respond_with(respond("product_supplier_without_supplier"))
        .mount(&server)
        .await;

    let suppliers = client.products().suppliers(ProductId(9)).await.unwrap();
    assert_eq!(suppliers[0].idsupplier, None);
    assert_eq!(suppliers[0].is_default, Some(true));
}

#[tokio::test]
async fn a_product_with_more_suppliers_lists_them_default_first() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/products/9/suppliers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "idproduct_supplier": 88104, "idsupplier": 3310, "productcode": "HK-VEST-RB-79",
                "deliverytime": 2, "minimum_purchase_quantity": null,
                "purchase_in_quantities_of": 12, "is_default": true
            },
            {
                "idproduct_supplier": 88213, "idsupplier": 4521, "productcode": "SUP-6531-RB",
                "deliverytime": 5, "minimum_purchase_quantity": 10,
                "purchase_in_quantities_of": 5, "is_default": false
            }
        ])))
        .mount(&server)
        .await;

    let suppliers = client.products().suppliers(ProductId(9)).await.unwrap();
    assert_eq!(suppliers.len(), 2);
    assert_eq!(suppliers[0].is_default, Some(true));
    assert_eq!(suppliers[1].idproduct_supplier, ProductSupplierId(88213));
    assert_eq!(suppliers[1].is_default, Some(false));
}

#[tokio::test]
async fn a_supplier_is_linked_changed_and_unlinked() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/products/9/suppliers"))
        .and(body_json(json!({
            "idsupplier": 229481, "productcode": "P-1", "deliverytime": 3,
            "minimum_purchase_quantity": 10, "purchase_in_quantities_of": 5,
            "is_default": true
        })))
        .respond_with(created("product_supplier"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/products/9/suppliers/23180285"))
        .and(body_json(json!({
            "productcode": null, "deliverytime": 4, "is_default": true
        })))
        .respond_with(respond("product_supplier"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/products/9/suppliers/23180285"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let products = client.products();
    let linked = products
        .add_supplier(
            ProductId(9),
            &NewProductSupplier::new(SupplierId(229481))
                .productcode(SupplierProductCode::new("P-1").unwrap())
                .deliverytime(3)
                .minimum_purchase_quantity(10)
                .purchase_in_quantities_of(5)
                .make_default(),
        )
        .await
        .unwrap();
    assert_eq!(linked.idproduct_supplier, ProductSupplierId(23180285));
    products
        .update_supplier(
            ProductId(9),
            ProductSupplierId(23180285),
            &ProductSupplierChanges::new()
                .without_productcode()
                .deliverytime(4)
                .make_default(),
        )
        .await
        .unwrap();
    products
        .remove_supplier(ProductId(9), ProductSupplierId(23180285))
        .await
        .unwrap();
}

#[tokio::test]
async fn a_second_supplier_on_a_fulfilment_account_comes_back_with_code_31() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/products/9/suppliers"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": true, "error_code": 31,
            "error_message": "Product 9 already has a supplier, and fulfilment companies cannot add extra suppliers"
        })))
        .mount(&server)
        .await;

    let error = client
        .products()
        .add_supplier(ProductId(9), &NewProductSupplier::new(SupplierId(2)))
        .await
        .unwrap_err();
    let api = error.as_api().unwrap();
    assert_eq!(api.status, 400);
    assert_eq!(
        api.code,
        Some(Reported::Known(
            picqer_rs::ErrorCode::NotAvailableForFulfilment
        ))
    );
}
