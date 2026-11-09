//! Country codes.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::ValidationError;

const ALL: &[&str] = &[
    "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX", "AZ",
    "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ", "BR", "BS",
    "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CN",
    "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM", "DO", "DZ", "EC", "EE",
    "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR", "GA", "GB", "GD", "GE", "GF",
    "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK", "HM",
    "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IQ", "IR", "IS", "IT", "JE", "JM",
    "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN", "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC",
    "LI", "LK", "LR", "LS", "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK",
    "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA",
    "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG",
    "PH", "PK", "PL", "PM", "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW",
    "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS",
    "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO",
    "TR", "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
    "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
];

/// An ISO 3166-1 alpha-2 country code.
///
/// Picqer documents the country of an address and of origin as an ISO 3166 code but does not check
/// it, so a mistake such as `UK` for `GB` would be stored as it is. Only the 249 ISO codes are
/// accepted here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CountryCode(&'static str);

impl CountryCode {
    /// The Netherlands.
    pub const NL: Self = Self("NL");
    /// Belgium.
    pub const BE: Self = Self("BE");

    /// Parses an upper-case ISO country code.
    pub fn new(code: &str) -> Result<Self, ValidationError> {
        ALL.binary_search(&code)
            .map(|index| Self(ALL[index]))
            .map_err(|_| {
                ValidationError::new(
                    "country code",
                    format!("`{code}` is not a supported country code"),
                )
            })
    }

    /// Every accepted country code.
    pub fn all() -> impl Iterator<Item = Self> {
        ALL.iter().map(|code| Self(code))
    }

    /// The two letter code.
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl FromStr for CountryCode {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Serialize for CountryCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0)
    }
}

impl<'de> Deserialize<'de> for CountryCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(&value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_sorted_for_binary_search() {
        assert!(ALL.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn rejects_codes_that_are_not_iso() {
        for refused in ["nl", "ZZ", "", "AN", "XK", "UK", "NLD"] {
            assert!(CountryCode::new(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn accepts_the_iso_codes() {
        assert_eq!(ALL.len(), 249);
        assert!(CountryCode::new("NL").is_ok());
        assert!(CountryCode::new("GB").is_ok());
    }
}
