use std::{collections::BTreeMap, sync::RwLock};

use cacache::Metadata;
use ssri::Integrity;

use super::Cache;

use crate::{
    error::{Error, Result},
    request::Headers,
};

#[derive(Debug)]
struct Entry {
    meta: Metadata,
    text: String,
}

pub struct DummyCache {
    /// The real, cacache-based implementation takes only immutable references
    /// and the API is adopted to handle that. Thus we'll have to do our
    /// own interior mutability here.
    ///
    /// This maps paths to entries.
    content: RwLock<BTreeMap<String, Entry>>,
}

impl Cache for DummyCache {
    fn init() -> Result<Self> {
        Ok(DummyCache {
            content: RwLock::new(BTreeMap::new()),
        })
    }

    fn read(&self, meta: &Metadata) -> Result<String> {
        let path = path_from_integrity(&meta.integrity);
        let read = self.content.read().expect("Reading cache failed");
        let entry = read
            .get(&path)
            .expect("BUG: Metadata exists, but entry does not!");
        Ok(entry.text.clone())
    }

    fn write(&self, headers: &Headers, url: &str, text: &str) -> Result<()> {
        let mut write = self.content.write().expect("Writing cache failed");
        let hash = path_from_key(url);
        let meta = assemble_meta(headers, url, text)?;
        write.insert(
            hash,
            Entry {
                meta,
                text: text.to_owned(),
            },
        );
        Ok(())
    }

    fn meta(&self, url: &str) -> Result<Option<Metadata>> {
        let hash = path_from_key(url);
        let read = self.content.read().expect("Reading cache failed");
        let entry = read.get(&hash);
        match entry {
            Some(entry) => Ok(Some(clone_metadata(&entry.meta))),
            None => Ok(None),
        }
    }

    fn clear(&self) -> Result<()> {
        self.content.write().expect("Writing cache failed").clear();
        Ok(())
    }

    fn list(&self) -> Result<Vec<Metadata>> {
        let read = self.content.read().expect("Reading cache failed");
        let list = read
            .values()
            .map(|entry| clone_metadata(&entry.meta))
            .collect();
        Ok(list)
    }
}

fn path_from_key(key: &str) -> String {
    let integrity = Integrity::from(key);
    path_from_integrity(&integrity)
}

fn path_from_integrity(integrity: &Integrity) -> String {
    let mut path = String::new();
    let (algorithm, digest) = integrity.to_hex();
    path += &algorithm.to_string();
    path += &digest;
    path
}

fn clone_metadata(meta: &Metadata) -> Metadata {
    Metadata {
        key: meta.key.clone(),
        integrity: meta.integrity.clone(),
        time: meta.time,
        size: meta.size,
        metadata: meta.metadata.clone(),
    }
}

fn assemble_meta(headers: &Headers, url: &str, text: &str) -> Result<Metadata> {
    let time = chrono::Utc::now();
    Ok(Metadata {
        key: url.to_owned(),
        integrity: Integrity::from(url),
        time: time.timestamp_millis() as u128,
        size: text.len(),
        metadata: serde_json::to_value(headers)
            .map_err(|why| Error::Serializing(why, "converting headers to json"))?,
    })
}
