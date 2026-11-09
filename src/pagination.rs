//! Pagination of list endpoints.

use crate::http::Request;

/// The number of results Picqer returns in one page.
pub const PAGE_SIZE: usize = 100;

/// A page of a list endpoint, counted from zero.
///
/// Picqer pages by `offset`, with at most [`PAGE_SIZE`] results per page. Some endpoints, such as
/// the one for products, round an offset down to a multiple of that and others skip exactly that
/// many results, so only whole pages can be asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Page(u32);

impl Page {
    /// The first page.
    pub const FIRST: Self = Self(0);

    /// The page with the given index, where zero is the first page.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// The index of the page.
    pub const fn index(self) -> u32 {
        self.0
    }

    /// The offset of the first result of the page.
    pub const fn offset(self) -> u64 {
        self.0 as u64 * PAGE_SIZE as u64
    }

    pub(crate) fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub(crate) fn apply(self, request: Request) -> Request {
        match self.offset() {
            0 => request,
            offset => request.query("offset", offset.to_string()),
        }
    }
}

macro_rules! collect_pages {
    ($page:ident => $fetch:expr) => {{
        let mut all = ::std::vec::Vec::new();
        let mut $page = $crate::pagination::Page::FIRST;
        loop {
            let items = $fetch?;
            let full = items.len() >= $crate::pagination::PAGE_SIZE;
            all.extend(items);
            if !full {
                break;
            }
            $page = $page.next();
        }
        Ok(all)
    }};
}

pub(crate) use collect_pages;

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    #[test]
    fn pages_are_whole_multiples_of_the_page_size() {
        assert_eq!(Page::FIRST.offset(), 0);
        assert_eq!(Page::new(1).offset(), 100);
        assert_eq!(Page::new(u32::MAX).offset(), 429_496_729_500);
    }

    #[test]
    fn the_first_page_sends_no_offset() {
        let request = Request::new(Method::GET, vec!["products".into()]);
        assert!(Page::FIRST.apply(request.clone()).query.is_empty());
        assert_eq!(
            Page::new(2).apply(request).query,
            vec![("offset", "200".to_owned())]
        );
    }
}
