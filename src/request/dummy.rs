//! This contains the [`DummyApi`] used for testing purposes.
use reqwest::StatusCode;

use crate::error::Result;

use super::{Api, Headers, Response};

/// A dummy API, serving local, deterministic Responses
#[derive(Debug)]
pub struct DummyApi;

impl Api for DummyApi {
    fn create() -> Result<Self> {
        Ok(DummyApi)
    }

    fn get<'url, S>(&self, url: &'url str, etag: Option<S>) -> Result<Response<'url>>
    where
        S: AsRef<str>,
    {
        if url == "http://invalid.local/test" {
            get_test_page(etag)
        } else {
            panic!("BUG: Invalid url in dummy api: {:?}", url)
        }
    }
}

/// GET http://invalid.local/test
fn get_test_page<S: AsRef<str>>(etag: Option<S>) -> Result<Response<'static>> {
    let etag = etag.map(|etag| etag.as_ref().to_owned());
    Ok(Response {
        url: "http://invalid.local/test",
        status: if etag == Some("static".into()) {
            StatusCode::NOT_MODIFIED
        } else {
            StatusCode::OK
        },
        headers: Headers {
            etag: Some("static".into()),
            this_page: Some(1),
            next_page: None,
            last_page: Some(1),
        },
        body: "It works".to_owned(),
    })
}
