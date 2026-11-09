#![allow(dead_code)]

use picqer_rs::{ApiKey, Client, Subdomain, UserAgent};
use url::Url;
use wiremock::{MockServer, ResponseTemplate};

pub const AUTH: &str = "Basic dGVzdC1rZXk6";

pub fn user_agent() -> UserAgent {
    UserAgent::new("Tests", "dev@example.com").unwrap()
}

pub async fn setup() -> (MockServer, Client) {
    let server = MockServer::start().await;
    let client = Client::builder(
        Subdomain::new("example").unwrap(),
        ApiKey::new("test-key").unwrap(),
        user_agent(),
    )
    .base_url(Url::parse(&server.uri()).unwrap())
    .build()
    .unwrap();
    (server, client)
}

pub fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

pub fn json(name: &str) -> serde_json::Value {
    serde_json::from_str(&fixture(name)).unwrap()
}

pub fn respond(name: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(fixture(name), "application/json")
}

pub fn created(name: &str) -> ResponseTemplate {
    ResponseTemplate::new(201).set_body_raw(fixture(name), "application/json")
}

pub fn error(status: u16, name: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_raw(fixture(name), "application/json")
}
