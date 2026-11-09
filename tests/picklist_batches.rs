mod common;

use std::num::NonZeroU32;

use common::{created, respond, setup};
use picqer_rs::Reported;
use picqer_rs::pagination::Page;
use picqer_rs::picklist_batch_presets::PresetStatus;
use picqer_rs::picklist_batches::{BatchPdf, BatchQuery, BatchStatus, BatchType, NewBatch};
use picqer_rs::picklists::PicklistStatus;
use picqer_rs::types::{
    NonEmpty, PicklistBatchId, PicklistBatchPresetId, PicklistId, ProductId, UserId, WarehouseId,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn batches_are_listed_without_their_products_and_picklists() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches"))
        .respond_with(respond("picklist_batches"))
        .expect(1)
        .mount(&server)
        .await;

    let batches = client
        .picklist_batches()
        .list(&BatchQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(batches.len(), 5);
    let completed = batches
        .iter()
        .find(|batch| batch.idpicklist_batch_preset.is_none())
        .unwrap();
    assert_eq!(
        completed.status,
        Some(Reported::Known(BatchStatus::Completed))
    );
    assert_eq!(completed.assigned_to, None);
    assert!(completed.completed_by.is_some());
    assert!(completed.completed_at.is_some());
    assert_eq!(completed.total_picklists, Some(0));
    let open = batches
        .iter()
        .find(|batch| batch.title.as_deref() == Some("Normal batch"))
        .unwrap();
    assert_eq!(open.status, Some(Reported::Known(BatchStatus::Open)));
    assert_eq!(open.batch_type, Some(Reported::Known(BatchType::Normal)));
    assert_eq!(open.title.as_deref(), Some("Normal batch"));
    assert_eq!(open.picklist_batchid.as_deref(), Some("1"));
    assert!(open.idpicklist_batch_preset.is_some());
    let user = open.assigned_to.as_ref().unwrap();
    assert_eq!(user.full_name.as_deref(), Some("Alex Example"));
    assert_eq!(user.username.as_deref(), Some("alex"));
    assert!(open.completed_by.is_none());
    assert!(open.products.is_empty() && open.picklists.is_empty());
}

#[tokio::test]
async fn a_single_batch_comes_with_its_products_and_picklists() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches/9"))
        .respond_with(respond("picklist_batch"))
        .mount(&server)
        .await;

    let batch = client
        .picklist_batches()
        .get(PicklistBatchId(9))
        .await
        .unwrap();
    assert_eq!(batch.batch_type, Some(Reported::Known(BatchType::Singles)));
    assert_eq!(batch.products.len(), 2);
    let product = &batch.products[0];
    assert!(product.amount.unwrap() > 0);
    assert_eq!(product.amount_picked, Some(0));
    assert!(!product.barcodes.is_empty());
    assert!(product.stock_location.is_some());
    assert_eq!(batch.picklists.len(), 3);
    let picklist = &batch.picklists[0];
    assert_eq!(picklist.status, Some(Reported::Known(PicklistStatus::New)));
    assert_eq!(picklist.alias.as_deref(), Some("A"));
    assert_eq!(picklist.has_notes, Some(false));
    assert!(picklist.created_at.is_some());
}

