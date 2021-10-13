use cacache::Metadata;
use chrono::{Duration, TimeZone};
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

use std::{path::PathBuf, str::from_utf8};

use crate::error::{Error, Result};

lazy_static! {
    static ref CACHE: PathBuf = {
        // TODO: Don't unwrap? Fallback?
        dirs::cache_dir().unwrap().join("mensa")
    };
}

pub fn get_json<U, T>(client: &Client, url: U, ttl: Duration) -> Result<T>
where
    U: AsRef<str>,
    T: DeserializeOwned,
{
    let url = url.as_ref();
    let meta = cacache::metadata_sync(&*CACHE, url).map_err(Error::ReadingCacheMetadata)?;
    if let Some(meta) = meta {
        let age = get_cache_age(&meta);
        if age > ttl {
            get_json_and_update_cache(client, url)
        } else if let Ok(cached) = cacache::read_hash_sync(&*CACHE, &meta.integrity) {
            let utf8 = from_utf8(&cached).unwrap();
            serde_json::from_str(utf8).map_err(Error::DeserializingCacheJson)
        } else {
            get_json_and_update_cache(client, url)
        }
    } else {
        get_json_and_update_cache(client, url)
    }
}

fn get_json_and_update_cache<T>(client: &Client, url: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let value = client
        .get(url)
        .send()
        .map_err(Error::Reqwest)?
        .text()
        .map_err(Error::Reqwest)?;
    cacache::write_sync(&*CACHE, url, &value).map_err(Error::WritingToCache)?;
    serde_json::from_str(&value).map_err(Error::DeserializingCacheJson)
}

fn get_cache_age(meta: &Metadata) -> Duration {
    let now = chrono::Utc::now();
    let age_ms = meta.time;
    let cache_age = chrono::Utc.timestamp((age_ms / 1000) as i64, (age_ms % 1000) as u32);
    now - cache_age
}
