#![cfg(feature = "blocking")]

mod common;

use picqer_rs::blocking::Client;
use picqer_rs::pagination::Page;
use picqer_rs::products::ProductQuery;
use picqer_rs::types::ProductId;
use picqer_rs::{ApiKey, Subdomain};
use url::Url;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer};

fn serve() -> (tokio::runtime::Runtime, MockServer) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let server = runtime.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/products/1"))
            .and(header("authorization", common::AUTH))
            .respond_with(common::respond("product_full"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/products"))
            .respond_with(common::respond("products"))
            .mount(&server)
            .await;
        server
    });
    (runtime, server)
}

fn client(server: &MockServer) -> Client {
    Client::builder(
        Subdomain::new("example").unwrap(),
        ApiKey::new("test-key").unwrap(),
        common::user_agent(),
    )
    .base_url(Url::parse(&server.uri()).unwrap())
    .build_blocking()
    .unwrap()
}

#[test]
fn the_blocking_client_runs_the_same_operations() {
    let (_runtime, server) = serve();
    let client = client(&server);

    let product = client.products().get(ProductId(1)).unwrap();
    assert!(!product.name.is_empty());
    let listed = client
        .products()
        .list(&ProductQuery::new(), Page::FIRST)
        .unwrap();
    assert_eq!(listed.len(), 2);
}
