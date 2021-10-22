//! Wrapper around [`reqwest`] for multi-page requests.
//!
//! Only relevant/tested on the OpenMensa API.

use chrono::Duration;
use itertools::Itertools;
use serde::de::DeserializeOwned;

use std::marker::PhantomData;

use crate::{
    cache,
    error::{Error, Result},
};

/// An iterator over json pages containing lists.
///
/// # Example
///
/// ## Page 1
///
/// ```json
/// [ { "id": 1 },
///   { "id": 2 } ]
/// ```
///
/// ## Page 2
///
/// ```json
/// [ { "id": 3 },
///   { "id": 4 } ]
/// ```
pub struct PaginatedList<T>
where
    T: DeserializeOwned,
{
    next_page: Option<String>,
    ttl: Duration,
    __item: PhantomData<T>,
}

impl<T> PaginatedList<T>
where
    T: DeserializeOwned,
{
    /// Create a new page iterator
    ///
    /// Takes the `url` for the first page and a
    /// `local_ttl` for the cached values.
    pub fn new<S: AsRef<str>>(url: S, ttl: Duration) -> Self {
        PaginatedList {
            ttl,
            next_page: Some(url.as_ref().into()),
            __item: PhantomData,
        }
    }
}

impl<T> PaginatedList<T>
where
    T: DeserializeOwned,
{
    /// Consumes this iterator, flattening the collected pages.
    pub fn consume(self) -> Result<Vec<T>> {
        self.flatten_ok().try_collect()
    }
}

impl<T> Iterator for PaginatedList<T>
where
    T: DeserializeOwned,
{
    type Item = Result<Vec<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        // This will yield until no next_page is available
        let curr_page = self.next_page.take()?;
        let res = cache::fetch(curr_page, self.ttl, |text, headers| {
            let val = serde_json::from_str::<Vec<_>>(&text)
                .map_err(|why| Error::Deserializing(why, "fetching json in pagination iterator"))?;
            Ok((val, headers.this_page, headers.next_page, headers.last_page))
        });
        match res {
            Ok((val, this_page, next_page, last_page)) => {
                // Only update next_page, if we're not on the last page!
                // This should be safe for all cases
                if this_page.unwrap_or_default() < last_page.unwrap_or_default() {
                    // OpenMensa returns empty lists for large pages
                    // this is just to keep me sane
                    if !val.is_empty() {
                        self.next_page = next_page;
                    }
                }
                Some(Ok(val))
            }
            Err(why) => {
                // Implicitly does not set the next_page url, so
                // this iterator is done now
                Some(Err(why))
            }
        }
    }
}
