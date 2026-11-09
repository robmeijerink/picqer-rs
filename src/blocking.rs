//! A blocking client with the same operations as the asynchronous [`Client`](crate::Client).
//!
//! Do not use it from inside an asynchronous runtime: it panics there, as `reqwest::blocking`
//! does.

use std::fmt;
use std::sync::Arc;
use std::thread::sleep;

use crate::auth::ApiKey;
use crate::client::ClientBuilder;
use crate::error::{Error, Result};
use crate::http::{Operation, Response, Transport};
use crate::rate_limit::{Limiter, RateLimit, until_next_minute};
use crate::types::{Subdomain, UserAgent};

/// Blocking client for the Picqer API.
///
/// Cloning is cheap; clones share the connection pool and what is known about the rate limit.
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    http: reqwest::blocking::Client,
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

impl ClientBuilder {
    /// Builds the blocking client. A client passed to [`ClientBuilder::http_client`] is ignored.
    pub fn build_blocking(self) -> Result<Client> {
        let settings = self.into_settings()?;
        let http = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(settings.timeout)
            .connect_timeout(settings.connect_timeout)
            .build()?;
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

impl Client {
    /// Creates a client with default settings.
    pub fn new(subdomain: Subdomain, api_key: ApiKey, user_agent: UserAgent) -> Result<Self> {
        Self::builder(subdomain, api_key, user_agent).build_blocking()
    }

    /// Creates a client with the subdomain from `PICQER_SUBDOMAIN` and the key from
    /// `PICQER_API_KEY`.
    pub fn from_env(user_agent: UserAgent) -> Result<Self> {
        Self::new(Subdomain::from_env()?, ApiKey::from_env()?, user_agent)
    }

    /// Returns a builder for customising the client. Finish it with
    /// [`ClientBuilder::build_blocking`].
    pub fn builder(subdomain: Subdomain, api_key: ApiKey, user_agent: UserAgent) -> ClientBuilder {
        ClientBuilder::new(subdomain, api_key, user_agent)
    }

    /// What the API reported about the rate limit with its last response, if it reported anything.
    pub fn rate_limit(&self) -> Option<RateLimit> {
        self.inner.limiter.current()
    }

    /// Operations on comments.
    pub fn comments(&self) -> Comments<'_> {
        Comments { client: self }
    }

    /// Operations on customers.
    pub fn customers(&self) -> Customers<'_> {
        Customers { client: self }
    }

