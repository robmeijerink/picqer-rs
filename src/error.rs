use std::fmt;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::reported::Reported;

/// Result type used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

const MAX_BODY_CHARS: usize = 500;

/// Errors returned by the client.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The API answered with a non-success status. It is boxed to keep [`Error`] small.
    #[error(transparent)]
    Api(Box<ApiError>),
    /// The request could not be sent or the response could not be read.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// The response body did not have the expected shape.
    #[error("failed to decode response: {source}")]
    Decode {
        /// Underlying deserialization error.
        source: serde_json::Error,
        /// The (possibly truncated) response body. It may contain personal data, so
        /// think twice before logging it.
        body: String,
    },
    /// A value was rejected before anything was sent.
    #[error(transparent)]
    Validation(#[from] ValidationError),
    /// The rate limit was reached. The request was ignored by the API, so it is safe to send it
    /// again once the minute is over.
    #[error("rate limit reached, it resets in {} seconds", resets_in.as_secs())]
    RateLimited {
        /// How long until the count starts again.
        resets_in: Duration,
    },
    /// The API answered successfully, but not with what was asked for.
    #[error("unexpected response: {0}")]
    UnexpectedResponse(&'static str),
}

impl From<ApiError> for Error {
    fn from(error: ApiError) -> Self {
        Error::Api(Box::new(error))
    }
}

impl Error {
    /// Returns the API error, if this is one.
    pub fn as_api(&self) -> Option<&ApiError> {
        match self {
            Error::Api(error) => Some(error),
            _ => None,
        }
    }

    /// Whether the credentials were refused.
    pub fn is_unauthorized(&self) -> bool {
        self.as_api().is_some_and(|error| error.status == 401)
    }

    /// Whether the requested object does not exist.
    pub fn is_not_found(&self) -> bool {
        self.as_api().is_some_and(|error| error.status == 404)
    }
}

/// A value that was rejected because it does not satisfy the API's constraints.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid {field}: {message}")]
#[non_exhaustive]
pub struct ValidationError {
    /// What was being validated.
    pub field: &'static str,
    /// Why the value was rejected.
    pub message: String,
}

impl ValidationError {
    pub(crate) fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

int_enum! {
    /// The error code of an error response. Every code is unique, so it can be used to show your
    /// own message.
    pub enum ErrorCode(u32) {
        /// Unknown application error.
        UnknownApplicationError = 1,
        /// Missing parameters with `POST` or `PUT`.
        MissingParameters = 5,
        /// Company domain already exists.
        CompanyDomainExists = 6,
        /// Listed in the documentation as "Resource ID found".
        ResourceIdFound = 7,
        /// API function not yet implemented.
        NotImplemented = 8,
        /// Invalid country code.
        InvalidCountryCode = 9,
        /// Invalid address.
        InvalidAddress = 10,
        /// Invalid customer to add to an order.
        InvalidOrderCustomer = 12,
        /// Order is missing products.
        OrderWithoutProducts = 13,
        /// Unknown VAT group.
        UnknownVatGroup = 14,
        /// No product given when picking a product.
        NoProductToPick = 15,
        /// Not a valid picking amount.
        InvalidPickAmount = 16,
        /// Product to pick not found.
        ProductToPickNotFound = 17,
        /// Amount to pick is too high.
        PickAmountTooHigh = 18,
        /// Domain not allowed, it is reserved.
        ReservedDomain = 19,
        /// Item not found.
        ItemNotFound = 20,
        /// Item not found or does not have the right status.
        ItemNotFoundOrWrongStatus = 21,
        /// No valid JSON.
        InvalidJson = 22,
        /// Product id or amount missing from a product in an order.
        OrderProductIncomplete = 23,
        /// The product you are trying to order does not exist.
        OrderedProductNotFound = 24,
        /// The customer you are making an order for does not exist.
        OrderCustomerNotFound = 25,
        /// Cannot create shipment.
        CannotCreateShipment = 26,
        /// Product not found on purchase order.
        ProductNotOnPurchaseOrder = 27,
        /// Rate limit exceeded.
        RateLimitExceeded = 28,
        /// Cannot create order because the monthly limit is reached.
        MonthlyOrderLimitReached = 29,
        /// Cannot change order, it is being processed.
        OrderBeingProcessed = 30,
        /// Function not available for fulfilment.
        NotAvailableForFulfilment = 31,
        /// Order already closed.
        OrderAlreadyClosed = 32,
        /// Object has the wrong status for this action.
        WrongStatus = 33,
        /// Cannot create invoice.
        CannotCreateInvoice = 34,
        /// Invalid date or datetime format.
        InvalidDateFormat = 35,
        /// This user has no rights to this feature.
        NoRights = 36,
        /// Relation already exists on object.
        RelationExists = 37,
        /// Object type is invalid for this action.
        InvalidObjectType = 38,
        /// Cannot change property due to pre-conditions.
        PropertyLocked = 39,
        /// Invalid credentials, could not log in.
        InvalidCredentials = 40,
        /// Product is not found in open purchase orders.
        ProductNotInOpenPurchaseOrders = 41,
        /// Search query failed, try again later.
        SearchFailed = 42,
        /// Stock cannot be negative.
        NegativeStock = 43,
        /// Invalid location for this action.
        InvalidLocation = 44,
        /// IP of request is not allowed.
        IpNotAllowed = 45,
        /// Bad request.
        BadRequest = 46,
        /// Data in request is invalid or of the wrong type.
        InvalidData = 47,
        /// Specific request is blocked for performance reasons.
        BlockedForPerformance = 48,
    }
}

/// An error response from the Picqer API.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ApiError {
    /// HTTP status code.
    pub status: u16,
    /// The error code, when the response carries one.
    pub code: Option<Reported<ErrorCode>>,
    /// The message of the API, or the start of the body when that is not JSON. The API writes it
    /// in the language of the account, so it is meant to be read, not matched.
    pub message: String,
    /// The `errors` object some validation errors carry, as the API sent it.
    pub errors: Option<Value>,
    /// The `context` object some errors carry, as the API sent it. Receiving returned products
    /// puts the errors of each line in `context.errors`, keyed by the line.
    pub context: Option<Value>,
}

#[derive(Deserialize)]
struct Body {
    error_code: Option<Reported<ErrorCode>>,
    error_message: Option<String>,
    errors: Option<Value>,
    context: Option<Value>,
}

impl ApiError {
    pub(crate) fn from_response(status: u16, body: &[u8]) -> Self {
        match serde_json::from_slice::<Body>(body) {
            Ok(parsed) => Self {
                status,
                code: parsed.error_code,
                message: parsed.error_message.unwrap_or_default(),
                errors: parsed.errors,
                context: parsed.context,
            },
            Err(_) => Self {
                status,
                code: None,
                message: truncate(&String::from_utf8_lossy(body)),
                errors: None,
                context: None,
            },
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "picqer api error (status {}", self.status)?;
        match &self.code {
            Some(Reported::Known(code)) => write!(f, ", code {}", code.value())?,
            Some(Reported::Unknown(raw)) => write!(f, ", code {raw}")?,
            None => {}
        }
        write!(f, "): {}", self.message)?;
        match (&self.errors, self.message.is_empty()) {
            (Some(errors), true) => write!(f, "{errors}"),
            _ => Ok(()),
        }
    }
}

impl std::error::Error for ApiError {}

pub(crate) fn truncate(text: &str) -> String {
    text.chars().take(MAX_BODY_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_documented_body() {
        let body = br#"{"error":true,"error_code":20,"error_message":"Item not found"}"#;
        let error = ApiError::from_response(404, body);
        assert_eq!(error.code, Some(Reported::Known(ErrorCode::ItemNotFound)));
        assert_eq!(error.message, "Item not found");
        assert_eq!(
            error.to_string(),
            "picqer api error (status 404, code 20): Item not found"
        );
    }

    #[test]
    fn keeps_codes_it_does_not_know() {
        let body = br#"{"error":true,"error_code":99,"error_message":"New"}"#;
        let error = ApiError::from_response(400, body);
        assert_eq!(error.code, Some(Reported::Unknown(Value::from(99))));
        assert_eq!(
            error.to_string(),
            "picqer api error (status 400, code 99): New"
        );
    }

    #[test]
    fn keeps_the_errors_object_as_sent() {
        let body = br#"{"error":true,"error_code":47,"error_message":"x","errors":{"a":["b"]}}"#;
        let error = ApiError::from_response(422, body);
        assert_eq!(error.errors, Some(serde_json::json!({"a": ["b"]})));
    }

    #[test]
    fn keeps_the_context_as_sent() {
        let body = br#"{"error":true,"error_code":46,"error_message":"Check the lot.","context":{"errors":{"7":{"lot_number":["required"]}}}}"#;
        let error = ApiError::from_response(422, body);
        assert_eq!(
            error.context,
            Some(serde_json::json!({"errors": {"7": {"lot_number": ["required"]}}}))
        );
    }

    #[test]
    fn a_validation_answer_without_a_message_shows_its_errors() {
        let body = br#"{"errors":{"name":["Name is required."]}}"#;
        let error = ApiError::from_response(422, body);
        assert!(error.code.is_none());
        assert_eq!(
            error.to_string(),
            r#"picqer api error (status 422): {"name":["Name is required."]}"#
        );
    }

    #[test]
    fn falls_back_to_the_truncated_raw_body() {
        let html = "<html>".to_owned() + &"x".repeat(2000);
        let error = ApiError::from_response(401, html.as_bytes());
        assert!(error.code.is_none());
        assert_eq!(error.message.chars().count(), MAX_BODY_CHARS);
    }

    #[test]
    fn every_documented_code_round_trips() {
        for code in ErrorCode::ALL {
            assert_eq!(ErrorCode::from_value(code.value()), Some(*code));
        }
        assert_eq!(ErrorCode::ALL.len(), 44);
    }
}
