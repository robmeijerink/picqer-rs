use std::fmt;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use url::Url;

use crate::auth::ApiKey;
use crate::error::{Error, Result, ValidationError};
use crate::http::{
    CONNECT_TIMEOUT, DEFAULT_TIMEOUT, Operation, Response, Transport, default_base_url,
    validate_base_url,
};
use crate::rate_limit::{Limiter, RateLimit, until_next_minute};
use crate::types::{Subdomain, UserAgent};

/// Asynchronous client for the Picqer API.
///
/// Cloning is cheap; clones share the connection pool and what is known about the rate limit.
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    http: reqwest::Client,
    transport: Transport,
    limiter: Limiter,
    retries: u32,
    throttle: bool,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client").finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a client with default settings.
    pub fn new(subdomain: Subdomain, api_key: ApiKey, user_agent: UserAgent) -> Result<Self> {
        Self::builder(subdomain, api_key, user_agent).build()
    }

    /// Creates a client with the subdomain from `PICQER_SUBDOMAIN` and the key from
    /// `PICQER_API_KEY`.
    pub fn from_env(user_agent: UserAgent) -> Result<Self> {
        Self::new(Subdomain::from_env()?, ApiKey::from_env()?, user_agent)
    }

    /// Returns a builder for customising the client.
    pub fn builder(subdomain: Subdomain, api_key: ApiKey, user_agent: UserAgent) -> ClientBuilder {
        ClientBuilder::new(subdomain, api_key, user_agent)
    }

    /// What the API reported about the rate limit with its last response, if it reported anything.
    pub fn rate_limit(&self) -> Option<RateLimit> {
        self.inner.limiter.current()
    }

    /// Operations on comments.
    pub fn comments(&self) -> crate::comments::Comments<'_> {
        crate::comments::Comments { client: self }
    }

    /// Operations on customers.
    pub fn customers(&self) -> crate::customers::Customers<'_> {
        crate::customers::Customers { client: self }
    }

    /// Operations on order fields.
    pub fn order_fields(&self) -> crate::order_fields::OrderFields<'_> {
        crate::order_fields::OrderFields { client: self }
    }

    /// Operations on customer fields.
    pub fn customer_fields(&self) -> crate::customer_fields::CustomerFields<'_> {
        crate::customer_fields::CustomerFields { client: self }
    }

    /// Operations on orders.
    pub fn orders(&self) -> crate::orders::Orders<'_> {
        crate::orders::Orders { client: self }
    }

    /// Operations on picklists.
    pub fn picklists(&self) -> crate::picklists::Picklists<'_> {
        crate::picklists::Picklists { client: self }
    }

    /// Operations on packagings.
    pub fn packagings(&self) -> crate::packagings::Packagings<'_> {
        crate::packagings::Packagings { client: self }
    }

    /// Operations on picklist batches.
    pub fn picklist_batches(&self) -> crate::picklist_batches::PicklistBatches<'_> {
        crate::picklist_batches::PicklistBatches { client: self }
    }

    /// Operations on the presets of picklist batches.
    pub fn picklist_batch_presets(
        &self,
    ) -> crate::picklist_batch_presets::PicklistBatchPresets<'_> {
        crate::picklist_batch_presets::PicklistBatchPresets { client: self }
    }

    /// Operations on location types.
    pub fn location_types(&self) -> crate::location_types::LocationTypes<'_> {
        crate::location_types::LocationTypes { client: self }
    }

    /// Operations on locations and containers.
    pub fn locations(&self) -> crate::locations::Locations<'_> {
        crate::locations::Locations { client: self }
    }

    /// Operations on the stock history of locations.
    pub fn location_stock_history(
        &self,
    ) -> crate::location_stock_history::LocationStockHistory<'_> {
        crate::location_stock_history::LocationStockHistory { client: self }
    }

    /// Operations on picking containers.
    pub fn picking_containers(&self) -> crate::picking_containers::PickingContainers<'_> {
        crate::picking_containers::PickingContainers { client: self }
    }

    /// Operations on hooks, the subscriptions to webhook events.
    pub fn hooks(&self) -> crate::webhooks::Hooks<'_> {
        crate::webhooks::Hooks { client: self }
    }

    /// Operations on stats.
    pub fn stats(&self) -> crate::stats::Stats<'_> {
        crate::stats::Stats { client: self }
    }

    /// Operations on the orders of webshop integrations.
    pub fn webshop_orders(&self) -> crate::webshop_orders::WebshopOrders<'_> {
        crate::webshop_orders::WebshopOrders { client: self }
    }

    /// Operations on fulfilment customers.
    pub fn fulfilment_customers(&self) -> crate::fulfilment::FulfilmentCustomers<'_> {
        crate::fulfilment::FulfilmentCustomers { client: self }
    }

    /// Operations on tasks.
    pub fn tasks(&self) -> crate::tasks::Tasks<'_> {
        crate::tasks::Tasks { client: self }
    }

    /// Operations on backorders.
    pub fn backorders(&self) -> crate::backorders::Backorders<'_> {
        crate::backorders::Backorders { client: self }
    }

    /// Operations on returns.
    pub fn returns(&self) -> crate::returns::Returns<'_> {
        crate::returns::Returns { client: self }
    }

    /// Operations on the statuses of returns.
    pub fn return_statuses(&self) -> crate::returns::ReturnStatuses<'_> {
        crate::returns::ReturnStatuses { client: self }
    }

    /// Operations on the reasons for returns.
    pub fn return_reasons(&self) -> crate::returns::ReturnReasons<'_> {
        crate::returns::ReturnReasons { client: self }
    }

    /// Operations on receipts.
    pub fn receipts(&self) -> crate::receipts::Receipts<'_> {
        crate::receipts::Receipts { client: self }
    }

    /// Operations on shipments.
    pub fn shipments(&self) -> crate::shipments::Shipments<'_> {
        crate::shipments::Shipments { client: self }
    }

    /// Operations on shipping providers.
    pub fn shipping_providers(&self) -> crate::shipping_providers::ShippingProviders<'_> {
        crate::shipping_providers::ShippingProviders { client: self }
    }

    /// Operations on templates.
    pub fn templates(&self) -> crate::templates::Templates<'_> {
        crate::templates::Templates { client: self }
    }

    /// Operations on users.
    pub fn users(&self) -> crate::users::Users<'_> {
        crate::users::Users { client: self }
    }

    /// Operations on packing stations.
    pub fn packing_stations(&self) -> crate::packing_stations::PackingStations<'_> {
        crate::packing_stations::PackingStations { client: self }
    }

    /// Operations on products.
    pub fn products(&self) -> crate::products::Products<'_> {
        crate::products::Products { client: self }
    }

    /// Operations on warehouses.
    pub fn warehouses(&self) -> crate::warehouses::Warehouses<'_> {
        crate::warehouses::Warehouses { client: self }
    }

    /// Operations on VAT groups.
    pub fn vat_groups(&self) -> crate::vat_groups::VatGroups<'_> {
        crate::vat_groups::VatGroups { client: self }
    }

    /// Operations on the stock history.
    pub fn stock_history(&self) -> crate::stock_history::StockHistory<'_> {
        crate::stock_history::StockHistory { client: self }
    }

    /// Operations on purchase orders.
    pub fn purchase_orders(&self) -> crate::purchase_orders::PurchaseOrders<'_> {
        crate::purchase_orders::PurchaseOrders { client: self }
    }

    /// Operations on suppliers.
    pub fn suppliers(&self) -> crate::suppliers::Suppliers<'_> {
        crate::suppliers::Suppliers { client: self }
    }

    /// Operations on tags.
    pub fn tags(&self) -> crate::tags::Tags<'_> {
        crate::tags::Tags { client: self }
    }

    /// Operations on pricelists.
    pub fn pricelists(&self) -> crate::pricelists::Pricelists<'_> {
        crate::pricelists::Pricelists { client: self }
    }

    /// Operations on product fields.
    pub fn product_fields(&self) -> crate::product_fields::ProductFields<'_> {
        crate::product_fields::ProductFields { client: self }
    }

    pub(crate) async fn execute<O: Operation>(&self, operation: O) -> Result<O::Output> {
        let inner = &self.inner;
        let request = operation.request()?;
        let url = inner.transport.url(&request);
        let headers = inner.transport.headers(&request)?;
        let mut retries = inner.retries;
        loop {
            if inner.throttle
                && let Some(wait) = inner.limiter.throttle_wait()
            {
                tokio::time::sleep(wait).await;
            }
            let mut builder = inner
                .http
                .request(request.method.clone(), url.clone())
                .headers(headers.clone());
            if let Some(body) = &request.body {
                builder = builder.body(body.clone());
            }
            let response = builder.send().await?;
            let status = response.status().as_u16();
            let headers = response.headers().clone();
            let body = response.bytes().await?.to_vec();
            inner.limiter.record(&headers);
            if status == 429 {
                let resets_in = until_next_minute(&headers);
                if retries == 0 {
                    return Err(Error::RateLimited { resets_in });
                }
                retries -= 1;
                tokio::time::sleep(resets_in).await;
                continue;
            }
            return operation.parse(Response { status, body });
        }
    }
}