    /// Operations on order fields.
    pub fn order_fields(&self) -> OrderFields<'_> {
        OrderFields { client: self }
    }

    /// Operations on customer fields.
    pub fn customer_fields(&self) -> CustomerFields<'_> {
        CustomerFields { client: self }
    }

    /// Operations on orders.
    pub fn orders(&self) -> Orders<'_> {
        Orders { client: self }
    }

    /// Operations on picklists.
    pub fn picklists(&self) -> Picklists<'_> {
        Picklists { client: self }
    }

    /// Operations on packagings.
    pub fn packagings(&self) -> Packagings<'_> {
        Packagings { client: self }
    }

    /// Operations on picklist batches.
    pub fn picklist_batches(&self) -> PicklistBatches<'_> {
        PicklistBatches { client: self }
    }

    /// Operations on the presets of picklist batches.
    pub fn picklist_batch_presets(&self) -> PicklistBatchPresets<'_> {
        PicklistBatchPresets { client: self }
    }

    /// Operations on location types.
    pub fn location_types(&self) -> LocationTypes<'_> {
        LocationTypes { client: self }
    }

    /// Operations on locations and containers.
    pub fn locations(&self) -> Locations<'_> {
        Locations { client: self }
    }

    /// Operations on the stock history of locations.
    pub fn location_stock_history(&self) -> LocationStockHistory<'_> {
        LocationStockHistory { client: self }
    }

    /// Operations on picking containers.
    pub fn picking_containers(&self) -> PickingContainers<'_> {
        PickingContainers { client: self }
    }

    /// Operations on hooks, the subscriptions to webhook events.
    pub fn hooks(&self) -> Hooks<'_> {
        Hooks { client: self }
    }

    /// Operations on stats.
    pub fn stats(&self) -> Stats<'_> {
        Stats { client: self }
    }

    /// Operations on the orders of webshop integrations.
    pub fn webshop_orders(&self) -> WebshopOrders<'_> {
        WebshopOrders { client: self }
    }

    /// Operations on fulfilment customers.
    pub fn fulfilment_customers(&self) -> FulfilmentCustomers<'_> {
        FulfilmentCustomers { client: self }
    }

    /// Operations on tasks.
    pub fn tasks(&self) -> Tasks<'_> {
        Tasks { client: self }
    }

    /// Operations on backorders.
    pub fn backorders(&self) -> Backorders<'_> {
        Backorders { client: self }
    }

    /// Operations on returns.
    pub fn returns(&self) -> Returns<'_> {
        Returns { client: self }
    }

    /// Operations on the statuses of returns.
    pub fn return_statuses(&self) -> ReturnStatuses<'_> {
        ReturnStatuses { client: self }
    }

    /// Operations on the reasons for returns.
    pub fn return_reasons(&self) -> ReturnReasons<'_> {
        ReturnReasons { client: self }
    }

    /// Operations on receipts.
    pub fn receipts(&self) -> Receipts<'_> {
        Receipts { client: self }
    }

    /// Operations on shipments.
    pub fn shipments(&self) -> Shipments<'_> {
        Shipments { client: self }
    }

    /// Operations on shipping providers.
    pub fn shipping_providers(&self) -> ShippingProviders<'_> {
        ShippingProviders { client: self }
    }

    /// Operations on templates.
    pub fn templates(&self) -> Templates<'_> {
        Templates { client: self }
    }

    /// Operations on users.
    pub fn users(&self) -> Users<'_> {
        Users { client: self }
    }

    /// Operations on packing stations.
    pub fn packing_stations(&self) -> PackingStations<'_> {
        PackingStations { client: self }
    }

    /// Operations on products.
    pub fn products(&self) -> Products<'_> {
        Products { client: self }
    }

    /// Operations on warehouses.
    pub fn warehouses(&self) -> Warehouses<'_> {
        Warehouses { client: self }
    }

    /// Operations on VAT groups.
    pub fn vat_groups(&self) -> VatGroups<'_> {
        VatGroups { client: self }
    }

    /// Operations on the stock history.
    pub fn stock_history(&self) -> StockHistory<'_> {
        StockHistory { client: self }
    }

    /// Operations on purchase orders.
    pub fn purchase_orders(&self) -> PurchaseOrders<'_> {
        PurchaseOrders { client: self }
    }

    /// Operations on suppliers.
    pub fn suppliers(&self) -> Suppliers<'_> {
        Suppliers { client: self }
    }

    /// Operations on tags.
    pub fn tags(&self) -> Tags<'_> {
        Tags { client: self }
    }

    /// Operations on pricelists.
    pub fn pricelists(&self) -> Pricelists<'_> {
        Pricelists { client: self }
    }

    /// Operations on product fields.
    pub fn product_fields(&self) -> ProductFields<'_> {
        ProductFields { client: self }
    }

    pub(crate) fn execute<O: Operation>(&self, operation: O) -> Result<O::Output> {
        let inner = &self.inner;
        let request = operation.request()?;
        let url = inner.transport.url(&request);
        let headers = inner.transport.headers(&request)?;
        let mut retries = inner.retries;
        loop {
            if inner.throttle
                && let Some(wait) = inner.limiter.throttle_wait()
            {
                sleep(wait);
            }
            let mut builder = inner
                .http
                .request(request.method.clone(), url.clone())
                .headers(headers.clone());
            if let Some(body) = &request.body {
                builder = builder.body(body.clone());
            }
            let response = builder.send()?;
            let status = response.status().as_u16();
            let headers = response.headers().clone();
            let body = response.bytes()?.to_vec();
            inner.limiter.record(&headers);
            if status == 429 {
                let resets_in = until_next_minute(&headers);
                if retries == 0 {
                    return Err(Error::RateLimited { resets_in });
                }
                retries -= 1;
                sleep(resets_in);
                continue;
            }
            return operation.parse(Response { status, body });
        }
    }
}

