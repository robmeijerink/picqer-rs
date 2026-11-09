//! The rate limit of the API.
//!
//! Picqer allows 500 requests per minute per API key. The count starts again at every full
//! minute, and every successful response reports what is left.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use reqwest::header::{DATE, HeaderMap};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc2822;

const LIMIT: &str = "x-ratelimit-limit";
const REMAINING: &str = "x-ratelimit-remaining";
const WINDOW: Duration = Duration::from_secs(60);

/// What the API last reported about the rate limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit {
    limit: u32,
    remaining: u32,
    received_at: Instant,
    resets_in: Duration,
}

impl RateLimit {
    /// The number of requests allowed per minute.
    pub const fn limit(&self) -> u32 {
        self.limit
    }

    /// The number of requests left in the current minute when the response was sent.
    pub const fn remaining(&self) -> u32 {
        self.remaining
    }

    /// How long until the count starts again, zero when that moment has passed.
    pub fn resets_in(&self) -> Duration {
        self.resets_in.saturating_sub(self.received_at.elapsed())
    }
}

/// The time left in the minute of the `Date` header, or a whole minute without a usable header.
pub(crate) fn until_next_minute(headers: &HeaderMap) -> Duration {
    headers
        .get(DATE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| OffsetDateTime::parse(value, &Rfc2822).ok())
        .map_or(WINDOW, |date| {
            WINDOW - Duration::from_secs(u64::from(date.second()))
        })
}

fn number(headers: &HeaderMap, name: &str) -> Option<u32> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

#[derive(Debug, Default)]
pub(crate) struct Limiter {
    last: Mutex<Option<RateLimit>>,
}

impl Limiter {
    pub fn record(&self, headers: &HeaderMap) {
        let (Some(limit), Some(remaining)) = (number(headers, LIMIT), number(headers, REMAINING))
        else {
            return;
        };
        let observed = RateLimit {
            limit,
            remaining,
            received_at: Instant::now(),
            resets_in: until_next_minute(headers),
        };
        *self.last.lock().unwrap_or_else(|e| e.into_inner()) = Some(observed);
    }

    pub fn current(&self) -> Option<RateLimit> {
        *self.last.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn throttle_wait(&self) -> Option<Duration> {
        let observed = self.current()?;
        (observed.remaining == 0)
            .then(|| observed.resets_in())
            .filter(|wait| !wait.is_zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    fn headers(date: Option<&str>, remaining: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(date) = date {
            headers.insert(DATE, HeaderValue::from_str(date).unwrap());
        }
        headers.insert(LIMIT, HeaderValue::from_static("500"));
        if let Some(remaining) = remaining {
            headers.insert(REMAINING, HeaderValue::from_str(remaining).unwrap());
        }
        headers
    }

    #[test]
    fn the_wait_is_what_is_left_of_the_minute() {
        let at = |date| until_next_minute(&headers(Some(date), None));
        assert_eq!(at("Mon, 09 Nov 2026 12:29:24 GMT"), Duration::from_secs(36));
        assert_eq!(at("Mon, 09 Nov 2026 12:29:59 GMT"), Duration::from_secs(1));
        assert_eq!(at("Mon, 09 Nov 2026 12:30:00 GMT"), Duration::from_secs(60));
    }

    #[test]
    fn a_missing_or_unreadable_date_means_a_whole_minute() {
        assert_eq!(until_next_minute(&headers(None, None)), WINDOW);
        assert_eq!(until_next_minute(&headers(Some("yesterday"), None)), WINDOW);
    }

    #[test]
    fn the_limiter_remembers_the_last_response() {
        let limiter = Limiter::default();
        assert!(limiter.current().is_none());
        limiter.record(&headers(Some("Mon, 09 Nov 2026 12:29:24 GMT"), Some("498")));
        let observed = limiter.current().unwrap();
        assert_eq!((observed.limit(), observed.remaining()), (500, 498));
        assert!(limiter.throttle_wait().is_none());
    }

    #[test]
    fn responses_without_the_headers_change_nothing() {
        let limiter = Limiter::default();
        limiter.record(&headers(Some("Mon, 09 Nov 2026 12:29:24 GMT"), Some("7")));
        limiter.record(&HeaderMap::new());
        assert_eq!(limiter.current().unwrap().remaining(), 7);
    }

    #[test]
    fn an_empty_count_asks_to_wait() {
        let limiter = Limiter::default();
        limiter.record(&headers(Some("Mon, 09 Nov 2026 12:29:24 GMT"), Some("0")));
        let wait = limiter.throttle_wait().unwrap();
        assert!(wait <= Duration::from_secs(36) && wait > Duration::from_secs(35));
    }
}
