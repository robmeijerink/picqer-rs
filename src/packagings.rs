//! Packagings.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::types::{DateTime, PackagingId};

bounded_string! {
    /// The name of a packaging. Picqer does not require it to be unique.
    PackagingName, "packaging name", 1, usize::MAX
}

/// A packaging that picklists are packed in.
///
/// Picqer does not document the unit of the dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Packaging {
    /// The packaging.
    pub idpackaging: PackagingId,
    /// The name.
    pub name: String,
    /// The barcode.
    pub barcode: Option<String>,
    /// The length.
    pub length: Option<i64>,
    /// The width.
    pub width: Option<i64>,
    /// The height.
    pub height: Option<i64>,
    /// Whether Picqer may advise this packaging.
    pub use_in_auto_advice: Option<bool>,
    /// Whether the packaging is in use.
    pub active: bool,
    /// When the packaging was created.
    pub created_at: DateTime,
    /// When the packaging was last changed.
    pub updated_at: DateTime,
}

/// Which packagings to list. Picqer lists the active ones unless asked for the inactive ones.
///
/// It treats the presence of the `inactive` parameter as asking for the inactive ones, whatever its
/// value, so the active ones are asked for by leaving it out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackagingStatus {
    /// The active packagings.
    #[default]
    Active,
    /// The inactive packagings.
    Inactive,
}

/// A packaging to create. Picqer creates it as an active one.
///
/// Picqer refuses a dimension that is negative or a fraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewPackaging {
    name: PackagingName,
    #[serde(skip_serializing_if = "Option::is_none")]
    barcode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_in_auto_advice: Option<bool>,
}

impl NewPackaging {
    /// A packaging with the field Picqer requires.
    pub fn new(name: PackagingName) -> Self {
        Self {
            name,
            barcode: None,
            length: None,
            width: None,
            height: None,
            use_in_auto_advice: None,
        }
    }

    /// Sets the barcode.
    pub fn barcode(mut self, barcode: impl Into<String>) -> Self {
        self.barcode = Some(barcode.into());
        self
    }

    /// Sets the dimensions.
    pub fn dimensions(mut self, length: u32, width: u32, height: u32) -> Self {
        self.length = Some(length);
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Sets whether Picqer may advise this packaging.
    pub fn use_in_auto_advice(mut self, advise: bool) -> Self {
        self.use_in_auto_advice = Some(advise);
        self
    }
}

/// Changes to a packaging. Only what is set is sent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PackagingChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<PackagingName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    barcode: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_in_auto_advice: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
}

impl PackagingChanges {
    /// No changes yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the name.
    pub fn name(mut self, name: PackagingName) -> Self {
        self.name = Some(name);
        self
    }

    /// Changes the barcode.
    pub fn barcode(mut self, barcode: impl Into<String>) -> Self {
        self.barcode = Some(Some(barcode.into()));
        self
    }

    /// Removes the barcode.
    pub fn without_barcode(mut self) -> Self {
        self.barcode = Some(None);
        self
    }

    /// Changes the dimensions.
    pub fn dimensions(mut self, length: u32, width: u32, height: u32) -> Self {
        self.length = Some(length);
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Changes whether Picqer may advise this packaging.
    pub fn use_in_auto_advice(mut self, advise: bool) -> Self {
        self.use_in_auto_advice = Some(advise);
        self
    }

    /// Makes the packaging active or inactive. Picqer cannot delete a packaging.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }
}

pub(crate) fn list(status: PackagingStatus, page: Page) -> Call<Vec<Packaging>> {
    let mut request = Request::new(Method::GET, segments(&[&"packagings"]));
    if status == PackagingStatus::Inactive {
        request = request.query("inactive", "true");
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: PackagingId) -> Call<Packaging> {
    Call::new(Request::new(Method::GET, segments(&[&"packagings", &id])))
}

pub(crate) fn create(packaging: &NewPackaging) -> Result<Call<Packaging>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"packagings"])).json_body(packaging)?,
    ))
}

pub(crate) fn update(id: PackagingId, changes: &PackagingChanges) -> Result<Call<Packaging>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"packagings", &id])).json_body(changes)?,
    ))
}

macro_rules! packagings_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of packagings.
        pub $($async)? fn list(
            &self,
            status: $crate::packagings::PackagingStatus,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::packagings::Packaging>> {
            self.client.execute($crate::packagings::list(status, page)) $(.$await)?
        }

        /// Lists all active or all inactive packagings, following the pages.
        pub $($async)? fn list_all(
            &self,
            status: $crate::packagings::PackagingStatus,
        ) -> $crate::error::Result<Vec<$crate::packagings::Packaging>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::packagings::list(status, page)) $(.$await)?)
        }

        /// Fetches a packaging.
        pub $($async)? fn get(
            &self,
            id: $crate::types::PackagingId,
        ) -> $crate::error::Result<$crate::packagings::Packaging> {
            self.client.execute($crate::packagings::get(id)) $(.$await)?
        }

        /// Creates a packaging.
        pub $($async)? fn create(
            &self,
            packaging: &$crate::packagings::NewPackaging,
        ) -> $crate::error::Result<$crate::packagings::Packaging> {
            self.client.execute($crate::packagings::create(packaging)?) $(.$await)?
        }

        /// Changes a packaging.
        pub $($async)? fn update(
            &self,
            id: $crate::types::PackagingId,
            changes: &$crate::packagings::PackagingChanges,
        ) -> $crate::error::Result<$crate::packagings::Packaging> {
            self.client.execute($crate::packagings::update(id, changes)?) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use packagings_methods;

/// Operations on packagings.
#[derive(Debug, Clone, Copy)]
pub struct Packagings<'a> {
    pub(crate) client: &'a Client,
}

impl Packagings<'_> {
    packagings_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_packaging_sends_what_was_set() {
        let packaging = NewPackaging::new(PackagingName::new("Box").unwrap())
            .dimensions(10, 20, 30)
            .barcode("b-1")
            .use_in_auto_advice(false);
        assert_eq!(
            serde_json::to_value(packaging).unwrap(),
            serde_json::json!({
                "name": "Box", "barcode": "b-1", "length": 10, "width": 20, "height": 30,
                "use_in_auto_advice": false
            })
        );
    }

    #[test]
    fn changes_can_remove_the_barcode_and_deactivate() {
        assert_eq!(
            serde_json::to_string(&PackagingChanges::new()).unwrap(),
            "{}"
        );
        let changes = PackagingChanges::new().without_barcode().active(false);
        assert_eq!(
            serde_json::to_value(changes).unwrap(),
            serde_json::json!({"barcode": null, "active": false})
        );
    }
}
