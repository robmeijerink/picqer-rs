use std::collections::BTreeSet;

use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

const API_ROOT: &str = "/api/v1";
const DROPPED_COLUMNS: &[&str] = &["description", "example"];

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Page {
    title: String,
    endpoints: Vec<Endpoint>,
    tables: Vec<Table>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Endpoint {
    method: String,
    path: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Table {
    heading: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Changelog {
    entries: Vec<String>,
}

fn selector(css: &str) -> Selector {
    Selector::parse(css).expect("the selectors in this file are valid")
}

fn text(element: ElementRef<'_>) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn documentation_links(html: &str) -> BTreeSet<String> {
    let document = Html::parse_document(html);
    document
        .select(&selector("a[href]"))
        .filter_map(|link| link.value().attr("href"))
        .filter_map(|href| href.strip_prefix("/en/api/"))
        .map(|rest| rest.split(['#', '?']).next().unwrap_or(""))
        .filter(|slug| !slug.is_empty() && !slug.contains('/'))
        .map(|slug| format!("/en/api/{slug}"))
        .collect()
}

pub fn page(html: &str) -> Page {
    let document = Html::parse_document(html);
    let title = document
        .select(&selector("h1"))
        .next()
        .map(text)
        .unwrap_or_default();

    let endpoints = document
        .select(&selector("div.requesturi"))
        .map(|request| {
            let method = request.select(&selector(".http-method")).next().map(text);
            let uri = request.select(&selector(".uri")).next().map(text);
            let uri = uri.unwrap_or_default();
            let path = uri
                .split_once(API_ROOT)
                .map_or(uri.as_str(), |(_, rest)| rest);
            Endpoint {
                method: method.unwrap_or_default(),
                path: path.to_owned(),
            }
        })
        .collect();

    let mut tables = Vec::new();
    let mut heading = String::new();
    for element in document.select(&selector("h2, h3, table")) {
        if element.value().name() == "table" {
            tables.push(table(&heading, element));
        } else {
            heading = text(element);
        }
    }
    Page {
        title,
        endpoints,
        tables,
    }
}

fn table(heading: &str, element: ElementRef<'_>) -> Table {
    let header: Vec<String> = element
        .select(&selector("th"))
        .map(|cell| text(cell).to_lowercase())
        .collect();
    let keep: Vec<bool> = header
        .iter()
        .map(|name| !DROPPED_COLUMNS.contains(&name.as_str()))
        .collect();
    let columns = header
        .iter()
        .zip(&keep)
        .filter(|(_, keep)| **keep)
        .map(|(name, _)| name.clone())
        .collect();
    let rows = element
        .select(&selector("tr"))
        .map(|row| row.select(&selector("td")).map(text).collect::<Vec<_>>())
        .filter(|cells| !cells.is_empty())
        .map(|cells| {
            cells
                .into_iter()
                .zip(keep.iter().chain(std::iter::repeat(&true)))
                .filter(|(_, keep)| **keep)
                .map(|(cell, _)| cell)
                .collect()
        })
        .collect();
    Table {
        heading: heading.to_owned(),
        columns,
        rows,
    }
}

pub fn changelog(html: &str) -> Changelog {
    let document = Html::parse_document(html);
    let entries = document.select(&selector("h2")).map(text).collect();
    Changelog { entries }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = r#"
        <h1>Tags</h1>
        <h2>Attributes</h2>
        <table>
          <tr><th>Name</th><th>Type</th><th>Required</th><th>Description</th></tr>
          <tr><td>idtag</td><td>integer</td><td>generated</td><td>Unique   reference</td></tr>
        </table>
        <h2>Get all tags</h2>
        <div class="api-example">
          <div class="requesturi">
            <span class="http-method">GET</span>
            <span class="uri">https://example.picqer.com/api/v1/tags</span>
          </div>
        </div>
        <a href="/en/api/products#attributes">products</a>
        <a href="/en/api/tags">tags</a>
        <a href="/en/api/tags/extra">deeper</a>
        <a href="https://elsewhere.test/en/api/x">elsewhere</a>
    "#;

    #[test]
    fn a_page_has_a_title_endpoints_and_tables_without_descriptions() {
        let page = page(PAGE);
        assert_eq!(page.title, "Tags");
        assert_eq!(
            page.endpoints,
            vec![Endpoint {
                method: "GET".into(),
                path: "/tags".into()
            }]
        );
        assert_eq!(page.tables.len(), 1);
        assert_eq!(page.tables[0].heading, "Attributes");
        assert_eq!(page.tables[0].columns, ["name", "type", "required"]);
        assert_eq!(
            page.tables[0].rows,
            vec![vec!["idtag", "integer", "generated"]]
        );
    }

    #[test]
    fn the_host_of_an_example_url_does_not_matter() {
        let html = r#"<div class="requesturi"><span class="http-method">GET</span>
            <span class="uri">https://example.com/api/v1/packagings/{id}</span></div>"#;
        assert_eq!(page(html).endpoints[0].path, "/packagings/{id}");
    }

    #[test]
    fn only_pages_of_the_documentation_are_followed() {
        let links = documentation_links(PAGE);
        assert_eq!(
            links.into_iter().collect::<Vec<_>>(),
            ["/en/api/products", "/en/api/tags"]
        );
    }

    #[test]
    fn the_changelog_keeps_the_dates_of_its_entries() {
        let html =
            "<h1>Changelog</h1><h2><a>5 November 2026</a></h2><p>text</p><h2>1 May 2026</h2>";
        assert_eq!(changelog(html).entries, ["5 November 2026", "1 May 2026"]);
    }
}