pub(crate) struct Settings {
    pub transport: Transport,
    pub http: Option<reqwest::Client>,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub retries: u32,
    pub throttle: bool,
}

/// Builder for [`Client`] and `blocking::Client`.
pub struct ClientBuilder {
    api_key: ApiKey,
    base_url: Url,
    user_agent: UserAgent,
    timeout: Duration,
    connect_timeout: Duration,
    http: Option<reqwest::Client>,
    retries: Option<NonZeroU32>,
    throttle: bool,
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("base_url", &self.base_url.as_str())
            .finish_non_exhaustive()
    }
}

impl ClientBuilder {
    pub(crate) fn new(subdomain: Subdomain, api_key: ApiKey, user_agent: UserAgent) -> Self {
        Self {
            api_key,
            base_url: default_base_url(&subdomain),
            user_agent,
            timeout: DEFAULT_TIMEOUT,
            connect_timeout: CONNECT_TIMEOUT,
            http: None,
            retries: None,
            throttle: false,
        }
    }

    /// Replaces the base url, which is `https://{subdomain}.picqer.com/api/v1/` by default. It
    /// must use `https`; `http` is only accepted for loopback hosts, which is useful for mock
    /// servers.
    pub fn base_url(mut self, url: Url) -> Self {
        self.base_url = url;
        self
    }

