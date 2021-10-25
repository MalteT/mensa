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
use ::cacache::Metadata;
use chrono::{Duration, TimeZone};
use lazy_static::lazy_static;
use reqwest::{StatusCode, Url};
use serde::de::DeserializeOwned;
use tracing::{info, warn};

mod fetchable;
#[cfg(test)]
mod tests;

#[cfg(not(test))]
mod cacache;
#[cfg(not(test))]
use self::cacache::Cacache as DefaultCache;

#[cfg(test)]
mod dummy;
#[cfg(test)]
use self::dummy::DummyCache as DefaultCache;

pub use fetchable::Fetchable;

use crate::{
    error::{Error, Result, ResultExt},
    request::{Api, DefaultApi, Headers, Response},
};

/// Returned by most functions in this module.
type TextAndHeaders = (String, Headers);

lazy_static! {
    pub static ref CACHE: DefaultCache = DefaultCache::init().expect("Initialized cache");
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

/// Cache trait
///
/// Generalized over the default Cacache and a DummyCache used for tests.
pub trait Cache
where
    Self: Sized,
{
    /// Initialize the cache.
    fn init() -> Result<Self>;

    /// Read from the cache.
    fn read(&self, meta: &Metadata) -> Result<String>;

    /// Write to the cache.
    ///
    /// The `url` is used as key and the `text` as value for the entry.
    /// The `headers` are attached as additional metadata.
    fn write(&self, headers: &Headers, url: &str, text: &str) -> Result<()>;

    /// Get the [`Metadata`] for the cache entry.
    fn meta(&self, url: &str) -> Result<Option<Metadata>>;

    /// Clear all entries from the cache.
    fn clear(&self) -> Result<()>;

    /// List all cache entries.
    fn list(&self) -> Result<Vec<Metadata>>;

    /// Wrapper around [`fetch`] for responses that contain json.
    fn fetch_json<S, T>(&self, url: S, local_ttl: Duration) -> Result<T>
    where
        S: AsRef<str>,
        T: DeserializeOwned,
    {
        self.fetch(url, local_ttl, |text, _| {
            // TODO: Check content header?
            serde_json::from_str(&text).map_err(|why| Error::Deserializing(why, "fetching json"))
        })
    }

    /// Generic method for fetching remote url-based resources that may be cached.
    ///
    /// This is the preferred way to access the cache, as the requested value
    /// will be fetched from the inter-webs if the cache misses.
    fn fetch<Map, S, T>(&self, url: S, local_ttl: Duration, map: Map) -> Result<T>
    where
        S: AsRef<str>,
        Map: FnOnce(String, Headers) -> Result<T>,
    {
        // Normalize the url at this point since we're using it
        // as the cache key
        let url = Url::parse(url.as_ref()).map_err(|_| Error::InternalUrl)?;
        let url = url.as_ref();
        info!("Fetching {:?}", url);
        // Try getting the value from cache, if that fails, query the web
        let (text, headers) = match try_load_cache(self, url, local_ttl) {
            Ok(CacheResult::Hit(text_and_headers)) => {
                info!("Hit cache on {:?}", url);
                text_and_headers
            }
            Ok(CacheResult::Miss) => {
                info!("Missed cache on {:?}", url);
                get_and_update_cache(self, url, None, None)?
            }
            Ok(CacheResult::Stale(old_headers, meta)) => {
                info!("Stale cache on {:?}", url);
                // The cache is stale but may still be valid
                // Request the resource with set IF_NONE_MATCH tag and update
                // the caches metadata or value
                match get_and_update_cache(self, url, old_headers.etag, Some(meta)) {
                    Ok(tah) => tah,
                    Err(why) => {
                        warn!("{}", why);
                        // Fetching and updating failed for some reason, retry
                        // without the IF_NONE_MATCH tag and fail if unsuccessful
                        get_and_update_cache(self, url, None, None)?
                    }
                }
            }
            Err(why) => {
                // Fetching from the cache failed for some reason, just
                // request the resource and update the cache
                warn!("{}", why);
                get_and_update_cache(self, url, None, None)?
            }
        };
        // Apply the map and return the result
        map(text, headers)
    }
}

/// Try loading the cache content.
///
/// This can fail due to errors, but also exits with a [`CacheResult`].
fn try_load_cache<C: Cache>(
    cache: &C,
    url: &str,
    local_ttl: Duration,
) -> Result<CacheResult<TextAndHeaders>> {
    // Try reading the cache's metadata
    match cache.meta(url)? {
        Some(meta) => {
            // Metadata exists
            if is_fresh(&meta, &local_ttl) {
                // Fresh, try to fetch from cache
                let text = cache.read(&meta)?;
                to_text_and_headers(text, &meta.metadata).map(CacheResult::Hit)
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
fn get_and_update_cache<C: Cache>(
    cache: &C,
    url: &str,
    etag: Option<String>,
    meta: Option<Metadata>,
) -> Result<TextAndHeaders> {
    lazy_static! {
        static ref API: DefaultApi = DefaultApi::create().expect("Failed to create API");
    }
    // Send request with optional ETag header
    let resp = API.get(url, etag)?;
    info!("Request to {:?} returned {}", url, resp.status);
    match meta {
        Some(meta) if resp.status == StatusCode::NOT_MODIFIED => {
            // If we received code 304 NOT MODIFIED (after adding the If-None-Match)
            // our cache is actually fresh and it's timestamp should be updated
            touch_and_load_cache(cache, url, &meta, resp.headers)
        }
        _ if resp.status.is_success() => {
            // Request returned successfully, now update the cache with that
            update_cache_from_response(cache, resp)
        }
        _ => {
            // Some error occured, just error out
            // TODO: Retrying would be an option
            Err(Error::NonSuccessStatusCode(url.to_string(), resp.status))
        }
    }
}

/// Extract body and headers from response and update the cache.
///
/// Only relevant headers will be kept.
fn update_cache_from_response<C: Cache>(cache: &C, resp: Response) -> Result<TextAndHeaders> {
    let url = resp.url.to_owned();
    cache.write(&resp.headers, &url, &resp.body)?;
    Ok((resp.body, resp.headers))
}

/// Reset the cache's TTL, load and return it.
fn touch_and_load_cache<C: Cache>(
    cache: &C,
    url: &str,
    meta: &Metadata,
    headers: Headers,
) -> Result<TextAndHeaders> {
    let raw = cache.read(meta)?;
    let (text, _) = to_text_and_headers(raw, &meta.metadata)?;
    // TODO: Update the timestamp in a smarter way..
    // Do not fall on errors, this doesn’t matter
    cache.write(&headers, url, &text).log_warn();
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
fn to_text_and_headers(text: String, meta: &serde_json::Value) -> Result<TextAndHeaders> {
    let headers: Headers = serde_json::from_value(meta.clone()).map_err(|why| {
        Error::Deserializing(why, "reading headers from cache. Try clearing the cache.")
    })?;
    Ok((text, headers))
}
