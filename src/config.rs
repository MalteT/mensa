use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use structopt::StructOpt;

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    config::filter::TagFilter,
    error::{pass_info, Error, Result, ResultExt},
    meal::Tag,
};

mod filter;

use self::filter::{CategoryFilter, Filter};

lazy_static! {
    pub static ref CONFIG: Config = Config::assemble().log_panic();
}

#[derive(Debug)]
pub struct Config {
    args: Args,
    file: Option<ConfigFile>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize)]
pub enum PriceTags {
    Student,
    Employee,
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
struct ConfigFile {
    #[serde(default)]
    default_mensa_id: Option<usize>,
    #[serde(default)]
    price_tags: HashSet<PriceTags>,
    #[serde(default)]
    filter: Filter,
}

impl ConfigFile {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = fs::read_to_string(path).map_err(Error::ReadingConfig)?;
        toml::from_str(&file).map_err(Error::DeserializingConfig)
    }
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
    #[structopt(long, short, env = "MENSA_CONFIG", name = "PATH")]
    pub config: Option<PathBuf>,
    #[structopt(long, env = "MENSA_OVERWRITE_FILTER", takes_value = false)]
    pub overwrite_filter: bool,
    #[structopt(long, env = "MENSA_FILTER_TAG_ALLOW", parse(try_from_str = serde_plain::from_str))]
    pub allow_tag: Vec<Tag>,
    #[structopt(long, env = "MENSA_FILTER_TAG_DENY", parse(try_from_str = serde_plain::from_str))]
    pub deny_tag: Vec<Tag>,
    #[structopt(long, env = "MENSA_FILTER_CATEGORY_ALLOW")]
    pub allow_category: Vec<Regex>,
    #[structopt(long, env = "MENSA_FILTER_CATEGORY_DENY")]
    pub deny_category: Vec<Regex>,
    #[structopt(long, short, env = "MENSA_PRICES")]
    pub price: Option<Vec<PriceTags>>,
}

impl Config {
    pub fn mensa_id(&self) -> Result<usize> {
        // Get the default mensa id from the config file
        let default = self
            .file
            .as_ref()
            .map(|conf| conf.default_mensa_id)
            .flatten();
        self.args.mensa_id.or(default).ok_or(Error::MensaIdMissing)
    }

    pub fn date(&self) -> &chrono::NaiveDate {
        &self.args.date
    }

    pub fn price_tags(&self) -> HashSet<PriceTags> {
        let from_file = || self.file.as_ref().map(|conf| conf.price_tags.clone());
        let from_args = self
            .args
            .price
            .clone()
            .map(|prices| prices.into_iter().collect());
        from_args.or_else(from_file).unwrap_or_default()
    }

    pub fn filter(&self) -> Filter {
        let configuration_filter = self
            .file
            .as_ref()
            .map(|file| &file.filter)
            .cloned()
            .unwrap_or_default();
        let args_filter = Filter {
            tag: TagFilter {
                allow: self.args.allow_tag.clone(),
                deny: self.args.deny_tag.clone(),
            },
            category: CategoryFilter::from_arg_parts(
                &self.args.allow_category,
                &self.args.deny_category,
            ),
        };
        if self.args.overwrite_filter {
            args_filter
        } else {
            configuration_filter.joined(args_filter)
        }
    }

    fn assemble() -> Result<Self> {
        let args = Args::from_args();
        let path = args
            .config
            .clone()
            .or_else(|| default_config_path().log_warn());
        let file = path.map(ConfigFile::load).transpose().log_warn().flatten();
        let config = pass_info(Config { file, args });
        Ok(config)
    }
}

fn parse_human_date(inp: &str) -> Result<NaiveDate> {
    date_time_parser::DateParser::parse(inp).ok_or(Error::InvalidDateInArgs)
}

fn default_config_path() -> Result<PathBuf> {
    dirs::config_dir()
        .ok_or(Error::NoConfigDir)
        .map(|base| base.join("mensa/config.toml"))
}

impl FromStr for PriceTags {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "student" => Ok(Self::Student),
            "employee" => Ok(Self::Employee),
            "other" => Ok(Self::Other),
            _ => Err("expected `Student`, `Employee`, or `Other`"),
        }
    }
}
