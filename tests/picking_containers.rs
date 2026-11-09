mod common;

use common::{respond, setup};
use picqer_rs::pagination::Page;
use picqer_rs::picking_containers::{PickingContainerName, PickingContainerQuery};
use picqer_rs::types::{PickingContainerId, PicklistId};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn containers_are_listed_with_the_picklist_they_hold() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picking-containers"))
        .respond_with(respond("picking_containers"))
        .expect(1)
        .mount(&server)
        .await;

    let containers = client
        .picking_containers()
        .list(PickingContainerQuery::new(), Page::FIRST)
        .await
        .unwrap();
    assert_eq!(containers.len(), 2);
    let linked = containers.iter().find(|c| c.name == "Blue bin").unwrap();
    assert!(linked.idpicklist.is_some());
    assert!(linked.created_at.is_some());
    let free = containers.iter().find(|c| c.name == "Red bin").unwrap();
    assert_eq!(free.idpicklist, None);
}

#[tokio::test]
async fn the_filter_becomes_a_query_parameter() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/picking-containers"))
        .and(query_param("idpicklist", "8"))
        .respond_with(respond("picking_containers"))
        .expect(1)
        .mount(&server)
        .await;

    client
        .picking_containers()
        .list_all(PickingContainerQuery::new().idpicklist(PicklistId(8)))
        .await
        .unwrap();
}

#[tokio::test]
async fn a_container_is_made_renamed_linked_and_deleted() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/picking-containers"))
        .and(body_json(json!({"name": "Red bin"})))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(common::json("picking_container_created")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/picking-containers/3"))
        .and(body_json(json!({"name": "Green bin"})))
        .respond_with(respond("picking_container_created"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/picking-containers/3/link"))
        .and(body_json(json!({"idpicklist": 8})))
        .respond_with(respond("picking_container"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/picking-containers/3"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let containers = client.picking_containers();
    let id = PickingContainerId(3);
    let made = containers
        .create(&PickingContainerName::new("Red bin").unwrap())
        .await
        .unwrap();
    assert_eq!(made.idpicklist, None);
    containers
        .rename(id, &PickingContainerName::new("Green bin").unwrap())
        .await
        .unwrap();
    let linked = containers.link(id, PicklistId(8)).await.unwrap();
    assert!(linked.idpicklist.is_some());
    containers.delete(id).await.unwrap();
}

#[tokio::test]
async fn a_name_that_is_a_number_is_read_as_text() {
    let (server, client) = setup().await;
    let mut body = common::json("picking_container_created");
    body["name"] = json!(5);
    Mock::given(method("GET"))
        .and(path("/picking-containers/4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let container = client
        .picking_containers()
        .get(PickingContainerId(4))
        .await
        .unwrap();
    assert_eq!(container.name, "5");
}