#[tokio::test]
async fn filters_become_query_parameters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches"))
        .and(query_param("idwarehouse", "2"))
        .and(query_param("assigned_to_iduser", "3"))
        .and(query_param("type", "singles"))
        .and(query_param("status", "completed"))
        .respond_with(respond("picklist_batches"))
        .expect(1)
        .mount(&server)
        .await;

    let query = BatchQuery::new()
        .idwarehouse(WarehouseId(2))
        .assigned_to(UserId(3))
        .batch_type(BatchType::Singles)
        .status(BatchStatus::Completed);
    client
        .picklist_batches()
        .list(&query, Page::FIRST)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_batch_is_created_from_picklists_or_from_a_preset() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picklists/batches"))
        .and(body_json(json!({"idpicklists": [422, 763]})))
        .respond_with(created("picklist_batch"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/picklists/batches"))
        .and(body_json(json!({
            "idpicklist_batch_preset": 5,
            "preferred_number_of_picklists": 1
        })))
        .respond_with(created("picklist_batch"))
        .expect(1)
        .mount(&server)
        .await;

    let batches = client.picklist_batches();
    let from_picklists =
        NewBatch::from_picklists(NonEmpty::new(vec![PicklistId(422), PicklistId(763)]).unwrap());
    assert_eq!(
        batches
            .create(&from_picklists)
            .await
            .unwrap()
            .products
            .len(),
        2
    );
    let from_preset = NewBatch::from_preset(PicklistBatchPresetId(5))
        .preferred_number_of_picklists(NonZeroU32::new(1).unwrap());
    batches.create(&from_preset).await.unwrap();
}

#[tokio::test]
async fn picklists_are_added_removed_and_the_batch_is_assigned() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picklists/batches/9/picklists"))
        .and(body_json(json!({"idpicklist": 382})))
        .respond_with(created("picklist_batch"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/picklists/batches/9/picklists/382"))
        .respond_with(respond("picklist_batch"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/picklists/batches/9/assign"))
        .and(body_json(json!({"iduser": 233})))
        .respond_with(respond("picklist_batch"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/picklists/batches/9/assign"))
        .and(body_json(json!({"iduser": null})))
        .respond_with(respond("picklist_batch"))
        .expect(1)
        .mount(&server)
        .await;

    let batches = client.picklist_batches();
    let id = PicklistBatchId(9);
    batches.add_picklist(id, PicklistId(382)).await.unwrap();
    batches.remove_picklist(id, PicklistId(382)).await.unwrap();
    batches.assign(id, Some(UserId(233))).await.unwrap();
    batches.assign(id, None).await.unwrap();
}

#[tokio::test]
async fn the_picklists_of_a_product_in_a_batch_are_whole_picklists() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches/9/products/52948001/picklists"))
        .respond_with(respond("picklist_batch_product_picklists"))
        .mount(&server)
        .await;

    let picklists = client
        .picklist_batches()
        .picklists_of_product(PicklistBatchId(9), ProductId(52948001))
        .await
        .unwrap();
    assert_eq!(picklists.len(), 2);
    assert_eq!(picklists[0].idpicklist_batch.map(|id| id.0), Some(9772360));
    assert!(!picklists[0].products.is_empty());
}

#[tokio::test]
async fn the_pdf_of_a_batch_is_downloaded_as_it_is() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches/9/pdf"))
        .and(query_param_is_missing("includePicklists"))
        .and(query_param_is_missing("includePackinglists"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(b"%PDF-1.7 batch".to_vec(), "application/pdf"),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches/9/pdf"))
        .and(query_param("includePicklists", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(b"%PDF-1.7 picklists".to_vec(), "application/pdf"),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches/9/pdf"))
        .and(query_param("includePackinglists", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(b"%PDF-1.7 packinglists".to_vec(), "application/pdf"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let batches = client.picklist_batches();
    let id = PicklistBatchId(9);
    assert_eq!(
        batches.pdf(id, BatchPdf::Batch).await.unwrap(),
        b"%PDF-1.7 batch"
    );
    assert_eq!(
        batches.pdf(id, BatchPdf::WithPicklists).await.unwrap(),
        b"%PDF-1.7 picklists"
    );
    assert_eq!(
        batches.pdf(id, BatchPdf::WithPackinglists).await.unwrap(),
        b"%PDF-1.7 packinglists"
    );
}

#[tokio::test]
async fn a_refused_picklist_comes_back_as_an_api_error() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picklists/batches/9/picklists"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": true, "error_code": 33, "error_message": "Cannot add the picklist, the batch is completed."
        })))
        .mount(&server)
        .await;

    let refused = client
        .picklist_batches()
        .add_picklist(PicklistBatchId(9), PicklistId(1))
        .await
        .unwrap_err();
    assert_eq!(
        refused.as_api().unwrap().code,
        Some(Reported::Known(picqer_rs::ErrorCode::WrongStatus))
    );
}

#[tokio::test]
async fn presets_are_listed_and_fetched() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches/presets"))
        .and(query_param_is_missing("inactive"))
        .respond_with(respond("picklist_batch_presets"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches/presets"))
        .and(query_param("inactive", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/picklists/batches/presets/18113"))
        .respond_with(respond("picklist_batch_preset"))
        .mount(&server)
        .await;

    let presets = client.picklist_batch_presets();
    let active = presets.list_all(PresetStatus::Active).await.unwrap();
    assert_eq!(active.len(), 2);
    assert_eq!(active[0].active, Some(true));
    assert_eq!(active[0].picklists_count, None);
    assert!(
        presets
            .list_all(PresetStatus::Inactive)
            .await
            .unwrap()
            .is_empty()
    );
    let single = presets.get(PicklistBatchPresetId(18113)).await.unwrap();
    assert_eq!(single.name.as_deref(), Some("Singles batch"));
    assert_eq!(single.picklists_count, Some(211));
}
