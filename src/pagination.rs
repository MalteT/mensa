use chrono::Duration;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

use std::marker::PhantomData;

use crate::{
    cache,
    error::{Error, Result},
};

pub struct PaginatedList<'client, T>
where
    T: DeserializeOwned,
{
    client: &'client Client,
    next_page: Option<String>,
    ttl: Duration,
    __item: PhantomData<T>,
}

impl<'client, T> PaginatedList<'client, T>
where
    T: DeserializeOwned,
{
    pub fn from<S: AsRef<str>>(client: &'client Client, url: S, ttl: Duration) -> Result<Self> {
        Ok(PaginatedList {
            client,
            ttl,
            next_page: Some(url.as_ref().into()),
            __item: PhantomData,
        })
    }
}

impl<'client, T> PaginatedList<'client, T>
where
    T: DeserializeOwned,
{
    pub fn try_flatten_and_collect(self) -> Result<Vec<T>> {
        let mut ret = vec![];
        for value in self {
            let value = value?;
            ret.extend(value);
        }
        Ok(ret)
    }
}

impl<'client, T> Iterator for PaginatedList<'client, T>
where
    T: DeserializeOwned,
{
    type Item = Result<Vec<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        // This will yield until no next_page is available
        let curr_page = self.next_page.take()?;
        let res = cache::fetch(self.client, &curr_page, self.ttl, |text, headers| {
            let val = serde_json::from_str::<Vec<_>>(&text)
                .map_err(|why| Error::Deserializing(why, "fetching json in pagination iterator"))?;
            Ok((val, headers.this_page, headers.next_page, headers.last_page))
        });
        match res {
            Ok((val, this_page, next_page, last_page)) => {
                // Only update next_page, if we're not on the last page!
                // This should be safe for all cases
                if this_page.unwrap_or_default() < last_page.unwrap_or_default() {
                    // OpenMensa returns empty lists for large pages
                    // this is just to keep me sane
                    if !val.is_empty() {
                        self.next_page = next_page;
                    }
                }
                Some(Ok(val))
            }
            Err(why) => {
                // Implicitly does not set the next_page url, so
                // this iterator is done now
                Some(Err(why))
            }
        }
    }
}
