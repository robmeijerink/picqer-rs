mod common;

use std::num::NonZeroU32;

use common::{created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::pagination::Page;
use picqer_rs::products::LotNumber;
use picqer_rs::receipts::{
    NewReceipt, ReceiptQuery, ReceiptStatus, ReceiveProduct, ReceiveStrategy, StrategyKind,
};
use picqer_rs::types::{
    LocationId, ProductId, PurchaseOrderId, PurchaseOrderProductId, ReceiptId, ReceiptProductId,
    SupplierId, WarehouseId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn receipts_are_listed_with_what_was_received() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/receipts"))
        .and(query_param_is_missing("status"))
        .respond_with(respond("receipts"))
        .expect(1)
        .mount(&server)
        .await;

    let receipts = client
        .receipts()
        .list(&ReceiptQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(receipts.len(), 2);
    let receipt = receipts
        .iter()
        .find(|receipt| receipt.receiptid.as_deref() == Some("RC2026-1000"))
        .unwrap();
    assert_eq!(
        receipt.status,
        Some(Reported::Known(ReceiptStatus::Completed))
    );
    assert_eq!(receipt.version, Some(2));
    assert_eq!(receipt.amount_received, Some(4));
    assert_eq!(
        receipt.supplier.as_ref().unwrap().name.as_deref(),
        Some("Fitness Plaza")
    );
    assert!(receipt.purchaseorder.is_some());
    let user = receipt.completed_by.as_ref().unwrap();
    assert_eq!(user.name.as_deref(), Some("Alex Example"));
    assert!(receipt.completed_at.is_some());
    assert_eq!(receipt.products.len(), 2);
    assert_eq!(receipt.products[0].location, None);
}

#[tokio::test]
async fn a_single_receipt_says_where_each_product_went() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/receipts/5227431"))
        .respond_with(respond("receipt"))
        .mount(&server)
        .await;

    let receipt = client.receipts().get(ReceiptId(5227431)).await.unwrap();
    let line = &receipt.products[0];
    assert_eq!(line.amount, Some(3));
    assert_eq!(line.amount_ordered, Some(3));
    assert_eq!(line.added_by_receipt, Some(false));
    assert_eq!(line.lot_number, None);
    assert_eq!(line.reverted_at, None);
    assert!(line.created_at.is_some());
    let location = line.location.as_ref().unwrap();
    assert_eq!(location.name, "B.6.1.3");
}

#[tokio::test]
async fn a_completed_receipt_keeps_a_reverted_product() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/receipts/5227599"))
        .respond_with(respond("receipt_completed_reverted"))
        .mount(&server)
        .await;

    let receipt = client.receipts().get(ReceiptId(5227599)).await.unwrap();
    assert_eq!(
        receipt.status,
        Some(Reported::Known(ReceiptStatus::Completed))
    );
    assert_eq!(receipt.amount_received, Some(0));
    let line = &receipt.products[0];
    assert!(line.reverted_at.is_some());
    assert!(line.reverted_by_iduser.is_some());
}

#[tokio::test]
async fn a_receipt_of_version_1_is_read_with_what_it_has() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/receipts/5227596"))
        .respond_with(respond("receipt_v1"))
        .mount(&server)
        .await;

    let receipt = client.receipts().get(ReceiptId(5227596)).await.unwrap();
    assert_eq!(receipt.version, Some(1));
    assert_eq!(
        receipt.status,
        Some(Reported::Known(ReceiptStatus::Processing))
    );
    assert_eq!(receipt.products[0].amount, Some(0));
    assert_eq!(receipt.products[0].location, None);
}

#[tokio::test]
async fn filters_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/receipts"))
        .and(query_param("receiptid", "RC2026-1000"))
        .and(query_param("status", "completed"))
        .and(query_param("idpurchaseorder", "5"))
        .and(query_param("idsupplier", "6"))
        .and(query_param("updated_after", "2026-11-09 16:00:00"))
        .and(query_param("completed_after", "2026-11-09 16:00:00"))
        .and(query_param("include_indirect_receipts", "true"))
        .respond_with(respond("receipts"))
        .expect(1)
        .mount(&server)
        .await;

    let moment = "2026-11-09 16:00:00".parse().unwrap();
    let query = ReceiptQuery::new()
        .receiptid("RC2026-1000")
        .status(ReceiptStatus::Completed)
        .idpurchaseorder(PurchaseOrderId(5))
        .include_indirect_receipts()
        .idsupplier(SupplierId(6))
        .updated_after(moment)
        .completed_after(moment);
    client.receipts().list(&query, Page::FIRST).await.unwrap();
}

