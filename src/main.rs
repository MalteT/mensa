use chrono::Duration;
use directories_next::ProjectDirs;
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde::Deserialize;
use strum::IntoEnumIterator;
use tracing::error;
use unicode_width::UnicodeWidthStr;

use std::time::Duration as StdDuration;

mod cache;
mod canteen;
mod config;
mod error;
mod meal;

use config::{args::CanteensCommand, CONFIG};
use error::{Error, Result, ResultExt};
use meal::{tag::Tag, Meal};

use crate::{cache::get_json, canteen::Canteen, config::args::Command};

const ENDPOINT: &str = "https://openmensa.org/api/v2";
const MIN_TERM_WIDTH: usize = 20;

lazy_static! {
    static ref DIR: ProjectDirs =
        ProjectDirs::from("rocks", "tammena", "mensa").expect("Could not detect home directory");
    static ref TTL_GEOIP: Duration = Duration::minutes(5);
    static ref TTL_CANTEENS: Duration = Duration::days(1);
    static ref TTL_MEALS: Duration = Duration::hours(1);
    static ref REQUEST_TIMEOUT: StdDuration = StdDuration::from_secs(10);
}

fn main() -> Result<()> {
    let res = real_main();
    match res {
        Ok(_) => {}
        Err(ref why) => error!("{}", why),
    }
    res
}

fn real_main() -> Result<()> {
    // Initialize logger
    tracing_subscriber::fmt::init();
    // Construct client used for all requests
    let client = Client::builder().timeout(*REQUEST_TIMEOUT).build().unwrap();
    // Clear cache if requested
    if CONFIG.args.clear_cache {
        cache::clear()?;
    }

    match CONFIG.args.command {
        Some(Command::Show) | None => {
            let meals = fetch_meals(&client)?;
            // TODO: More pizzazz
            print_meals(&meals);
        }
        Some(Command::Canteens(ref cmd)) => {
            let canteens = Canteen::fetch(&client, cmd)?;
            println!();
            for canteen in canteens {
                canteen.print_to_terminal();
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

fn complete_lat_long(client: &Client, cmd: &CanteensCommand) -> Result<(f32, f32)> {
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

fn fetch_meals(client: &Client) -> Result<Vec<Meal>> {
    let url = format!(
        "{}/canteens/{}/days/{}/meals",
        ENDPOINT,
        CONFIG.canteen_id()?,
        CONFIG.date()
    );
    get_json(client, url, *TTL_MEALS)
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
    get_json(client, url, *TTL_GEOIP)
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
