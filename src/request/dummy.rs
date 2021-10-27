//! This contains the [`DummyApi`] used for testing purposes.
use std::{collections::HashMap, sync::RwLock};

use reqwest::StatusCode;

use crate::error::Result;

use super::{Api, Headers, Response};

#[derive(Debug, Clone)]
struct KnownResp {
    etag: Option<String>,
    value: String,
    this_page: Option<usize>,
    next_page: Option<String>,
    last_page: Option<usize>,
}

/// A dummy API, serving local, deterministic Responses
#[derive(Debug)]
pub struct DummyApi {
    known: RwLock<HashMap<String, KnownResp>>,
}

impl Api for DummyApi {
    fn create() -> Result<Self> {
        Ok(DummyApi {
            known: RwLock::new(HashMap::new()),
        })
    }

    fn get<'url, S>(&self, url: &'url str, etag: Option<S>) -> Result<Response<'url>>
    where
        S: AsRef<str>,
    {
        let read = self.known.read().expect("Reading known urls failed");
        let etag = etag.map(|etag| etag.as_ref().to_owned());
        match read.get(url) {
            Some(resp) => {
                let resp = resp.clone();
                Ok(Response {
                    url,
                    status: status_from_etags(&resp.etag, &etag),
                    headers: Headers {
                        etag: resp.etag,
                        this_page: resp.this_page,
                        next_page: resp.next_page,
                        last_page: resp.last_page,
                    },
                    body: resp.value,
                })
            }
            None => panic!("BUG: Invalid url in dummy api: {:?}", url),
        }
    }
}

impl DummyApi {
    /// Register a single page.
    pub fn register_single(&self, url: &str, value: &str, etag: Option<&str>) {
        self.register(url, value, etag, Some(1), None, Some(1))
    }

    /// Register multiple subsequent pages.
    ///
    /// `pages` maps urls to pairs of values and optional etags.
    pub fn register_pages(&self, pages: &[(&str, &str, Option<&str>)]) {
        let mut page = 1;
        let last_page = pages.len();
        let mut pages = pages.iter().peekable();
        while let Some((url, value, etag)) = pages.next() {
            let next = pages.peek().map(|(url, _, _)| *url);
            self.register(url, value, *etag, Some(page), next, Some(last_page));
            page += 1;
        }
    }

    fn register(
        &self,
        url: &str,
        value: &str,
        etag: Option<&str>,
        this_page: Option<usize>,
        next_page: Option<&str>,
        last_page: Option<usize>,
    ) {
        let mut write = self.known.write().expect("Writing known urls failed");
        let etag = etag.map(str::to_owned);
        let next_page = next_page.map(str::to_owned);
        let old = write.insert(
            url.to_owned(),
            KnownResp {
                etag,
                value: value.to_owned(),
                this_page,
                next_page,
                last_page,
            },
        );
        if old.is_some() {
            panic!("Adress {:?} already registered!", url);
        }
    }
}

fn status_from_etags(old: &Option<String>, new: &Option<String>) -> StatusCode {
    match (old, new) {
        (Some(old), Some(new)) if old == new => StatusCode::NOT_MODIFIED,
        _ => StatusCode::OK,
    }
}
