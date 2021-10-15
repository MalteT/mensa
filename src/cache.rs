use cacache::Metadata;
use chrono::{Duration, TimeZone};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::{io::Write, path::PathBuf};

use crate::{
    error::{Error, Result},
    DIR,
};

type TextAndHeaders = (String, Headers);

lazy_static! {
    static ref CACHE: PathBuf = DIR.cache_dir().into();
    static ref LINK_NEXT_PAGE_RE: Regex = Regex::new(r#"<(.*?)>; rel="first""#).unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Headers {
    pub etag: Option<String>,
    pub total_pages: Option<usize>,
    pub next_page: Option<String>,
}

/// Fetch url from the cache or web.
///
/// 1) If the url exists in the cache and the age of the entry is less than `min_ttl`, get that.
/// 2) If the url exists in the cache, but is stale,
///    fetch the head of of the url and compare ETAGs:
///    1) If the ETAG is valid, update the cache entry age and return the cache entry.
///    2) Else, get and update the cache.
/// 3) If the entry does not exist, get it and update the cache.
pub fn get<Map, S, U>(client: &Client, url: S, min_ttl: Duration, map: Map) -> Result<U>
where
    S: AsRef<str>,
    Map: FnOnce(String, Headers) -> Result<U>,
{
    let url = url.as_ref();
    let meta = get_metadata(url)?;
    // If metadata exists, use that
    let (text, headers): (String, Headers) = match meta {
        Some(meta) => {
            let age = get_cache_age(&meta);
            if age > min_ttl {
                // Stale, let's compare ETAGs
                let last_headers: Headers = serde_json::from_value(meta.metadata.clone())
                    .map_err(|why| Error::Deserializing(why, "loading headers from cache"))?;
                get_from_cache_or_update_if_stale(client, url, last_headers.etag.as_ref(), &meta)?
            } else {
                // Fresh, only fetch if an error occurs
                match cacache::read_hash_sync(&*CACHE, &meta.integrity) {
                    Ok(raw) => to_text_and_headers(raw, &meta.metadata)?,
                    Err(_) => get_and_update_cache(client, url)?,
                }
            }
        }
        None => get_and_update_cache(client, url)?,
    };
    map(text, headers)
}

fn get_from_cache_or_update_if_stale(
    client: &Client,
    url: &str,
    etag: Option<&String>,
    meta: &Metadata,
) -> Result<TextAndHeaders> {
    let etag_key = reqwest::header::IF_NONE_MATCH;
    // Get with IF_NONE_MATCH to conditionally request an update
    let mut builder = client.get(url);
    if let Some(etag) = etag {
        builder = builder.header(etag_key, etag);
    }
    let resp = builder.send().map_err(Error::Reqwest)?;
    // Status Code 304 signals a fresh cache
    if resp.status() == StatusCode::NOT_MODIFIED {
        get_from_cache_and_update_timestamp(url, meta)
    } else if resp.status().is_success() {
        update_cache(url, resp)
    } else {
        Err(Error::NonSuccessStatusCode(url.to_string(), resp.status()))
    }
}

fn update_cache(url: &str, resp: Response) -> Result<TextAndHeaders> {
    let header: Headers = resp.headers().clone().into();
    let header_serialized = serde_json::to_value(header.clone())
        .map_err(|why| Error::Serializing(why, "writing headers to local cache"))?;
    let text = resp.text().map_err(Error::Reqwest)?;
    let mut writer = cacache::WriteOpts::new()
        .metadata(header_serialized)
        .open_sync(&*CACHE, url)
        .map_err(Error::WritingToCache)?;
    writer
        .write_all(text.as_bytes())
        .map_err(Error::WritingCache)?;
    writer.commit().map_err(Error::WritingToCache)?;
    Ok((text, header))
}

fn get_from_cache_and_update_timestamp(url: &str, meta: &Metadata) -> Result<TextAndHeaders> {
    // TODO: We could still request the content if this fails..
    let raw = cacache::read_hash_sync(&*CACHE, &meta.integrity).map_err(Error::ReadingCache)?;
    // TODO: Update the timestamp in a smarter way..
    // Just rewrite and ignore errors
    cacache::write_sync(&*CACHE, url, &raw).ok();
    // This was written to the cach as utf8
    to_text_and_headers(raw, &meta.metadata)
}

fn to_text_and_headers(raw: Vec<u8>, meta: &serde_json::Value) -> Result<TextAndHeaders> {
    // This was written to the cache as utf8
    let utf8 = String::from_utf8(raw).unwrap();
    // TODO: This could fail in between versions
    let headers: Headers = serde_json::from_value(meta.clone()).unwrap();
    Ok((utf8, headers))
}

fn get_metadata(url: &str) -> Result<Option<Metadata>> {
    cacache::metadata_sync(&*CACHE, url).map_err(Error::ReadingCacheMetadata)
}

pub fn get_json<U, T>(client: &Client, url: U, min_ttl: Duration) -> Result<T>
where
    U: AsRef<str>,
    T: DeserializeOwned,
{
    get(client, url, min_ttl, |text, _| {
        serde_json::from_str(&text).map_err(|why| Error::Deserializing(why, "fetching json"))
    })
}

fn get_and_update_cache(client: &Client, url: &str) -> Result<TextAndHeaders> {
    let resp = client.get(url).send().map_err(Error::Reqwest)?;
    let header: Headers = resp.headers().clone().into();
    let header_serialized = serde_json::to_value(header.clone())
        .map_err(|why| Error::Serializing(why, "writing headers to cache"))?;
    let text = resp.text().map_err(Error::Reqwest)?;
    let mut writer = cacache::WriteOpts::new()
        .metadata(header_serialized)
        .open_sync(&*CACHE, url)
        .map_err(Error::WritingToCache)?;
    writer
        .write_all(text.as_bytes())
        .map_err(Error::WritingCache)?;
    writer.commit().map_err(Error::WritingToCache)?;
    Ok((text, header))
}

fn get_cache_age(meta: &Metadata) -> Duration {
    let now = chrono::Utc::now();
    let age_ms = meta.time;
    let cache_age = chrono::Utc.timestamp((age_ms / 1000) as i64, (age_ms % 1000) as u32);
    now - cache_age
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
        let total_pages = map
            .get("x-total-pages")
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
        Self {
            etag,
            total_pages,
            next_page,
        }
    }
}