#[tokio::test]
async fn a_receipt_is_made_completed_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/receipts"))
        .and(body_json(json!({"idpurchaseorder": 3452022, "version": 2})))
        .respond_with(created("receipt_created"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/receipts"))
        .and(body_json(
            json!({"idsupplier": 229504, "idwarehouse": 14160, "version": 2}),
        ))
        .respond_with(created("receipt_created"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/receipts/5227599"))
        .and(body_json(json!({"status": "completed"})))
        .respond_with(respond("receipt_completed_reverted"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/receipts/5227599"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let receipts = client.receipts();
    let made = receipts
        .create(NewReceipt::for_purchase_order(PurchaseOrderId(3452022)))
        .await
        .unwrap();
    assert_eq!(
        made.status,
        Some(Reported::Known(ReceiptStatus::Processing))
    );
    assert_eq!(made.amount_received, Some(0));
    assert!(made.products.is_empty());
    receipts
        .create(NewReceipt::for_supplier(SupplierId(229504)).idwarehouse(WarehouseId(14160)))
        .await
        .unwrap();
    receipts.complete(ReceiptId(5227599)).await.unwrap();
    receipts.delete(ReceiptId(5227599)).await.unwrap();
}

#[tokio::test]
async fn the_expected_products_come_with_the_ways_to_receive_them() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/receipts/9/expected-products"))
        .respond_with(respond("receipt_expected_products"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/receipts/9/expected-products/52975317"))
        .respond_with(respond("receipt_expected_product"))
        .mount(&server)
        .await;

    let expected = client
        .receipts()
        .expected_products(ReceiptId(9))
        .await
        .unwrap();
    assert_eq!(expected.len(), 1);
    let one = &expected[0];
    let totals = one.totals.as_ref().unwrap();
    assert_eq!(
        totals.remaining,
        Some(totals.expected.unwrap() - totals.received.unwrap())
    );
    let kinds: Vec<_> = one
        .strategies
        .iter()
        .map(|strategy| strategy.strategy.clone())
        .collect();
    assert_eq!(
        kinds,
        vec![
            Reported::Known(StrategyKind::Automatic),
            Reported::Known(StrategyKind::PurchaseorderProduct),
            Reported::Known(StrategyKind::New),
        ]
    );
    let line = &one.strategies[1];
    assert!(
        line.parameters
            .as_ref()
            .unwrap()
            .idpurchaseorder_product
            .is_some()
    );
    assert_eq!(
        line.remaining,
        line.ordered.map(|ordered| ordered - line.received.unwrap())
    );
    assert!(one.strategies[2].idpurchaseorder.is_some());
    assert_eq!(one.strategies[2].ordered, None);
    assert_eq!(
        client
            .receipts()
            .expected_product(ReceiptId(9), ProductId(52975317))
            .await
            .unwrap(),
        *one
    );
}

#[tokio::test]
async fn a_product_is_received_and_reverted() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/receipts/9/products"))
        .and(body_json(json!({
            "idproduct": 78, "amount": 2, "idpurchaseorder_product": 493,
            "idlocation": 664, "lot_number": "LOT-2026-001", "expiration_date": "2027-01-15"
        })))
        .respond_with(created_list())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/receipts/9/products"))
        .and(body_json(
            json!({"idproduct": 78, "amount": 5, "strategy": "automatic"}),
        ))
        .respond_with(created_list())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/receipts/9/products/128535534/revert"))
        .respond_with(respond("receipt_product_reverted"))
        .expect(1)
        .mount(&server)
        .await;

    let receipts = client.receipts();
    let received = receipts
        .receive(
            ReceiptId(9),
            &ReceiveProduct::new(
                ProductId(78),
                NonZeroU32::new(2).unwrap(),
                ReceiveStrategy::PurchaseOrderProduct(PurchaseOrderProductId(493)),
            )
            .idlocation(LocationId(664))
            .lot_number(LotNumber::new("LOT-2026-001").unwrap())
            .expiration_date("2027-01-15".parse().unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].reverted_at, None);
    receipts
        .receive(
            ReceiptId(9),
            &ReceiveProduct::new(
                ProductId(78),
                NonZeroU32::new(5).unwrap(),
                ReceiveStrategy::Automatic,
            ),
        )
        .await
        .unwrap();
    let reverted = receipts
        .revert(ReceiptId(9), ReceiptProductId(128535534))
        .await
        .unwrap();
    assert!(reverted.reverted_at.is_some());
    assert!(reverted.reverted_by_iduser.is_some());
}

fn created_list() -> ResponseTemplate {
    let one = common::json("receipt_product");
    ResponseTemplate::new(201).set_body_json(json!([one]))
}

#[tokio::test]
async fn a_refused_lot_comes_back_with_its_code() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/receipts/9/products"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": true, "error_code": 44,
            "error_message": "You cannot move lot L2 to a location with lot L1."
        })))
        .mount(&server)
        .await;

    let refused = client
        .receipts()
        .receive(
            ReceiptId(9),
            &ReceiveProduct::new(
                ProductId(78),
                NonZeroU32::new(1).unwrap(),
                ReceiveStrategy::New,
            )
            .idlocation(LocationId(1)),
        )
        .await
        .unwrap_err();
    assert_eq!(
        refused.as_api().unwrap().code,
        Some(Reported::Known(picqer_rs::ErrorCode::InvalidLocation))
    );
}
