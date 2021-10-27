//! <img src="https://raw.githubusercontent.com/MalteT/mensa/main/static/logo.svg?sanitize=true" alt="mensa CLI logo" width="400" align="right">
//!
//! [![tests](https://github.com/MalteT/mensa/actions/workflows/rust.yml/badge.svg)](https://github.com/MalteT/mensa/actions/workflows/rust.yml)
//!
//!
//! # mensa
//!
//! CLI tool to query the menu of canteens contained in the
//! [OpenMensa](https://openmensa.org) database.
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
//! - [X] List your favourite meals in canteens close to your location.
//! - [X] JSON Output
//!
//! ![example](https://raw.githubusercontent.com/MalteT/mensa/main/static/example-collection.png)
//!
//!
//! ## Installation
//!
//! ### Cargo
//!
//! **Only nightly Rust supported at the moment**.
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
//!
//! ### Examples
//!
//! ####
//! <details>
//!   <summary><b>Meals on monday</b> (<i>Click me!</i>)</summary>
//!
//!   You can omit the `-i/--id` if you've configured a default id in the config.toml.
//!
//!   ```console
//!   $ mensa meals -d mon -i 63
//!
//!    Leipzig, Mensa am Park
//!    ┊
//!    ┊ ╭───╴Bohnengemüse
//!    ┊ ├─╴Gemüsebeilage 🌱
//!    ┊ ╰╴( 0.55€ )
//!    ...
//!   ```
//! </details>
//!
//! <details>
//!   <summary><b>Canteens near your location</b> (<i>Click me!</i>)</summary>
//!
//!   ```console
//!   $ mensa canteens
//!
//!   70 Leipzig, Cafeteria Dittrichring
//!      Dittrichring 21, 04109 Leipzig
//!
//!   63 Leipzig, Mensa am Park
//!      Universitätsstraße 5, 04109 Leipzig
//!   ...
//!   ```
//! </details>
//!
//! <details>
//!   <summary><b>All currently known tags</b> (<i>Click me!</i>)</summary>
//!
//!   ```console
//!   $ mensa tags
//!
//!      0 Acidifier
//!        Contains artificial acidifier
//!
//!      1 Alcohol
//!        Contains alcohol
//!
//!      2 Antioxidant
//!        Contains an antioxidant
//!     ...
//!   ```
//! </details>
//!
//! <details>
//!   <summary><b>Meals of canteens close to your location next sunday</b> (<i>Click me!</i>)</summary>
//!
//!   ```console
//!   $ mensa meals close --date sun
//!
//!    Leipzig, Cafeteria Dittrichring
//!    ┊
//!    ┊ ╭───╴Vegetarisch gefüllte Zucchini
//!    ┊ ├─╴Vegetarisches Gericht 🧀
//!    ┊ ├╴Rucola-Kartoffelpüree
//!    ┊ ├╴Tomaten-Ratatouille-Soße
//!    ┊ ╰╴( 2.65€ )  2 11 12 19
//!
//!    Leipzig, Mensa am Park
//!    ┊
//!    ┊ ╭───╴Apfelrotkohl
//!    ┊ ├─╴Gemüsebeilage 🌱
//!    ┊ ╰╴( 0.55€ )  2
//!    ...
//!   ```
//! </details>
//!
//! <details>
//!   <summary><b>Count OpenMensa's canteens</b> (<i>Click me!</i>)</summary>
//!
//!   ```console
//!   $ mensa canteens --all --json | jq '.[].id' | wc -l
//!   704
//!   ```
//! </details>
//!
//! ## Configuration *(Optional)*
//!
//! See [config.toml](config.toml) for an example. Copy the file to:
//! - `$XDG_CONFIG_DIR/mensa/config.toml` on **Linux**,
//! - `$HOME/Library/Application Support/mensa/config.toml` on **macOS**,
//! - `{FOLDERID_RoamingAppData}\mensa\config.toml` on **Windows**

use std::io;

