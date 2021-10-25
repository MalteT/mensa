use ::reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[cfg(not(test))]
mod reqwest;
#[cfg(not(test))]
pub use self::reqwest::ReqwestApi as DefaultApi;

#[cfg(test)]
mod dummy;
#[cfg(test)]
pub use self::dummy::DummyApi as DefaultApi;

/// Assortment of headers relevant to the program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Headers {
    pub etag: Option<String>,
    pub this_page: Option<usize>,
    pub next_page: Option<String>,
    pub last_page: Option<usize>,
}

/// A subset of a Response, derived from [`reqwest::Response`].
pub struct Response<'url> {
    pub url: &'url str,
    pub status: StatusCode,
    pub headers: Headers,
    pub body: String,
}

/// Generalized API endpoint.
///
/// This abstracts away from the real thing to allow for deterministic local
/// tests with a DummyApi.
pub trait Api
where
    Self: Sized,
{
    /// Create the Api.
    fn create() -> Result<Self>;

    /// Send a get request.
    ///
    /// Optionally attach an `If-None-Match` header, if `etag` is `Some`.
    fn get<'url, S>(&self, url: &'url str, etag: Option<S>) -> Result<Response<'url>>
    where
        S: AsRef<str>;
}
