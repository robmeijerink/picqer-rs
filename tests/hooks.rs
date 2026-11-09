mod common;

use common::{respond, setup};
use picqer_rs::Reported;
use picqer_rs::types::HookId;
use picqer_rs::webhooks::{HookName, NewHook, WebhookEvent, WebhookSecret};
use serde_json::json;
use url::Url;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn hooks_are_listed_with_their_state() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/hooks"))
        .respond_with(respond("hooks"))
        .expect(1)
        .mount(&server)
        .await;

    let hooks = client.hooks().list_all().await.unwrap();
    assert_eq!(hooks.len(), 3);
    assert!(hooks.iter().all(|hook| hook.active == Some(false)));
    let signed = hooks.iter().find(|hook| hook.secret == Some(true)).unwrap();
    assert_eq!(signed.event, Reported::Known(WebhookEvent::MovementsMoved));
    assert!(signed.address.starts_with("https://"));
    assert_eq!(
        hooks
            .iter()
            .filter(|hook| hook.secret == Some(false))
            .count(),
        2
    );
}

#[tokio::test]
async fn a_hook_is_made_with_a_secret_and_deactivated_and_reactivated() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/hooks"))
        .and(body_json(json!({
            "name": "picqer-rs live hook secret", "event": "movements.moved",
            "address": "https://example.com/picqer-rs-live-secret", "secret": "my-secret-key"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(common::json("hook")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/hooks/70255"))
        .respond_with(respond("hook"))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/hooks/70255"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/hooks/70254/reactivate"))
        .respond_with(respond("hook_plain"))
        .expect(1)
        .mount(&server)
        .await;

    let hooks = client.hooks();
    let new = NewHook::new(
        HookName::new("picqer-rs live hook secret").unwrap(),
        WebhookEvent::MovementsMoved,
        Url::parse("https://example.com/picqer-rs-live-secret").unwrap(),
    )
    .unwrap()
    .secret(WebhookSecret::new("my-secret-key").unwrap());
    let made = hooks.create(&new).await.unwrap();
    assert_eq!(made.secret, Some(true));
    assert_eq!(hooks.get(HookId(70255)).await.unwrap(), made);
    hooks.deactivate(HookId(70255)).await.unwrap();
    let back = hooks.reactivate(HookId(70254)).await.unwrap();
    assert_eq!(back.secret, Some(false));
}

#[tokio::test]
async fn a_hook_that_exists_already_comes_back_as_an_error_with_a_code() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/hooks"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": true, "error_code": 5, "error_message": "Error: Hook already exists"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/hooks/1/reactivate"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": true, "error_code": 21,
            "error_message": "Item not found or does not have the right status"
        })))
        .mount(&server)
        .await;

    let new = NewHook::new(
        HookName::new("again").unwrap(),
        WebhookEvent::MovementsMoved,
        Url::parse("https://example.com/picqer-rs-live-secret").unwrap(),
    )
    .unwrap();
    let exists = client.hooks().create(&new).await.unwrap_err();
    assert_eq!(
        exists.as_api().unwrap().code,
        Some(Reported::Known(picqer_rs::ErrorCode::MissingParameters))
    );
    let active = client.hooks().reactivate(HookId(1)).await.unwrap_err();
    assert_eq!(
        active.as_api().unwrap().code,
        Some(Reported::Known(
            picqer_rs::ErrorCode::ItemNotFoundOrWrongStatus
        ))
    );
}
