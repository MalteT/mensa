use std::{io::Write, path::PathBuf};

use cacache::Metadata;
use itertools::Itertools;
use lazy_static::lazy_static;
use tracing::info;

use super::Cache;

use crate::{
    error::{Error, Result},
    request::Headers,
    DIR,
};

lazy_static! {
    /// Path to the cache.
    static ref CACHE: PathBuf  = DIR.cache_dir().into();
}

pub struct Cacache;

impl Cache for Cacache
where
    Self: Sized,
{
    fn init() -> Result<Self> {
        Ok(Cacache)
    }

    fn write(&self, headers: &Headers, url: &str, text: &str) -> Result<()> {
        let header_serialized = serde_json::to_value(headers.clone())
            .map_err(|why| Error::Serializing(why, "writing headers to cache"))?;
        let mut writer = cacache::WriteOpts::new()
            .metadata(header_serialized)
            .open_sync(&*CACHE, url)
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

    fn read(&self, meta: &Metadata) -> Result<String> {
        cacache::read_hash_sync(&*CACHE, &meta.integrity)
            .map_err(|why| Error::Cache(why, "reading value"))
            .and_then(|raw| String::from_utf8(raw).map_err(Error::DecodingUtf8))
    }

    fn meta(&self, url: &str) -> Result<Option<Metadata>> {
        cacache::metadata_sync(&*CACHE, url).map_err(|why| Error::Cache(why, "reading metadata"))
    }

    fn clear(&self) -> Result<()> {
        cacache::clear_sync(&*CACHE).map_err(|why| Error::Cache(why, "clearing"))
    }

    fn list(&self) -> Result<Vec<Metadata>> {
        cacache::list_sync(&*CACHE)
            .map(|res| res.map_err(|why| Error::Cache(why, "listing")))
            .try_collect()
    }
}
