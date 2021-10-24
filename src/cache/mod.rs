//! Caching build around [`cacache`] and [`reqwest`].
//!
//! ```text
//!            No
//!   Cached? -----------------------------------+
//!    |                                         |
//!    | Yes                                     |
//!    v          No                 No          v       FAIL
//!   TTL valid? -----> ETAG valid? ----------> Get it! --------------+
//!    |                 |               ^       |                    |
//!    | Yes             | Yes           |       |                    |
//!    v                 v               |       v                    |
//!   Load cached! <--- Update cache     |      Update cache!*        |
//!    |    |           metadata!*       |       |                    |
//!    |    |FAIL                        |       |                    |
//!    |    |                            |       |                    |
//!    |    +----------------------------+       |                    |
//!    |                                         |                    |
//!    |                                         |                    |
//!    |<----------------------------------------+                    |
//!    |                                                              |
//!    v                                                              v
//!    OK                                                           FAIL
//!
//! * Ignores all errors
//! ```
//!
//! - `fetch` functions are generalized over web requests and cache loading.
//! - `get` functions only operate on requests.
//! - `load`, `update` functions only operate on the cache.
use cacache::Metadata;
use chrono::{Duration, TimeZone};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{blocking::Response, StatusCode, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{info, warn};

mod fetchable;
#[cfg(test)]
mod tests;
mod wrapper;

pub use fetchable::Fetchable;
pub use wrapper::clear_cache as clear;

use crate::{
    config::CONF,
    error::{Error, Result, ResultExt},
};

/// Returned by most functions in this module.
type TextAndHeaders = (String, Headers);

lazy_static! {
    /// Regex to find the next page in a link header
    /// Probably only applicable to the current version of the openmensa API.
    // TODO: Improve this. How do these LINK headers look in general?
    static ref LINK_NEXT_PAGE_RE: Regex = Regex::new(r#"<([^>]*)>; rel="next""#).unwrap();
}

/// Assortment of headers relevant to the program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Headers {
    pub etag: Option<String>,
    pub this_page: Option<usize>,
    pub next_page: Option<String>,
    pub last_page: Option<usize>,
}

/// Possible results from a cache load.
#[derive(Debug, PartialEq)]
enum CacheResult<T> {
    /// Missed, no entry exists.
    Miss,
    /// Entry exists, but exceeded it's local TTL.
    Stale(Headers, Metadata),
    /// Entry exists and is fresh.
    Hit(T),
}

/// Wrapper around [`fetch`] for responses that contain json.
pub fn fetch_json<S, T>(url: S, local_ttl: Duration) -> Result<T>
where
    S: AsRef<str>,
    T: DeserializeOwned,
{
    fetch(url, local_ttl, |text, _| {
        // TODO: Check content header?
        serde_json::from_str(&text).map_err(|why| Error::Deserializing(why, "fetching json"))
    })
}

/// Generic method for fetching remote url-based resources that may be cached.
pub fn fetch<Map, S, T>(url: S, local_ttl: Duration, map: Map) -> Result<T>
where
    S: AsRef<str>,
    Map: FnOnce(String, Headers) -> Result<T>,
{
    // Normalize the url at this point since we're using it
    // as the cache key
    let url = Url::parse(url.as_ref()).map_err(|_| Error::InternalUrlError)?;
    let url = url.as_ref();
    info!("Fetching {:?}", url);
    // Try getting the value from cache, if that fails, query the web
    let (text, headers) = match try_load_cache(url, local_ttl) {
        Ok(CacheResult::Hit(text_and_headers)) => {
            info!("Hit cache on {:?}", url);
            text_and_headers
        }
        Ok(CacheResult::Miss) => {
            info!("Missed cache on {:?}", url);
            get_and_update_cache(url, None, None)?
        }
        Ok(CacheResult::Stale(old_headers, meta)) => {
            info!("Stale cache on {:?}", url);
            // The cache is stale but may still be valid
            // Request the resource with set IF_NONE_MATCH tag and update
            // the caches metadata or value
            match get_and_update_cache(url, old_headers.etag, Some(meta)) {
                Ok(tah) => tah,
                Err(why) => {
                    warn!("{}", why);
                    // Fetching and updating failed for some reason, retry
                    // without the IF_NONE_MATCH tag and fail if unsuccessful
                    get_and_update_cache(url, None, None)?
                }
            }
        }
        Err(why) => {
            // Fetching from the cache failed for some reason, just
            // request the resource and update the cache
            warn!("{}", why);
            get_and_update_cache(url, None, None)?
        }
    };
    // Apply the map and return the result
    map(text, headers)
}

/// Try loading the cache content.
///
/// This can fail due to errors, but also exits with a [`CacheResult`].
fn try_load_cache(url: &str, local_ttl: Duration) -> Result<CacheResult<TextAndHeaders>> {
    // Try reading the cache's metadata
    match wrapper::read_cache_meta(url)? {
        Some(meta) => {
            // Metadata exists
            if is_fresh(&meta, &local_ttl) {
                // Fresh, try to fetch from cache
                let raw = wrapper::read_cache(&meta)?;
                to_text_and_headers(raw, &meta.metadata).map(CacheResult::Hit)
            } else {
                // Local check failed, but the value may still be valid
                let old_headers = headers_from_metadata(&meta)?;
                Ok(CacheResult::Stale(old_headers, meta))
            }
        }
        None => {
            // No metadata exists, assuming no value exists either
            Ok(CacheResult::Miss)
        }
    }
}

/// Request the resource and update the cache.
///
/// This should only be called if the cache load already failed.
///
/// If an optional `etag` is provided, add the If-None-Match header, and thus
/// only get an update if the new ETAG differs from the given `etag`.
fn get_and_update_cache(
    url: &str,
    etag: Option<String>,
    meta: Option<Metadata>,
) -> Result<TextAndHeaders> {
    // Construct the request
    let mut builder = CONF.client.get(url);
    // Add If-None-Match header, if etag is present
    if let Some(etag) = etag {
        let etag_key = reqwest::header::IF_NONE_MATCH;
        builder = builder.header(etag_key, etag);
    }
    let resp = wrapper::send_request(builder)?;
    let status = resp.status();
    info!("Request to {:?} returned {}", url, status);
    match meta {
        Some(meta) if status == StatusCode::NOT_MODIFIED => {
            // If we received code 304 NOT MODIFIED (after adding the If-None-Match)
            // our cache is actually fresh and it's timestamp should be updated
            let headers = resp.headers().clone().into();
            // Just verified, that meta can be unwrapped!
            touch_and_load_cache(url, &meta, headers)
        }
        _ if status.is_success() => {
            // Request returned successfully, now update the cache with that
            update_cache_from_response(resp)
        }
        _ => {
            // Some error occured, just error out
            // TODO: Retrying would be an option
            Err(Error::NonSuccessStatusCode(url.to_string(), resp.status()))
        }
    }
}

/// Extract body and headers from response and update the cache.
///
/// Only relevant headers will be kept.
fn update_cache_from_response(resp: Response) -> Result<TextAndHeaders> {
    let headers: Headers = resp.headers().clone().into();
    let url = resp.url().as_str().to_owned();
    let text = resp.text().map_err(Error::Reqwest)?;
    wrapper::write_cache(&headers, &url, &text)?;
    Ok((text, headers))
}

/// Reset the cache's TTL, load and return it.
fn touch_and_load_cache(url: &str, meta: &Metadata, headers: Headers) -> Result<TextAndHeaders> {
    let raw = wrapper::read_cache(meta)?;
    let (text, _) = to_text_and_headers(raw, &meta.metadata)?;
    // TODO: Update the timestamp in a smarter way..
    // Do not fall on errors, this doesn’t matter
    wrapper::write_cache(&headers, url, &text).log_warn();
    Ok((text, headers))
}

/// Deserialize the metadata into [`Headers`].
fn headers_from_metadata(meta: &Metadata) -> Result<Headers> {
    serde_json::from_value(meta.metadata.clone())
        .map_err(|why| Error::Deserializing(why, "loading headers from cache"))
}

/// Compares metadata age and local TTL.
fn is_fresh(meta: &Metadata, local_ttl: &Duration) -> bool {
    let now = chrono::Utc::now();
    let age_ms = meta.time;
    let cache_age = chrono::Utc.timestamp((age_ms / 1000) as i64, (age_ms % 1000) as u32);
    now - cache_age < *local_ttl
}

/// Helper to convert raw text and serialized json to [`TextAndHeaders`].
fn to_text_and_headers(raw: Vec<u8>, meta: &serde_json::Value) -> Result<TextAndHeaders> {
    let utf8 = String::from_utf8(raw).map_err(Error::DecodingUtf8)?;
    let headers: Headers = serde_json::from_value(meta.clone()).map_err(|why| {
        Error::Deserializing(why, "reading headers from cache. Try clearing the cache.")
    })?;
    Ok((utf8, headers))
}

impl From<reqwest::header::HeaderMap> for Headers {
    fn from(map: reqwest::header::HeaderMap) -> Self {
        use reqwest::header::*;
        let etag = map
            .get(ETAG)
            .map(|raw| {
                let utf8 = raw.to_str().ok()?;
                Some(utf8.to_string())
            })
            .flatten();
        let this_page = map
            .get("x-current-page")
            .map(|raw| {
                let utf8 = raw.to_str().ok()?;
                utf8.parse().ok()
            })
            .flatten();
        let next_page = map
            .get(LINK)
            .map(|raw| {
                let utf8 = raw.to_str().ok()?;
                let captures = LINK_NEXT_PAGE_RE.captures(utf8)?;
                Some(captures[1].to_owned())
            })
            .flatten();
        let last_page = map
            .get("x-total-pages")
            .map(|raw| {
                let utf8 = raw.to_str().ok()?;
                utf8.parse().ok()
            })
            .flatten();
        Self {
            etag,
            this_page,
            last_page,
            next_page,
        }
    }
}
