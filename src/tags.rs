//! Tags.

use std::fmt;

use reqwest::Method;
use serde::{Deserialize, Deserializer, Serialize};

use crate::client::Client;
use crate::error::{Result, ValidationError};
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::types::TagId;

bounded_string! {
    /// The title of a tag.
    TagTitle, "tag title", 1, usize::MAX
}

/// A color as `#` followed by six hexadecimal digits, such as `#823882`.
///
/// Picqer refuses other forms, such as `#fff`, and answers with a server error and a corrupt tag
/// for longer ones, so nothing else is accepted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct HexColor(String);

impl HexColor {
    /// Validates a color.
    pub fn new(value: impl Into<String>) -> std::result::Result<Self, ValidationError> {
        let value = value.into();
        let digits = value.strip_prefix('#').unwrap_or("");
        if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ValidationError::new(
                "color",
                "must be `#` followed by six hexadecimal digits",
            ));
        }
        Ok(Self(value))
    }

    /// The color.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HexColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A label that can be put on products, customers and orders.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Tag {
    /// The tag.
    pub idtag: TagId,
    /// The title of the tag.
    pub title: String,
    /// The color of the tag.
    pub color: HexColor,
    /// Whether orders inherit the tags of a customer.
    pub inherit: bool,
    /// The color of the text in the tag, a contrasting color chosen by Picqer.
    #[serde(rename = "textColor")]
    pub text_color: HexColor,
}

/// A tag to create.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewTag {
    title: TagTitle,
    color: HexColor,
    inherit: bool,
}

impl NewTag {
    /// A tag with the fields Picqer requires.
    pub fn new(title: TagTitle, color: HexColor, inherit: bool) -> Self {
        Self {
            title,
            color,
            inherit,
        }
    }
}

/// Changes to a tag. Only what is set is sent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TagChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<TagTitle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<HexColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inherit: Option<bool>,
}

impl TagChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the title.
    pub fn title(mut self, title: TagTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// Changes the color.
    pub fn color(mut self, color: HexColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Changes whether orders inherit the tags of a customer.
    pub fn inherit(mut self, inherit: bool) -> Self {
        self.inherit = Some(inherit);
        self
    }
}

/// Filters for listing tags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagQuery {
    search: Option<String>,
}

impl TagQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Searches through the names of the tags.
    pub fn search(mut self, text: impl Into<String>) -> Self {
        self.search = Some(text.into());
        self
    }
}

pub(crate) fn list(query: &TagQuery, page: Page) -> Call<Vec<Tag>> {
    let mut request = Request::new(Method::GET, segments(&[&"tags"]));
    if let Some(search) = &query.search {
        request = request.query("search", search.as_str());
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: TagId) -> Call<Tag> {
    Call::new(Request::new(Method::GET, segments(&[&"tags", &id])))
}

pub(crate) fn create(tag: &NewTag) -> Result<Call<Tag>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"tags"])).json_body(tag)?,
    ))
}

pub(crate) fn update(id: TagId, changes: &TagChanges) -> Result<Call<Tag>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"tags", &id])).json_body(changes)?,
    ))
}

pub(crate) fn delete(id: TagId) -> NoContent {
    NoContent::new(Request::new(Method::DELETE, segments(&[&"tags", &id])))
}

macro_rules! tags_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of tags.
        pub $($async)? fn list(
            &self,
            query: &$crate::tags::TagQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::tags::Tag>> {
            self.client.execute($crate::tags::list(query, page)) $(.$await)?
        }

        /// Lists all tags that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::tags::TagQuery,
        ) -> $crate::error::Result<Vec<$crate::tags::Tag>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::tags::list(query, page)) $(.$await)?)
        }

        /// Fetches a tag.
        pub $($async)? fn get(&self, id: $crate::types::TagId) -> $crate::error::Result<$crate::tags::Tag> {
            self.client.execute($crate::tags::get(id)) $(.$await)?
        }

        /// Creates a tag.
        pub $($async)? fn create(
            &self,
            tag: &$crate::tags::NewTag,
        ) -> $crate::error::Result<$crate::tags::Tag> {
            self.client.execute($crate::tags::create(tag)?) $(.$await)?
        }

        /// Changes a tag.
        pub $($async)? fn update(
            &self,
            id: $crate::types::TagId,
            changes: &$crate::tags::TagChanges,
        ) -> $crate::error::Result<$crate::tags::Tag> {
            self.client.execute($crate::tags::update(id, changes)?) $(.$await)?
        }

        /// Deletes a tag.
        pub $($async)? fn delete(&self, id: $crate::types::TagId) -> $crate::error::Result<()> {
            self.client.execute($crate::tags::delete(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use tags_methods;

/// Operations on tags.
#[derive(Debug, Clone, Copy)]
pub struct Tags<'a> {
    pub(crate) client: &'a Client,
}

impl Tags<'_> {
    tags_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_are_six_hexadecimal_digits() {
        assert!(HexColor::new("#823882").is_ok());
        assert!(HexColor::new("#ABCDEF").is_ok());
        assert!(HexColor::new("#abcdef").is_ok());
        for refused in [
            "#fff",
            "ffffff",
            "#1234567",
            "#12345g",
            "#",
            "",
            "rgb(1,2,3)",
        ] {
            assert!(HexColor::new(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn a_change_sends_only_what_is_set() {
        let changes = TagChanges::new().inherit(true);
        assert_eq!(
            serde_json::to_string(&changes).unwrap(),
            r#"{"inherit":true}"#
        );
        assert_eq!(serde_json::to_string(&TagChanges::new()).unwrap(), "{}");
    }
}
