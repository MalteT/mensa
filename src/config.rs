use serde::Deserialize;
use chrono::NaiveDate;
use lazy_static::lazy_static;
use structopt::StructOpt;

use std::path::PathBuf;

use crate::error::{Error, Result};

lazy_static! {
    pub static ref CONFIG: Config = Config {
        args: Args::from_args()
    };
}

pub struct Config {
    args: Args,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {

}

#[derive(Debug, StructOpt)]
pub struct Args {
    /// Mensa ID for which to fetch things.
    #[structopt(long = "id", short = "i", env = "MENSA_ID")]
    pub mensa_id: Option<usize>,
    /// Date for which to display information.
    ///
    /// Try values like `tomorrow`, `wed`, etc.
    #[structopt(long, short,
                env = "MENSA_DATE",
                parse(try_from_str = parse_human_date),
                default_value = "today")]
    pub date: NaiveDate,
    /// Path to the configuration file.
    #[structopt(long, short, env = "MENSA_CONFIG", default_value = "~/.config/mensa/config.toml", name = "PATH")]
    pub config: PathBuf,
}

impl Config {
    pub fn mensa_id(&self) -> usize {
        match self.args.mensa_id {
            Some(id) => id,
            None => todo!("Config not done yet"),
        }
    }

    pub fn date(&self) -> &chrono::NaiveDate {
        &self.args.date
    }
}

fn parse_human_date(inp: &str) -> Result<NaiveDate> {
    date_time_parser::DateParser::parse(inp).ok_or(Error::InvalidDateInArgs)
}
