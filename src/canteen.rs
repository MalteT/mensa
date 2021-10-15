use chrono::Duration;
use reqwest::{blocking::Client, IntoUrl, Url};
use serde::{de::DeserializeOwned, Deserialize};
use tracing::info;

use std::marker::PhantomData;

use crate::{
    cache, complete_lat_long,
    config::args::CanteensCommand,
    error::{Error, Result},
    ENDPOINT, TTL_CANTEENS,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Canteen {
    id: usize,
    name: String,
    //city: String,
    address: String,
    //oordinates: [f32; 2],
}

impl Canteen {
    pub fn print_to_terminal(&self) {
        use termion::{color, style};
        println!(
            "{}{}{:>4} {}{}{}\n     {}{}{}",
            style::Bold,
            color::Fg(color::LightYellow),
            self.id,
            color::Fg(color::Reset),
            self.name,
            style::Reset,
            color::Fg(color::LightBlack),
            self.address,
            color::Fg(color::Reset),
        );
    }

    pub fn fetch(client: &Client, cmd: &CanteensCommand) -> Result<Vec<Self>> {
        let (lat, long) = complete_lat_long(client, cmd)?;
        let url = format!(
            "{}/canteens?near[lat]={}&near[lng]={}&near[dist]={}",
            ENDPOINT, lat, long, cmd.radius,
        );
        info!(
            "Fetching canteens for lat: {}, long: {} with radius: {}",
            lat, long, cmd.radius
        );
        PaginatedList::from(client, url, *TTL_CANTEENS)?.try_flatten_and_collect()
    }
}

struct PaginatedList<'client, T>
where
    T: DeserializeOwned,
{
    client: &'client Client,
    url: Url,
    // Key/Value pairs from the original url
    original_query: Vec<(String, String)>,
    page: usize,
    ttl: Duration,
    is_done: bool,
    __item: PhantomData<T>,
}

impl<'client, T> PaginatedList<'client, T>
where
    T: DeserializeOwned,
{
    pub fn from<U: IntoUrl>(client: &'client Client, url: U, ttl: Duration) -> Result<Self> {
        // Parse the url and store the query seperately.
        let url = url.into_url().map_err(Error::InvalidUrl)?;
        let original_query = url.query_pairs().into_owned().collect();
        // Page count is 1-based
        Ok(PaginatedList {
            client,
            original_query,
            ttl,
            is_done: false,
            url,
            page: 1,
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
        // Do not send requests after the first error
        if self.is_done {
            return None;
        }
        // Drop the query and replace it with the original + the page number
        self.url
            .query_pairs_mut()
            .clear()
            .extend_pairs(self.original_query.iter())
            .append_pair("page", &self.page.to_string());
        info!("Requesting page {}: {}", self.page, self.url);
        let val: Result<Vec<T>> = cache::get(self.client, &self.url, self.ttl, |text, headers| {
            eprintln!("{:?}", headers);

            serde_json::from_str(&text)
                .map_err(|why| Error::Deserializing(why, "fetching canteen list"))
        });
        match val {
            Ok(value) => {
                self.page += 1;
                // OpenMensa returns empty lists for large pages
                if value.is_empty() {
                    self.is_done = true;
                }
                Some(Ok(value))
            }
            Err(why) => {
                // Don't continue after an error occurred
                self.is_done = true;
                Some(Err(why))
            }
        }
    }
}
