mod common;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::stats::StatKey;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn the_stats_are_listed_with_what_they_tell() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/stats"))
        .respond_with(respond("stats"))
        .expect(1)
        .mount(&server)
        .await;

    let stats = client.stats().list().await.unwrap();
    assert_eq!(stats.len(), 12);
    assert!(
        stats
            .iter()
            .all(|(key, _)| matches!(key, Reported::Known(_)))
    );
    let open = stats
        .iter()
        .find(|(key, _)| *key == Reported::Known(StatKey::OpenOrders))
        .unwrap();
    assert_eq!(open.1.description, "Number of open orders");
}

#[tokio::test]
async fn a_stat_is_a_number() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/stats/open-orders"))
        .respond_with(respond("stat"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/stats/open-tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": 3})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/stats/bogus"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": true, "error_code": 20, "error_message": "Route or resource not found"
        })))
        .mount(&server)
        .await;

    assert_eq!(client.stats().get(StatKey::OpenOrders).await.unwrap(), 161);
    assert_eq!(client.stats().get_by_key("open-tasks").await.unwrap(), 3);
    assert!(
        client
            .stats()
            .get_by_key("bogus")
            .await
            .unwrap_err()
            .is_not_found()
    );
}
