//! CLI tool to query the menu of canteens contained in the
//! [OpenMensa](https://openmensa.org) database.
//!
//! ![example](https://user-images.githubusercontent.com/11077981/137278085-75ec877a-dba0-44bb-a8dc-6c802e24178c.png)
//!
//! # Features
//!
//! - [X] Custom filters and favourites using CLI flags or the
//!       optional configuration file.
//! - [X] List canteens close to you based on GeoIP.
//! - [X] All request are cached locally.
//! - [X] Fuzzy date parsing based on
//!       [date_time_parser](https://lib.rs/crates/date_time_parser).
//! - [ ] List your favourite meals in canteens close to your location.
//!
//! # Installation
//!
//! ## Cargo
//!
//! ```console
//! $ cargo install --git https://github.com/MalteT/mensa
//! ```
//!
//! ## Nix
//!
//! This is a [Nix Flake](https://nixos.wiki/wiki/Flakes), add it
//! to your configuration or just test the application with:
//!
//! ```console
//! $ nix run github:MalteT/mensa
//! ```
//!
//! # Usage
//!
//! See `mensa --help`.
//!
//! - `mensa` will show meals served today for the default canteen mentioned
//!   in the configuration.
//!   If no such configuration exists, try `mensa --id 63`.
//!   You can find the id for your canteen using
//! - `mensa canteens` lists canteens near you based on your current
//!   IP in a default radius of 10km.
//! - `mensa tags` will list the currently known meal tags like "**12** Nuts".
//!
//!
//! # Configuration
//!
//! See [config.toml](config.toml) for an example. Copy the file to:
//! - `$XDG_CONFIG_DIR/mensa/config.toml` on **Linux**,
//! - `$HOME/Library/Application Support/mensa/config.toml` on **macOS**,
//! - ~~`{FOLDERID_RoamingAppData}\mensa\config.toml` on **Windows**~~
//!   I don't think it'll run on Windows.. 🤷‍♀️

use chrono::Duration;
use directories_next::ProjectDirs;
use lazy_static::lazy_static;
use strum::IntoEnumIterator;
use tracing::error;
use unicode_width::UnicodeWidthStr;

mod cache;
mod canteen;
mod config;
mod error;
mod geoip;
mod meal;

use config::{args::parse_human_date, MealsState, State};
use error::{Error, Result, ResultExt};
use meal::{tag::Tag, Meal};

use crate::{cache::fetch_json, canteen::Canteen, config::args::Command};

const ENDPOINT: &str = "https://openmensa.org/api/v2";
const MIN_TERM_WIDTH: usize = 20;

lazy_static! {
    static ref DIR: ProjectDirs =
        ProjectDirs::from("rocks", "tammena", "mensa").expect("Could not detect home directory");
    static ref TTL_CANTEENS: Duration = Duration::days(1);
    static ref TTL_MEALS: Duration = Duration::hours(1);
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
    // Construct client, load config and assemble cli args
    let state = State::assemble()?;
    // Clear cache if requested
    if state.cmd.clear_cache {
        cache::clear()?;
    }
    // Match over the user requested command
    match state.cmd.command {
        Some(Command::Meals(cmd)) => {
            let state = State::from(state.config, state.client, cmd);
            let meals = fetch_meals(&state)?;
            print_meals(&state, &meals);
        }
        Some(Command::Tomorrow(mut cmd)) => {
            // Works like the meals command. But we replace the date with tomorrow!
            cmd.date = parse_human_date("tomorrow").unwrap();
            let state = State::from(state.config, state.client, cmd);
            let meals = fetch_meals(&state)?;
            print_meals(&state, &meals);
        }
        Some(Command::Canteens(cmd)) => {
            let state = State::from(state.config, state.client, cmd);
            let canteens = Canteen::fetch(&state)?;
            println!();
            for canteen in canteens {
                canteen.print_to_terminal();
            }
        }
        Some(Command::Tags) => print_tags(),
        None => {
            let state = State::from(state.config, state.client, Default::default());
            let meals = fetch_meals(&state)?;
            print_meals(&state, &meals);
        }
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

fn fetch_meals(state: &MealsState) -> Result<Vec<Meal>> {
    let url = format!(
        "{}/canteens/{}/days/{}/meals",
        ENDPOINT,
        state.canteen_id()?,
        state.date()
    );
    fetch_json(&state.client, url, *TTL_MEALS)
}

fn print_meals(state: &MealsState, meals: &[Meal]) {
    let filter = state.get_filter();
    let favs = state.get_favs_rule();
    println!();
    for meal in meals {
        if filter.is_match(meal) {
            let is_fav = favs.is_match(meal);
            meal.print_to_terminal(state, is_fav);
            println!();
        }
    }
}

fn get_sane_terminal_dimensions() -> (usize, usize) {
    termion::terminal_size()
        .map(|(w, h)| (w as usize, h as usize))
        .map(|(w, h)| (w.max(MIN_TERM_WIDTH), h))
        .map_err(Error::UnableToGetTerminalSize)
        .log_warn()
        .unwrap_or((80, 80))
}
