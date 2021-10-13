use reqwest::blocking::Client;
use serde::Deserialize;
use strum::IntoEnumIterator;
use tracing::{error, info};
use unicode_width::UnicodeWidthStr;

use std::time::Duration;

mod cache;
mod config;
mod error;
mod meal;

use config::{args::PlacesCommand, CONFIG};
use error::{Error, Result, ResultExt};
use meal::{tag::Tag, Meal};

use crate::{cache::get_json, config::args::Command, error::pass_info};

const ENDPOINT: &str = "https://openmensa.org/api/v2";
const MIN_TERM_WIDTH: usize = 20;

fn main() -> Result<()> {
    let res = real_main();
    match res {
        Ok(_) => {}
        Err(ref why) => error!("{}", why),
    }
    res
}

fn real_main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let client = Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();

    match CONFIG.args.command {
        Some(Command::Show) | None => {
            let meals = fetch_meals(&client)?;
            // TODO: More pizzazz
            print_meals(&meals);
        }
        Some(Command::Places(ref cmd)) => {
            let places = fetch_mensas(&client, cmd)?;
            println!();
            for place in places {
                place.print_to_terminal();
            }
        }
        Some(Command::Tags) => print_tags(),
    }
    Ok(())
}

fn print_tags() {
    use termion::{color, style};
    println!();
    for tag in Tag::iter() {
        const EMOJI_WIDTH: usize = 4;
        const TEXT_INDENT: &str = "     ";
        let emoji = tag.as_emoji();
        let emoji_len = emoji.width();
        let emoji_padded = format!(
            "{}{}",
            " ".repeat(EMOJI_WIDTH.saturating_sub(emoji_len)),
            emoji
        );
        let description_width = get_sane_terminal_dimensions().0;
        let description = textwrap::fill(
            tag.describe(),
            textwrap::Options::new(description_width)
                .initial_indent(TEXT_INDENT)
                .subsequent_indent(TEXT_INDENT),
        );
        println!(
            "{}{}{}{} {}{}\n{}{}{}",
            style::Bold,
            color::Fg(color::LightYellow),
            emoji_padded,
            color::Fg(color::Reset),
            tag,
            style::Reset,
            color::Fg(color::LightBlack),
            description,
            color::Fg(color::Reset),
        );
    }
}

fn fetch_mensas(client: &Client, cmd: &PlacesCommand) -> Result<Vec<Mensa>> {
    let (lat, long) = complete_lat_long(client, cmd)?;
    let url = format!(
        "{}/canteens?near[lat]={}&near[lng]={}&near[dist]={}",
        ENDPOINT, lat, long, cmd.radius,
    );
    info!(
        "Fetching mensas for lat: {}, long: {} with radius: {}",
        lat, long, cmd.radius
    );
    client
        .get(pass_info(url))
        .send()?
        .json()
        .map_err(Error::FetchingMensas)
}

fn complete_lat_long(client: &Client, cmd: &PlacesCommand) -> Result<(f32, f32)> {
    Ok(if cmd.lat.is_none() || cmd.long.is_none() {
        let guessed = fetch_lat_long_for_ip(client)?;
        (
            cmd.lat.unwrap_or(guessed.latitude),
            cmd.long.unwrap_or(guessed.longitude),
        )
    } else {
        // Cannot panic, due to above if
        (cmd.lat.unwrap(), cmd.long.unwrap())
    })
}

#[derive(Debug, Clone, Deserialize)]
struct Mensa {
    id: usize,
    name: String,
    city: String,
    address: String,
    coordinates: [f32; 2],
}

impl Mensa {
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
}

fn fetch_meals(client: &Client) -> Result<Vec<Meal>> {
    let url = format!(
        "{}/canteens/{}/days/{}/meals",
        ENDPOINT,
        CONFIG.mensa_id()?,
        CONFIG.date()
    );
    get_json(client, url, chrono::Duration::minutes(1))
}

fn print_meals(meals: &[Meal]) {
    let filter = CONFIG.get_filter();
    let favs = CONFIG.get_favs_rule();
    println!();
    for meal in meals {
        if filter.is_match(meal) {
            let is_fav = favs.is_match(meal);
            meal.print_to_terminal(is_fav);
            println!();
        }
    }
}

fn fetch_lat_long_for_ip(client: &Client) -> Result<LatLong> {
    let url = "https://api.geoip.rs";
    get_json(client, url, chrono::Duration::minutes(30))
}

#[derive(Debug, Clone, Deserialize)]
struct LatLong {
    latitude: f32,
    longitude: f32,
}

fn get_sane_terminal_dimensions() -> (usize, usize) {
    termion::terminal_size()
        .map(|(w, h)| (w as usize, h as usize))
        .map(|(w, h)| (w.max(MIN_TERM_WIDTH), h))
        .map_err(Error::UnableToGetTerminalSize)
        .log_warn()
        .unwrap_or((80, 80))
}