macro_rules! blocking_service {
    ($(#[$meta:meta])* $name:ident, $methods:path) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name<'a> {
            pub(crate) client: &'a Client,
        }

        impl $name<'_> {
            $methods!(;);
        }
    };
}

blocking_service!(
    /// Operations on comments.
    Comments,
    crate::comments::comments_methods
);
blocking_service!(
    /// Operations on customer fields.
    CustomerFields,
    crate::customer_fields::customer_fields_methods
);
blocking_service!(
    /// Operations on customers.
    Customers,
    crate::customers::customers_methods
);
blocking_service!(
    /// Operations on order fields.
    OrderFields,
    crate::order_fields::order_fields_methods
);
blocking_service!(
    /// Operations on orders.
    Orders,
    crate::orders::orders_methods
);
blocking_service!(
    /// Operations on picklists.
    Picklists,
    crate::picklists::picklists_methods
);
blocking_service!(
    /// Operations on packagings.
    Packagings,
    crate::packagings::packagings_methods
);
blocking_service!(
    /// Operations on picklist batches.
    PicklistBatches,
    crate::picklist_batches::picklist_batches_methods
);
blocking_service!(
    /// Operations on the presets of picklist batches.
    PicklistBatchPresets,
    crate::picklist_batch_presets::picklist_batch_presets_methods
);
blocking_service!(
    /// Operations on location types.
    LocationTypes,
    crate::location_types::location_types_methods
);
blocking_service!(
    /// Operations on locations and containers.
    Locations,
    crate::locations::locations_methods
);
blocking_service!(
    /// Operations on the stock history of locations.
    LocationStockHistory,
    crate::location_stock_history::location_stock_history_methods
);
blocking_service!(
    /// Operations on picking containers.
    PickingContainers,
    crate::picking_containers::picking_containers_methods
);
blocking_service!(
    /// Operations on hooks, the subscriptions to webhook events.
    Hooks,
    crate::webhooks::hooks_methods
);
blocking_service!(
    /// Operations on stats.
    Stats,
    crate::stats::stats_methods
);
blocking_service!(
    /// Operations on the orders of webshop integrations.
    WebshopOrders,
    crate::webshop_orders::webshop_orders_methods
);
blocking_service!(
    /// Operations on fulfilment customers.
    FulfilmentCustomers,
    crate::fulfilment::fulfilment_customers_methods
);
blocking_service!(
    /// Operations on tasks.
    Tasks,
    crate::tasks::tasks_methods
);
blocking_service!(
    /// Operations on backorders.
    Backorders,
    crate::backorders::backorders_methods
);
blocking_service!(
    /// Operations on returns.
    Returns,
    crate::returns::returns_methods
);
blocking_service!(
    /// Operations on the statuses of returns.
    ReturnStatuses,
    crate::returns::return_statuses_methods
);
blocking_service!(
    /// Operations on the reasons for returns.
    ReturnReasons,
    crate::returns::return_reasons_methods
);
blocking_service!(
    /// Operations on receipts.
    Receipts,
    crate::receipts::receipts_methods
);
blocking_service!(
    /// Operations on shipments.
    Shipments,
    crate::shipments::shipments_methods
);
blocking_service!(
    /// Operations on shipping providers.
    ShippingProviders,
    crate::shipping_providers::shipping_providers_methods
);
blocking_service!(
    /// Operations on templates.
    Templates,
    crate::templates::templates_methods
);
blocking_service!(
    /// Operations on users.
    Users,
    crate::users::users_methods
);
blocking_service!(
    /// Operations on packing stations.
    PackingStations,
    crate::packing_stations::packing_stations_methods
);
blocking_service!(
    /// Operations on products.
    Products,
    crate::products::products_methods
);
blocking_service!(
    /// Operations on warehouses.
    Warehouses,
    crate::warehouses::warehouses_methods
);
blocking_service!(
    /// Operations on VAT groups.
    VatGroups,
    crate::vat_groups::vat_groups_methods
);
blocking_service!(
    /// Operations on the stock history.
    StockHistory,
    crate::stock_history::stock_history_methods
);
blocking_service!(
    /// Operations on purchase orders.
    PurchaseOrders,
    crate::purchase_orders::purchase_orders_methods
);
blocking_service!(
    /// Operations on suppliers.
    Suppliers,
    crate::suppliers::suppliers_methods
);
blocking_service!(
    /// Operations on tags.
    Tags,
    crate::tags::tags_methods
);
blocking_service!(
    /// Operations on pricelists.
    Pricelists,
    crate::pricelists::pricelists_methods
);
blocking_service!(
    /// Operations on product fields.
    ProductFields,
    crate::product_fields::product_fields_methods
);

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use std::time::{Duration, Instant};

    use reqwest::Method;
    use serde_json::{Value, json};
    use url::Url;
    use wiremock::matchers::{header, path};
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

    fn client(server: &MockServer) -> ClientBuilder {
        Client::builder(
            Subdomain::new("example").unwrap(),
            ApiKey::new("test-key").unwrap(),
            UserAgent::new("Tests", "dev@example.com").unwrap(),
        )
        .base_url(Url::parse(&server.uri()).unwrap())
    }

    fn serve(mocks: impl std::future::Future<Output = MockServer>) -> MockServer {
        tokio::runtime::Runtime::new().unwrap().block_on(mocks)
    }

    #[test]
    fn the_blocking_client_sends_the_same_request() {
        let server = serve(async {
            let server = MockServer::start().await;
            Mock::given(path("/ping"))
                .and(header("authorization", "Basic dGVzdC1rZXk6"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("date", "Mon, 09 Nov 2026 12:29:30 GMT")
                        .insert_header("x-ratelimit-limit", "500")
                        .insert_header("x-ratelimit-remaining", "498")
                        .set_body_json(json!({"ok": true})),
                )
                .mount(&server)
                .await;
            server
        });

        let client = client(&server).build_blocking().unwrap();
        assert_eq!(client.execute(Ping).unwrap(), json!({"ok": true}));
        assert_eq!(client.rate_limit().unwrap().remaining(), 498);
    }

    #[test]
    fn the_blocking_client_retries_after_the_minute() {
        let server = serve(async {
            let server = MockServer::start().await;
            let date = "Mon, 09 Nov 2026 12:29:59 GMT";
            Mock::given(path("/ping"))
                .respond_with(ResponseTemplate::new(429).insert_header("date", date))
                .up_to_n_times(1)
                .mount(&server)
                .await;
            Mock::given(path("/ping"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
                .mount(&server)
                .await;
            server
        });

        let client = client(&server)
            .retry_on_rate_limit(NonZeroU32::new(1).unwrap())
            .build_blocking()
            .unwrap();
        let started = Instant::now();
        assert_eq!(client.execute(Ping).unwrap(), json!({"ok": true}));
        assert!(started.elapsed() >= Duration::from_millis(900));
    }

    #[test]
    fn the_blocking_client_reports_the_rate_limit_as_an_error() {
        let server = serve(async {
            let server = MockServer::start().await;
            Mock::given(path("/ping"))
                .respond_with(
                    ResponseTemplate::new(429)
                        .insert_header("date", "Mon, 09 Nov 2026 12:29:30 GMT"),
                )
                .mount(&server)
                .await;
            server
        });

        let error = client(&server)
            .build_blocking()
            .unwrap()
            .execute(Ping)
            .unwrap_err();
        assert!(matches!(
            error,
            Error::RateLimited { resets_in } if resets_in == Duration::from_secs(30)
        ));
    }
}