    /// Sets the timeout of a whole request. The default is 60 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the timeout for establishing a connection. The default is 10 seconds.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Uses an existing HTTP client. Its settings apply as they are, so make sure it does not
    /// follow redirects.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }

    /// Sends a request again, at most `retries` times, when the rate limit was reached.
    ///
    /// The client waits until the next full minute, when the count starts again. Picqer ignores a
    /// request it answers with `429`, so sending it again cannot do the work twice. Without this,
    /// the request fails with [`Error::RateLimited`].
    pub fn retry_on_rate_limit(mut self, retries: NonZeroU32) -> Self {
        self.retries = Some(retries);
        self
    }

    /// Waits until the next full minute before sending a request when the last response reported
    /// that no requests were left.
    ///
    /// Requests that are in flight at the same time are not counted, so this lowers the number of
    /// `429` responses but does not rule them out.
    pub fn throttle(mut self) -> Self {
        self.throttle = true;
        self
    }

    pub(crate) fn into_settings(self) -> std::result::Result<Settings, ValidationError> {
        validate_base_url(&self.base_url)?;
        Ok(Settings {
            transport: Transport::new(self.api_key, self.base_url, self.user_agent),
            http: self.http,
            timeout: self.timeout,
            connect_timeout: self.connect_timeout,
            retries: self.retries.map_or(0, NonZeroU32::get),
            throttle: self.throttle,
        })
    }

