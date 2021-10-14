use chrono::NaiveDate;
use regex::Regex;
use structopt::StructOpt;

use std::path::PathBuf;

use crate::{
    error::{Error, Result},
    meal::tag::Tag,
};

use super::PriceTags;

#[derive(Debug, StructOpt)]
pub struct Args {
    /// Canteen ID for which to fetch meals.
    #[structopt(long = "id", short = "i", env = "MENSA_ID")]
    pub canteen_id: Option<usize>,

    /// Date for which to display information.
    ///
    /// Try values like `tomorrow`, `wed`, etc.
    #[structopt(long, short,
                env = "MENSA_DATE",
                parse(try_from_str = parse_human_date),
                default_value = "today")]
    pub date: NaiveDate,

    /// Specify which price tags should be displayed
    #[structopt(long, short, env = "MENSA_PRICES", possible_values = &PriceTags::variants())]
    pub price: Option<Vec<PriceTags>>,

    /// Path to the configuration file.
    #[structopt(long, short, env = "MENSA_CONFIG", name = "PATH")]
    pub config: Option<PathBuf>,

    #[structopt(long, env = "MENSA_OVERWRITE_FILTER", takes_value = false)]
    pub overwrite_filter: bool,

    #[structopt(long, env = "MENSA_FILTER_NAME_ADD")]
    pub filter_name: Vec<Regex>,

    #[structopt(long, env = "MENSA_FILTER_NAME_SUB")]
    pub no_filter_name: Vec<Regex>,

    #[structopt(long, env = "MENSA_FILTER_TAG_ADD", parse(try_from_str = serde_plain::from_str))]
    pub filter_tag: Vec<Tag>,

    #[structopt(long, env = "MENSA_FILTER_TAG_SUB", parse(try_from_str = serde_plain::from_str))]
    pub no_filter_tag: Vec<Tag>,

    #[structopt(long, env = "MENSA_FILTER_CATEGORY_ADD")]
    pub filter_cat: Vec<Regex>,

    #[structopt(long, env = "MENSA_FILTER_CATEGORY_SUB")]
    pub no_filter_cat: Vec<Regex>,

    #[structopt(long, env = "MENSA_OVERWRITE_FAVS", takes_value = false)]
    pub overwrite_favs: bool,

    #[structopt(long, env = "MENSA_FAVS_NAME_ADD")]
    pub favs_name: Vec<Regex>,

    #[structopt(long, env = "MENSA_FAVS_NAME_SUB")]
    pub no_favs_name: Vec<Regex>,

    #[structopt(long, env = "MENSA_FAVS_TAG_ADD", parse(try_from_str = serde_plain::from_str))]
    pub favs_tag: Vec<Tag>,

    #[structopt(long, env = "MENSA_FAVS_TAG_SUB", parse(try_from_str = serde_plain::from_str))]
    pub no_favs_tag: Vec<Tag>,

    #[structopt(long, env = "MENSA_FAVS_CATEGORY_ADD")]
    pub favs_cat: Vec<Regex>,

    #[structopt(long, env = "MENSA_FAVS_CATEGORY_SUB")]
    pub no_favs_cat: Vec<Regex>,

    #[structopt(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// List canteens close to you.
    Canteens(CanteensCommand),
    /// List all known tags.
    Tags,
    /// Default. Show meals.
    Show,
}

#[derive(Debug, StructOpt)]
pub struct CanteensCommand {
    /// Latitude of your position. If omitted, geoip will be used to guess it.
    #[structopt(long)]
    pub lat: Option<f32>,
    /// Longitude of your position. If omitted, geoip will be used to guess it.
    #[structopt(long)]
    pub long: Option<f32>,
    /// Maximum distance of potential canteens from your position in km.
    #[structopt(long, short, default_value = "10")]
    pub radius: f32,
}

fn parse_human_date(inp: &str) -> Result<NaiveDate> {
    date_time_parser::DateParser::parse(inp).ok_or(Error::InvalidDateInArgs)
}

impl Default for Command {
    fn default() -> Self {
        Self::Show
    }
}