use cache::Cache;
use chrono::Duration;
use directories_next::ProjectDirs;
use lazy_static::lazy_static;
use serde::Serialize;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// Colorizes the output.
///
/// This will colorize for Stdout based on heuristics and colors
/// from the [`owo_colors`] library.
///
/// **Windows**: Automatic color defaults to no color at the moment!
// TODO: Make colors work on windows
macro_rules! color {
    ($what:expr; $($fn:ident),+) => {
        {
            #[cfg(not(windows))]
            {
                use owo_colors::{OwoColorize, Stream};
                use crate::config::args::ColorWhen;
                match crate::config::CONF.args.color {
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
            #[cfg(windows)]
            {
                use owo_colors::{OwoColorize};
                use crate::config::args::ColorWhen;
                match crate::config::CONF.args.color {
                    ColorWhen::Always => {
                        $what $(. $fn())+ .to_string()
                    }
                    ColorWhen::Automatic | ColorWhen::Never => {
                        $what.to_string()
                    }
                }
            }
        }
    };
}

/// Conditionally select one of two expressions.
///
/// The former will be used unless the `--plain` flag is specified.
macro_rules! if_plain {
    ($fancy:expr, $plain:expr) => {
        if cfg!(windows) || crate::config::CONF.args.plain {
            $plain
        } else {
            $fancy
        }
    };
}

/// Safer `println` which doesn't panic, but errors.
macro_rules! try_println {
    () => {
        try_println!("\n")
    };
    ($str:literal $(, $args:expr )* $(,)?) => ({
        use std::io::Write;
        writeln!(::std::io::stdout(), $str, $( $args ),* )
            .map_err(|why| crate::error::Error::Io(why, "printing"))
    })
}

mod cache;
mod canteen;
mod config;
mod error;
mod geoip;
mod meal;
mod pagination;
mod request;
mod tag;

use crate::{
    cache::CACHE,
    canteen::Canteen,
    config::{args::Command, CONF},
    error::{Error, Result, ResultExt},
    meal::Meal,
    tag::Tag,
};

const OPEN_MENSA_API: &str = "https://openmensa.org/api/v2";

lazy_static! {
    static ref DIR: ProjectDirs =
        ProjectDirs::from("rocks", "tammena", "mensa").expect("Could not detect home directory");
    static ref TTL_CANTEENS: Duration = Duration::days(1);
    static ref TTL_MEALS: Duration = Duration::hours(1);
}

fn main() -> Result<()> {
    match real_main() {
        Ok(_) => Ok(()),
        // Ignore broken pipe errors, but log them
        Err(Error::Io(err, _)) if err.kind() == io::ErrorKind::BrokenPipe => {
            info!("Pipe was closed");
            Ok(())
        }
        Err(why) => {
            error!("{}", why);
            Err(why)
        }
    }
}

fn real_main() -> Result<()> {
    // Initialize logger
    tracing_subscriber::FmtSubscriber::builder()
        .with_writer(::std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    // Clear cache if requested
    if CONF.args.clear_cache {
        CACHE.clear()?;
    }
    // Match over the user requested command
    match CONF.cmd() {
        Command::Meals(_) => {
            let mut canteens = Canteen::infer()?;
            Meal::print_for_all_canteens(&mut canteens)?;
        }
        Command::Canteens(_) => {
            let mut canteens = Canteen::infer()?;
            Canteen::print_all(&mut canteens)?;
        }
        Command::Tags => {
            Tag::print_all()?;
        }
    }
    Ok(())
}

fn get_sane_terminal_dimensions() -> (usize, usize) {
    const MIN_TERM_WIDTH: usize = 20;
    terminal_size::terminal_size()
        .map(|(w, h)| (w.0 as usize, h.0 as usize))
        .map(|(w, h)| (w.max(MIN_TERM_WIDTH), h))
        .ok_or(Error::UnableToGetTerminalSize)
        .log_warn()
        .unwrap_or((80, 80))
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    let stdout = std::io::stdout();
    let output = stdout.lock();
    let res = serde_json::to_writer_pretty(output, value);
    // This is done to catch broken pipe errors
    match res {
        Err(why) if why.is_io() => {
            // Propagate as simple io error.
            // BrokenPipe errors are catched in main
            Err(Error::Io(why.into(), "serializing json"))
        }
        Err(other) => Err(Error::Serializing(other, "writing meals as json")),
        Ok(()) => Ok(()),
    }
}
