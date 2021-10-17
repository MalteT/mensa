use chrono::Duration;
use reqwest::blocking::Client;
use serde::{de::DeserializeOwned, Deserialize};
use tracing::info;

use std::marker::PhantomData;

use crate::{
    cache,
    config::CanteensState,
    error::{Error, Result},
    geoip, get_sane_terminal_dimensions, ENDPOINT, TTL_CANTEENS,
};

const ADRESS_INDENT: &str = "     ";

#[derive(Debug, Clone, Deserialize)]
pub struct Canteen {
    id: usize,
    name: String,
    //city: String,
    address: String,
    //oordinates: [f32; 2],
}

impl Canteen {
    pub fn print(&self) {
        use termion::{color, style};
        let (width, _) = get_sane_terminal_dimensions();
        let address = textwrap::fill(
            &self.address,
            textwrap::Options::new(width)
                .initial_indent(ADRESS_INDENT)
                .subsequent_indent(ADRESS_INDENT),
        );
        println!(
            "{}{}{:>4} {}{}{}\n{}{}{}",
            style::Bold,
            color::Fg(color::LightYellow),
            self.id,
            color::Fg(color::Reset),
            self.name,
            style::Reset,
            color::Fg(color::LightBlack),
            address,
            color::Fg(color::Reset),
        );
    }

    pub fn fetch(state: &CanteensState) -> Result<Vec<Self>> {
        let url = if state.cmd.all {
            info!("Fetching all canteens");
            format!("{}/canteens", ENDPOINT)
        } else {
            let (lat, long) = geoip::fetch(state)?;
            info!(
                "Fetching canteens for lat: {}, long: {} with radius: {}",
                lat, long, state.cmd.radius
            );
            format!(
                "{}/canteens?near[lat]={}&near[lng]={}&near[dist]={}",
                ENDPOINT, lat, long, state.cmd.radius,
            )
        };
        PaginatedList::from(&state.client, url, *TTL_CANTEENS)?.try_flatten_and_collect()
    }

    pub fn print_all(canteens: &[Self]) {
        println!();
        for canteen in canteens {
            canteen.print();
        }
    }
}

struct PaginatedList<'client, T>
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
