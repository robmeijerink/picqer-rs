use std::time::Duration;

use reqwest::Method;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use url::Url;

use crate::auth::ApiKey;
use crate::error::{ApiError, Error, Result, ValidationError, truncate};
use crate::types::{Subdomain, UserAgent};

pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub(crate) struct Request {
    pub method: Method,
    pub segments: Vec<String>,
    pub query: Vec<(&'static str, String)>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(method: Method, segments: Vec<String>) -> Self {
        Self {
            method,
            segments,
            query: Vec::new(),
            body: None,
        }
    }

    pub fn query(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.query.push((name, value.into()));
        self
    }

    pub fn json_body<T: serde::Serialize>(mut self, body: &T) -> Result<Self> {
        let bytes = serde_json::to_vec(body).map_err(|source| Error::Decode {
            source,
            body: String::new(),
        })?;
        self.body = Some(bytes);
        Ok(self)
    }
}

pub(crate) fn segments(parts: &[&dyn std::fmt::Display]) -> Vec<String> {
    parts.iter().map(ToString::to_string).collect()
}

pub(crate) struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

impl Response {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn ensure_success(self) -> Result<Self> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(ApiError::from_response(self.status, &self.body).into())
        }
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(|source| Error::Decode {
            source,
            body: truncate(&String::from_utf8_lossy(&self.body)),
        })
    }
}

pub(crate) trait Operation {
    type Output;

    fn request(&self) -> Result<Request>;

    fn parse(&self, response: Response) -> Result<Self::Output>;
}

pub(crate) struct Transport {
    api_key: ApiKey,
    base: Url,
    user_agent: UserAgent,
}

impl Transport {
    pub fn new(api_key: ApiKey, base: Url, user_agent: UserAgent) -> Self {
        Self {
            api_key,
            base,
            user_agent,
        }
    }

    pub fn url(&self, request: &Request) -> Url {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .expect("the base url is validated by the builder")
            .pop_if_empty()
            .extend(&request.segments);
        if !request.query.is_empty() {
            url.query_pairs_mut().extend_pairs(&request.query);
        }
        url
    }

    pub fn headers(&self, request: &Request) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.api_key.authorization());
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(self.user_agent.as_str())
                .map_err(|_| ValidationError::new("user agent", "is not a valid header value"))?,
        );
        if request.body.is_some() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }
        Ok(headers)
    }
}

pub(crate) fn default_base_url(subdomain: &Subdomain) -> Url {
    Url::parse(&format!("https://{subdomain}.picqer.com/api/v1/"))
        .expect("a validated subdomain makes a valid url")
}

pub(crate) fn validate_base_url(url: &Url) -> std::result::Result<(), ValidationError> {
    const FIELD: &str = "base url";
    if url.cannot_be_a_base() {
        return Err(ValidationError::new(
            FIELD,
            "must be an absolute http(s) url",
        ));
    }
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(ValidationError::new(
            FIELD,
            "must not contain credentials, a query or a fragment",
        ));
    }
    match url.scheme() {
        "https" => Ok(()),
        "http" if is_loopback(url) => Ok(()),
        _ => Err(ValidationError::new(
            FIELD,
            "must use https (http is only allowed for loopback hosts)",
        )),
    }
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(url::Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn the_base_url_follows_the_subdomain() {
        let subdomain = Subdomain::new("example").unwrap();
        assert_eq!(
            default_base_url(&subdomain).as_str(),
            "https://example.picqer.com/api/v1/"
        );
    }

    #[test]
    fn base_urls_must_be_https_unless_loopback() {
        assert!(validate_base_url(&url("https://example.picqer.com/api/v1/")).is_ok());
        assert!(validate_base_url(&url("http://127.0.0.1:8080")).is_ok());
        assert!(validate_base_url(&url("http://localhost/")).is_ok());
        assert!(validate_base_url(&url("http://example.picqer.com/")).is_err());
        assert!(validate_base_url(&url("https://user:pw@example.picqer.com/")).is_err());
        assert!(validate_base_url(&url("https://example.picqer.com/?a=b")).is_err());
    }

    #[test]
    fn path_segments_are_encoded_below_the_base_path() {
        let transport = Transport::new(
            ApiKey::new("k").unwrap(),
            url("https://example.test/api/v1/"),
            UserAgent::new("App", "dev@example.com").unwrap(),
        );
        let request = Request::new(Method::GET, vec!["a/b".into(), "7".into()]).query("q", "a b");
        assert_eq!(
            transport.url(&request).as_str(),
            "https://example.test/api/v1/a%2Fb/7?q=a+b"
        );
    }
}
