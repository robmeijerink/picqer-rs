mod common;

use std::num::NonZeroU32;

use common::{created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::customers::Language;
use picqer_rs::pagination::Page;
use picqer_rs::purchase_orders::{
    NewPurchaseOrder, NewPurchaseOrderProduct, PurchaseOrderChanges, PurchaseOrderLineChanges,
    PurchaseOrderQuery, PurchaseOrderStatus,
};
use picqer_rs::suppliers::SupplierName;
use picqer_rs::types::{
    DateTime, Money, ProductId, PurchaseOrderId, PurchaseOrderProductId, SupplierId, WarehouseId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

fn amount(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

#[tokio::test]
async fn purchase_orders_are_listed_and_filtered() {
    let (server, client) = setup().await;
    Mock::given(path("/purchaseorders"))
        .and(query_param("status", "purchased"))
        .and(query_param("updated_after", "2026-11-09 00:00:00"))
        .respond_with(respond("purchaseorders"))
        .expect(1)
        .mount(&server)
        .await;

    let since: DateTime = "2026-11-09 00:00:00".parse().unwrap();
    let query = PurchaseOrderQuery::new()
        .status(PurchaseOrderStatus::Purchased)
        .updated_after(since);
    let orders = client
        .purchase_orders()
        .list(&query, Page::FIRST)
        .await
        .unwrap();
    assert_eq!(orders.len(), 2);
    assert_eq!(
        orders[0].status,
        Reported::Known(PurchaseOrderStatus::Concept)
    );
    assert_eq!(
        orders[1].status,
        Reported::Known(PurchaseOrderStatus::Purchased)
    );
    assert!(orders[1].purchased_at.is_some());
    assert!(orders[0].purchased_at.is_none());
    assert!(orders[0].products.len() >= 2);
    let line = &orders[0].products[0];
    assert!(line.price.unwrap() > Money::from_cents(0).unwrap());
    assert!(line.amount > 0);
    assert_eq!(line.amountreceived, Some(0));
    assert!(orders[0].delivery_date.is_some());
    assert_eq!(orders[0].language, Some(Reported::Known(Language::Nl)));
}

#[tokio::test]
async fn a_purchase_order_goes_from_concept_to_purchased_to_received() {
    let (server, client) = setup().await;
    for (action, fixture) in [
        ("mark-as-purchased", "purchaseorder_purchased"),
        ("close", "purchaseorder_received"),
        ("cancel", "purchaseorder_created"),
    ] {
        Mock::given(method("POST"))
            .and(path(format!("/purchaseorders/5/{action}")))
            .respond_with(respond(fixture))
            .expect(1)
            .mount(&server)
            .await;
    }

    let orders = client.purchase_orders();
    let purchased = orders.mark_as_purchased(PurchaseOrderId(5)).await.unwrap();
    assert_eq!(
        purchased.status,
        Reported::Known(PurchaseOrderStatus::Purchased)
    );
    assert!(purchased.purchased_at.is_some());
    let received = orders.close(PurchaseOrderId(5)).await.unwrap();
    assert_eq!(
        received.status,
        Reported::Known(PurchaseOrderStatus::Received)
    );
    assert!(received.completed_at.is_some());
    assert!(received.completed_by_iduser.is_some());
    orders.cancel(PurchaseOrderId(5)).await.unwrap();
}

#[tokio::test]
async fn an_order_is_created_for_a_supplier_by_id_or_by_name() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/purchaseorders"))
        .and(body_json(json!({
            "idsupplier": 3,
            "idwarehouse": 18,
            "language": "en",
            "delivery_date": "2030-02-03",
            "products": [{"idproduct": 147, "amount": 25, "price": 26.95}]
        })))
        .respond_with(created("purchaseorder_created"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/purchaseorders"))
        .and(body_json(json!({"supplier_name": "Acme"})))
        .respond_with(created("purchaseorder_created"))
        .expect(1)
        .mount(&server)
        .await;

    let line = NewPurchaseOrderProduct::new(ProductId(147), amount(25))
        .price(Money::from_cents(2695).unwrap());
    let order = NewPurchaseOrder::for_supplier(SupplierId(3))
        .warehouse(WarehouseId(18))
        .language(Language::En)
        .delivery_date("2030-02-03".parse().unwrap())
        .product(line);
    let made = client.purchase_orders().create(&order).await.unwrap();
    assert_eq!(made.status, Reported::Known(PurchaseOrderStatus::Concept));
    let by_name = NewPurchaseOrder::for_supplier_name(SupplierName::new("Acme").unwrap());
    client.purchase_orders().create(&by_name).await.unwrap();
}

#[tokio::test]
async fn an_order_is_changed_with_only_what_was_set() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/purchaseorders/5"))
        .and(body_json(
            json!({"remarks": "r", "supplier_orderid": "X-9"}),
        ))
        .respond_with(respond("purchaseorder_created"))
        .expect(1)
        .mount(&server)
        .await;

    let changes = PurchaseOrderChanges::new()
        .remarks("r")
        .supplier_orderid("X-9");
    client
        .purchase_orders()
        .update(PurchaseOrderId(5), &changes)
        .await
        .unwrap();
}

#[tokio::test]
async fn lines_are_listed_added_changed_and_removed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/purchaseorders/5/products"))
        .respond_with(respond("purchaseorder_lines"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/purchaseorders/5/products"))
        .and(body_json(
            json!({"idproduct": 9, "amount": 4, "price": 2.5}),
        ))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(common::json("purchaseorder_lines")[0].clone()),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/purchaseorders/5/products/7"))
        .and(body_json(json!({"amount": 7, "price": 3.25})))
        .respond_with(respond_one())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/purchaseorders/5/products/7"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let orders = client.purchase_orders();
    let id = PurchaseOrderId(5);
    assert_eq!(orders.lines(id).await.unwrap().len(), 1);
    let new = NewPurchaseOrderProduct::new(ProductId(9), amount(4))
        .price(Money::from_cents(250).unwrap());
    orders.add_line(id, &new).await.unwrap();
    let changes = PurchaseOrderLineChanges::new()
        .amount(amount(7))
        .price(Money::from_cents(325).unwrap());
    orders
        .update_line(id, PurchaseOrderProductId(7), &changes)
        .await
        .unwrap();
    orders
        .remove_line(id, PurchaseOrderProductId(7))
        .await
        .unwrap();
}

fn respond_one() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(common::json("purchaseorder_lines")[0].clone())
}

#[tokio::test]
async fn a_purchase_order_without_a_supplier_id_keeps_the_name_it_was_given() {
    let (server, client) = setup().await;
    let mut order = common::json("purchaseorder_created");
    order["idsupplier"] = json!(null);
    order["supplier_name"] = json!("Acme");
    order["status"] = json!("on_hold");
    Mock::given(path("/purchaseorders/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(order))
        .mount(&server)
        .await;

    let order = client
        .purchase_orders()
        .get(PurchaseOrderId(5))
        .await
        .unwrap();
    assert_eq!(order.idsupplier, None);
    assert_eq!(order.supplier_name.as_deref(), Some("Acme"));
    assert!(matches!(order.status, Reported::Unknown(_)));
}