    /// Builds the asynchronous client.
    pub fn build(self) -> Result<Client> {
        let settings = self.into_settings()?;
        let http = match settings.http {
            Some(http) => http,
            None => reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(settings.timeout)
                .connect_timeout(settings.connect_timeout)
                .build()?,
        };
        Ok(Client {
            inner: Arc::new(Inner {
                http,
                transport: settings.transport,
                limiter: Limiter::default(),
                retries: settings.retries,
                throttle: settings.throttle,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use reqwest::Method;
    use serde_json::{Value, json};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::http::Request;

    struct Ping;

    impl Operation for Ping {
        type Output = Value;

        fn request(&self) -> Result<Request> {
            Ok(Request::new(Method::GET, vec!["ping".into()]))
        }

        fn parse(&self, response: Response) -> Result<Value> {
            response.ensure_success()?.json()
        }
    }

    fn builder(server: &MockServer) -> ClientBuilder {
        Client::builder(
            Subdomain::new("example").unwrap(),
            ApiKey::new("test-key").unwrap(),
            UserAgent::new("Tests", "dev@example.com").unwrap(),
        )
        .base_url(Url::parse(&server.uri()).unwrap())
    }

    fn limited(date: &str, remaining: &str) -> ResponseTemplate {
        ResponseTemplate::new(200)
            .insert_header("date", date)
            .insert_header("x-ratelimit-limit", "500")
            .insert_header("x-ratelimit-remaining", remaining)
            .set_body_json(json!({"ok": true}))
    }

    fn too_many(date: &str) -> ResponseTemplate {
        ResponseTemplate::new(429)
            .insert_header("date", date)
            .set_body_json(
                json!({"error": true, "error_code": 28, "error_message": "Rate limit exceeded"}),
            )
    }

    const SECOND_59: &str = "Mon, 09 Nov 2026 12:29:59 GMT";
    const SECOND_30: &str = "Mon, 09 Nov 2026 12:29:30 GMT";

    #[tokio::test]
    async fn requests_carry_the_credentials_and_the_identity() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ping"))
            .and(header("authorization", "Basic dGVzdC1rZXk6"))
            .and(header("accept", "application/json"))
            .and(header(
                "user-agent",
                format!(
                    "Tests (dev@example.com) picqer-rs/{}",
                    env!("CARGO_PKG_VERSION")
                )
                .as_str(),
            ))
            .respond_with(limited(SECOND_30, "499"))
            .expect(1)
            .mount(&server)
            .await;

        let client = builder(&server).build().unwrap();
        assert_eq!(client.execute(Ping).await.unwrap(), json!({"ok": true}));
    }

    #[tokio::test]
    async fn the_rate_limit_is_known_after_a_response() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(limited(SECOND_30, "499"))
            .mount(&server)
            .await;

        let client = builder(&server).build().unwrap();
        assert!(client.rate_limit().is_none());
        client.execute(Ping).await.unwrap();
        let observed = client.rate_limit().unwrap();
        assert_eq!((observed.limit(), observed.remaining()), (500, 499));
        assert!(observed.resets_in() <= Duration::from_secs(30));
        assert_eq!(client.clone().rate_limit(), Some(observed));
    }

    #[tokio::test]
    async fn redirects_are_not_followed() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", "/elsewhere"))
            .mount(&server)
            .await;
        Mock::given(path("/elsewhere"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;

        let error = builder(&server)
            .build()
            .unwrap()
            .execute(Ping)
            .await
            .unwrap_err();
        assert_eq!(error.as_api().map(|api| api.status), Some(302));
    }

    #[tokio::test]
    async fn error_responses_become_api_errors() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(ResponseTemplate::new(404).set_body_json(
                json!({"error": true, "error_code": 20, "error_message": "Item not found"}),
            ))
            .mount(&server)
            .await;

        let error = builder(&server)
            .build()
            .unwrap()
            .execute(Ping)
            .await
            .unwrap_err();
        assert!(error.is_not_found());
        assert_eq!(
            error.to_string(),
            "picqer api error (status 404, code 20): Item not found"
        );
    }

    #[tokio::test]
    async fn refused_credentials_come_back_as_html() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(
                ResponseTemplate::new(401)
                    .insert_header("content-type", "text/html; charset=utf-8")
                    .set_body_string("<html>Unauthorized</html>"),
            )
            .mount(&server)
            .await;

        let error = builder(&server)
            .build()
            .unwrap()
            .execute(Ping)
            .await
            .unwrap_err();
        assert!(error.is_unauthorized());
    }

    #[tokio::test]
    async fn the_rate_limit_is_an_error_by_default() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(too_many(SECOND_30))
            .expect(1)
            .mount(&server)
            .await;

        let error = builder(&server)
            .build()
            .unwrap()
            .execute(Ping)
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            Error::RateLimited { resets_in } if resets_in == Duration::from_secs(30)
        ));
    }

    #[tokio::test]
    async fn a_request_is_sent_again_after_the_minute() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(too_many(SECOND_59))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(path("/ping"))
            .respond_with(limited(SECOND_59, "10"))
            .expect(1)
            .mount(&server)
            .await;

        let client = builder(&server)
            .retry_on_rate_limit(NonZeroU32::new(1).unwrap())
            .build()
            .unwrap();
        let started = Instant::now();
        assert_eq!(client.execute(Ping).await.unwrap(), json!({"ok": true}));
        assert!(started.elapsed() >= Duration::from_millis(900));
    }

    #[tokio::test]
    async fn retries_run_out() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(too_many(SECOND_59))
            .expect(2)
            .mount(&server)
            .await;

        let client = builder(&server)
            .retry_on_rate_limit(NonZeroU32::new(1).unwrap())
            .build()
            .unwrap();
        let error = client.execute(Ping).await.unwrap_err();
        assert!(matches!(error, Error::RateLimited { .. }));
    }

    #[tokio::test]
    async fn throttling_waits_when_nothing_is_left() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(limited(SECOND_59, "0"))
            .mount(&server)
            .await;

        let client = builder(&server).throttle().build().unwrap();
        client.execute(Ping).await.unwrap();
        let started = Instant::now();
        client.execute(Ping).await.unwrap();
        assert!(started.elapsed() >= Duration::from_millis(900));
    }

    #[tokio::test]
    async fn without_throttling_requests_go_straight_out() {
        let server = MockServer::start().await;
        Mock::given(path("/ping"))
            .respond_with(limited(SECOND_59, "0"))
            .mount(&server)
            .await;

        let client = builder(&server).build().unwrap();
        client.execute(Ping).await.unwrap();
        let started = Instant::now();
        client.execute(Ping).await.unwrap();
        assert!(started.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn an_unsafe_base_url_is_refused() {
        let server_url = Url::parse("http://example.com/").unwrap();
        let result = Client::builder(
            Subdomain::new("example").unwrap(),
            ApiKey::new("test-key").unwrap(),
            UserAgent::new("Tests", "dev@example.com").unwrap(),
        )
        .base_url(server_url)
        .build();
        assert!(matches!(result, Err(Error::Validation(_))));
    }
}
