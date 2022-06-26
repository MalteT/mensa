use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;

use std::time::Duration as StdDuration;

use crate::error::{Error, Result};

use super::{Api, Headers, Response};

lazy_static! {
    /// Regex to find the next page in a link header
    /// Probably only applicable to the current version of the openmensa API.
    // TODO: Improve this. How do these LINK headers look in general?
    static ref LINK_NEXT_PAGE_RE: Regex = Regex::new(r#"<([^>]*)>; rel="next""#).unwrap();
    static ref REQUEST_TIMEOUT: StdDuration = StdDuration::from_secs(10);
}

/// Real api accessing the inter-webs.
#[derive(Debug)]
pub struct ReqwestApi {
    client: Client,
}

impl Api for ReqwestApi {
    fn create() -> Result<Self> {
        let client = Client::builder()
            .timeout(*REQUEST_TIMEOUT)
            .build()
            .map_err(Error::Reqwest)?;
        Ok(ReqwestApi { client })
    }

    fn get<'url, S>(&self, url: &'url str, etag: Option<S>) -> Result<super::Response<'url>>
    where
        S: AsRef<str>,
    {
        let mut builder = self.client.get(url);
        if let Some(etag) = etag {
            let etag_key = reqwest::header::IF_NONE_MATCH;
            builder = builder.header(etag_key, etag.as_ref());
        }
        let resp = builder.send().map_err(Error::Reqwest)?;
        Ok(Response {
            url,
            status: resp.status(),
            headers: resp.headers().clone().into(),
            body: resp.text().map_err(Error::Reqwest)?,
        })
    }
}

impl From<reqwest::header::HeaderMap> for Headers {
    fn from(map: reqwest::header::HeaderMap) -> Self {
        use reqwest::header::*;
        let etag = map
            .get(ETAG)
            .and_then(|raw| {
                let utf8 = raw.to_str().ok()?;
                Some(utf8.to_string())
            });
        let this_page = map
            .get("x-current-page")
            .and_then(|raw| {
                let utf8 = raw.to_str().ok()?;
                utf8.parse().ok()
            });
        let next_page = map
            .get(LINK)
            .and_then(|raw| {
                let utf8 = raw.to_str().ok()?;
                let captures = LINK_NEXT_PAGE_RE.captures(utf8)?;
                Some(captures[1].to_owned())
            });
        let last_page = map
            .get("x-total-pages")
            .and_then(|raw| {
                let utf8 = raw.to_str().ok()?;
                utf8.parse().ok()
            });
        Self {
            etag,
            this_page,
            last_page,
            next_page,
        }
    }
}
