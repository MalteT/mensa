//! Wrapper for [`cacache`] and [`reqwest`] methods
//!
//! To make testing easier.
use cacache::Metadata;
use lazy_static::lazy_static;
use reqwest::blocking::{RequestBuilder, Response};
use tracing::info;

use std::{
    io::Write,
    path::Path,
};

use super::Headers;

use crate::error::{Error, Result};

pub fn write_cache(headers: &Headers, url: &str, text: &str) -> Result<()> {
    let header_serialized = serde_json::to_value(headers.clone())
        .map_err(|why| Error::Serializing(why, "writing headers to cache"))?;
    let mut writer = cacache::WriteOpts::new()
        .metadata(header_serialized)
        .open_sync(cache(), url)
        .map_err(|why| Error::Cache(why, "opening for write"))?;
    writer
        .write_all(text.as_bytes())
        .map_err(|why| Error::Io(why, "writing value"))?;
    writer
        .commit()
        .map_err(|why| Error::Cache(why, "commiting write"))?;
    info!("Updated cache for {:?}", url);
    Ok(())
}

pub fn read_cache(meta: &Metadata) -> Result<Vec<u8>> {
    cacache::read_hash_sync(cache(), &meta.integrity)
        .map_err(|why| Error::Cache(why, "reading value"))
}

pub fn read_cache_meta(url: &str) -> Result<Option<Metadata>> {
    cacache::metadata_sync(cache(), url).map_err(|why| Error::Cache(why, "reading metadata"))
}

pub fn clear_cache() -> Result<()> {
    cacache::clear_sync(cache()).map_err(|why| Error::Cache(why, "clearing"))
}

pub fn send_request(builder: RequestBuilder) -> Result<Response> {
    builder.send().map_err(Error::Reqwest)
}

#[cfg(test)]
pub fn list_cache() -> impl Iterator<Item = cacache::Result<Metadata>> {
    cacache::list_sync(cache())
}

#[cfg(not(test))]
fn cache() -> &'static Path {
    use std::path::PathBuf;
    lazy_static! {
        /// Path to the cache.
        static ref CACHE: PathBuf = crate::DIR.cache_dir().into();
    }
    &*CACHE
}

#[cfg(test)]
fn cache() -> &'static Path {
    lazy_static! {
        static ref CACHE: temp_dir::TempDir =
            temp_dir::TempDir::new().expect("Failed to create test cache dir");
    }
    CACHE.path()
}
