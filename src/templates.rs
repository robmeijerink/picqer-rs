//! Templates.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::types::TemplateId;

/// A template for documents such as picklists and purchase orders.
///
/// The documentation only shows examples, so what is not an identifier is optional here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Template {
    /// The template.
    pub idtemplate: TemplateId,
    /// The name.
    pub name: Option<String>,
    /// Style rules added to the documents.
    pub custom_css: Option<String>,
    /// Whether this is the default template.
    pub default: Option<bool>,
}

pub(crate) fn list(page: Page) -> Call<Vec<Template>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"templates"]))))
}

pub(crate) fn get(id: TemplateId) -> Call<Template> {
    Call::new(Request::new(Method::GET, segments(&[&"templates", &id])))
}

macro_rules! templates_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of templates.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::templates::Template>> {
            self.client.execute($crate::templates::list(page)) $(.$await)?
        }

        /// Lists all templates, following the pages.
        pub $($async)? fn list_all(&self) -> $crate::error::Result<Vec<$crate::templates::Template>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::templates::list(page)) $(.$await)?)
        }

        /// Fetches a template.
        pub $($async)? fn get(
            &self,
            id: $crate::types::TemplateId,
        ) -> $crate::error::Result<$crate::templates::Template> {
            self.client.execute($crate::templates::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use templates_methods;

/// Operations on templates.
#[derive(Debug, Clone, Copy)]
pub struct Templates<'a> {
    pub(crate) client: &'a Client,
}

impl Templates<'_> {
    templates_methods!(async; await);
}
