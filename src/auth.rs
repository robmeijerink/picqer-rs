use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::header::HeaderValue;
use zeroize::Zeroizing;

use crate::error::ValidationError;

const ENV_KEY: &str = "PICQER_API_KEY";

/// A Picqer API key.
///
/// This copy of the key is wiped from memory on drop, never shown by `Debug` and sent as a
/// sensitive header. The copy inside a request header, held by the HTTP client while the request
/// runs, is not wiped.
#[derive(Clone)]
pub struct ApiKey(Zeroizing<String>);

impl ApiKey {
    /// Wraps an API key, rejecting empty keys and keys with whitespace, control characters or a
    /// colon, which cannot be part of the user name of basic authentication.
    pub fn new(key: impl Into<String>) -> Result<Self, ValidationError> {
        let key = Zeroizing::new(key.into());
        if key.is_empty() {
            return Err(ValidationError::new("api key", "must not be empty"));
        }
        if key
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == ':')
        {
            return Err(ValidationError::new(
                "api key",
                "must not contain whitespace, control characters or `:`",
            ));
        }
        Ok(Self(key))
    }

    /// Reads the key from `PICQER_API_KEY`.
    pub fn from_env() -> Result<Self, ValidationError> {
        match std::env::var(ENV_KEY) {
            Ok(value) if !value.is_empty() => Self::new(value),
            _ => Err(ValidationError::new(
                "api key",
                format!("{ENV_KEY} is not set"),
            )),
        }
    }

    pub(crate) fn authorization(&self) -> HeaderValue {
        let credentials = Zeroizing::new(format!("{}:", self.0.as_str()));
        let encoded = Zeroizing::new(STANDARD.encode(credentials.as_bytes()));
        let mut value = HeaderValue::from_str(&format!("Basic {}", encoded.as_str()))
            .expect("base64 output is a valid header value");
        value.set_sensitive(true);
        value
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_malformed_keys() {
        assert!(ApiKey::new("").is_err());
        assert!(ApiKey::new("a b").is_err());
        assert!(ApiKey::new("a\nb").is_err());
        assert!(ApiKey::new("a:b").is_err());
    }

    #[test]
    fn debug_hides_the_key() {
        let key = ApiKey::new("secret").unwrap();
        assert_eq!(format!("{key:?}"), "ApiKey([redacted])");
    }

    #[test]
    fn authorization_is_basic_with_an_empty_password_and_sensitive() {
        let value = ApiKey::new("test-key").unwrap().authorization();
        assert_eq!(value.to_str().unwrap(), "Basic dGVzdC1rZXk6");
        assert!(value.is_sensitive());
    }
}
