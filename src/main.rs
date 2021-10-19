//! <img src="https://raw.githubusercontent.com/MalteT/mensa/main/static/logo.svg?sanitize=true" alt="mensa CLI logo" width="400" align="right">
//!
//! [![tests](https://github.com/MalteT/mensa/actions/workflows/rust.yml/badge.svg)](https://github.com/MalteT/mensa/actions/workflows/rust.yml)
//!
//! # mensa
//!
//! CLI tool to query the menu of canteens contained in the
//! [OpenMensa](https://openmensa.org) database.
//!
//! ![example](https://user-images.githubusercontent.com/11077981/137278085-75ec877a-dba0-44bb-a8dc-6c802e24178c.png)
//!
//! ## Features
//!
//! - [X] Runs on Linux, macOS and Windows.
//! - [X] Custom filters and favourites using CLI flags or the
//!       optional configuration file.
//! - [X] List canteens close to you based on GeoIP.
//! - [X] All request are cached locally.
//! - [X] Fuzzy date parsing based on
//!       [date_time_parser](https://lib.rs/crates/date_time_parser).
//! - [ ] List your favourite meals in canteens close to your location.
//!
//!
//! ## Installation
//!
//! ### Cargo
//!
//! ```console
//! $ cargo install --git https://github.com/MalteT/mensa
//! ```
//!
//! ### Nix
//!
//! This is a [Nix Flake](https://nixos.wiki/wiki/Flakes), add it
//! to your configuration or just test the application with:
//!
//! ```console
//! $ nix run github:MalteT/mensa
//! ```
//!
//!
//! ## Usage
//!
//! See `mensa --help`.
//!
//! - `mensa meals` will show meals served today for the default canteen
//!   mentioned in the configuration.
//!   If no such configuration exists, try `mensa meals --id 63`.
//!   You can find the id for your canteen using
//! - `mensa canteens` lists canteens near you based on your current
//!   IP in a default radius of 10km.
//! - `mensa tags` will list the currently known meal tags like "**12** Nuts".
//! - `mensa tomorrow` shortcut for `mensa meals -d tomorrow [...]`
//!
//!
//! ## Configuration
//!
//! See [config.toml](config.toml) for an example. Copy the file to:
//! - `$XDG_CONFIG_DIR/mensa/config.toml` on **Linux**,
//! - `$HOME/Library/Application Support/mensa/config.toml` on **macOS**,
//! - `{FOLDERID_RoamingAppData}\mensa\config.toml` on **Windows**

use chrono::Duration;
use directories_next::ProjectDirs;
use lazy_static::lazy_static;
use structopt::StructOpt;
use strum::IntoEnumIterator;
use tracing::error;
use unicode_width::UnicodeWidthStr;

/// Colorizes the output.
///
/// This will colorize for Stdout based on heuristics and colors
/// from the [`owo_colors`] library.
macro_rules! color {
    ($state:ident: $what:expr; $($fn:ident),+) => {
        {
            use owo_colors::{OwoColorize, Stream};
            use crate::config::args::ColorWhen;
            match $state.args.color {
                ColorWhen::Always => {
                    $what $(. $fn())+ .to_string()
                }
                ColorWhen::Automatic => {
                    $what.if_supports_color(Stream::Stdout,
                                            |txt| txt $(. $fn().to_string())+).to_string()
                }
                ColorWhen::Never => {
                    $what.to_string()
                }
            }
        }
    };
}

macro_rules! if_plain {
    ($state:ident: $fancy:expr, $plain:expr) => {
        if $state.args.plain {
            $plain
        } else {
            $fancy
        }
    };
}

mod cache;
mod canteen;
mod config;
mod error;
mod geoip;
mod meal;

use config::{
    args::{parse_human_date, Args, MealsCommand},
    State,
};
use error::{Error, Result, ResultExt};
use meal::{tag::Tag, Meal};

use crate::{canteen::Canteen, config::args::Command};

const ENDPOINT: &str = "https://openmensa.org/api/v2";
const MIN_TERM_WIDTH: usize = 20;

lazy_static! {
    static ref DIR: ProjectDirs =
        ProjectDirs::from("rocks", "tammena", "mensa").expect("Could not detect home directory");
    static ref TTL_CANTEENS: Duration = Duration::days(1);
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
    let args = Args::from_args();
    let state = State::assemble(&args)?;
    // Clear cache if requested
    if state.args.clear_cache {
        cache::clear()?;
    }
    // Match over the user requested command
    match &state.args.command {
        Some(Command::Meals(cmd)) => {
            let state = State::from(state, cmd);
            let meals = Meal::fetch(&state)?;
            Meal::print_all(&state, &meals);
        }
        Some(Command::Tomorrow(cmd)) => {
            // Works like the meals command. But we replace the date with tomorrow!
            let mut cmd = cmd.clone();
            cmd.date = parse_human_date("tomorrow").unwrap();
            let state = State::from(state, &cmd);
            let meals = Meal::fetch(&state)?;
            Meal::print_all(&state, &meals);
        }
        Some(Command::Canteens(cmd)) => {
            let state = State::from(state, cmd);
            let canteens = Canteen::fetch(&state)?;
            Canteen::print_all(&state, &canteens);
        }
        Some(Command::Tags) => print_tags(&state),
        None => {
            let cmd = MealsCommand::default();
            let state = State::from(state, &cmd);
            let meals = Meal::fetch(&state)?;
            Meal::print_all(&state, &meals);
        }
    }
    Ok(())
}

fn print_tags<Cmd>(state: &State<Cmd>) {
    for tag in Tag::iter() {
        println!();
        const ID_WIDTH: usize = 4;
        const TEXT_INDENT: &str = "     ";
        let emoji = if state.args.plain && tag.is_primary() {
            format!("{:>width$}", "-", width = ID_WIDTH)
        } else {
            let emoji = tag.as_id(state);
            let emoji_len = emoji.width();
            format!(
                "{}{}",
                " ".repeat(ID_WIDTH.saturating_sub(emoji_len)),
                emoji
            )
        };
        let description_width = get_sane_terminal_dimensions().0;
        let description = textwrap::fill(
            tag.describe(),
            textwrap::Options::new(description_width)
                .initial_indent(TEXT_INDENT)
                .subsequent_indent(TEXT_INDENT),
        );
        println!(
            "{} {}\n{}",
            color!(state: emoji; bright_yellow, bold),
            color!(state: tag; bold),
            color!(state: description; bright_black),
        );
    }
}

fn get_sane_terminal_dimensions() -> (usize, usize) {
    terminal_size::terminal_size()
        .map(|(w, h)| (w.0 as usize, h.0 as usize))
        .map(|(w, h)| (w.max(MIN_TERM_WIDTH), h))
        .ok_or(Error::UnableToGetTerminalSize)
        .log_warn()
        .unwrap_or((80, 80))
}
