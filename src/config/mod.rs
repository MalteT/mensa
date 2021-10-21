use chrono::NaiveDate;
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde::Deserialize;
use structopt::{clap::arg_enum, StructOpt};

use std::{collections::HashSet, fs, path::Path, time::Duration as StdDuration};

use crate::{
    canteen::CanteenId,
    config::args::{parse_human_date, Command},
    error::{Error, Result, ResultExt},
    DIR,
};

use self::{
    args::{Args, MealsCommand},
    rule::{RegexRule, Rule, TagRule},
};

pub mod args;
pub mod rule;

lazy_static! {
    pub static ref CONF: Config = Config::assemble().unwrap();
    static ref REQUEST_TIMEOUT: StdDuration = StdDuration::from_secs(10);
}

#[derive(Debug)]
pub struct Config {
    pub config: Option<ConfigFile>,
    pub client: Client,
    pub args: Args,
}

impl Config {
    fn assemble() -> Result<Self> {
        let args = Args::from_args();
        let default_config_path = || DIR.config_dir().join("config.toml");
        let path = args.config.clone().unwrap_or_else(default_config_path);
        let config = ConfigFile::load_or_log(path);
        let client = Client::builder()
            .timeout(*REQUEST_TIMEOUT)
            .build()
            .map_err(Error::Reqwest)?;
        Ok(Config {
            config,
            client,
            args,
        })
    }

    /// Easy reference to the Command
    pub fn cmd(&self) -> &Command {
        lazy_static! {
            static ref DEFAULT: Command = Command::Meals(MealsCommand::default());
        }
        match self.args.command {
            Some(ref cmd) => cmd,
            None => &*DEFAULT,
        }
    }

    pub fn canteen_id(&self) -> Result<CanteenId> {
        // Get the default canteen id from the config file
        let default = || self.config.as_ref()?.default_canteen_id;
        let id = match self.cmd() {
            Command::Meals(cmd) => cmd.canteen_id,
            _ => None,
        };
        id.or_else(default).ok_or(Error::CanteenIdMissing)
    }

    pub fn date(&self) -> &NaiveDate {
        lazy_static! {
            static ref DEFAULT: NaiveDate = parse_human_date("today").unwrap();
        }
        match self.cmd() {
            Command::Meals(cmd) => &cmd.date,
            _ => &*DEFAULT,
        }
    }

    pub fn price_tags(&self) -> HashSet<PriceTags> {
        let from_file = || Some(self.config.as_ref()?.price_tags.clone());
        match self.cmd() {
            Command::Meals(cmd) => match cmd.price.clone() {
                Some(prices) => prices.into_iter().collect(),
                None => from_file().unwrap_or_default(),
            },
            _ => from_file().unwrap_or_default(),
        }
    }

    pub fn get_filter_rule(&self) -> Rule {
        match self.cmd() {
            Command::Meals(cmd) => {
                let conf_filter = || Some(self.config.as_ref()?.filter.clone());
                let args_filter = Rule {
                    name: RegexRule::from_arg_parts(&cmd.filter_name, &cmd.no_filter_name),
                    tag: TagRule {
                        add: cmd.filter_tag.clone(),
                        sub: cmd.no_filter_tag.clone(),
                    },
                    category: RegexRule::from_arg_parts(&cmd.filter_cat, &cmd.no_filter_cat),
                };
                if cmd.overwrite_filter {
                    args_filter
                } else {
                    conf_filter().unwrap_or_default().joined(args_filter)
                }
            }
            _ => {
                unreachable!("Filters should not be relevant here")
            }
        }
    }

    pub fn get_favourites_rule(&self) -> Rule {
        match self.cmd() {
            Command::Meals(cmd) => {
                let conf_favs = || Some(self.config.as_ref()?.favs.clone());
                let args_favs = Rule {
                    name: RegexRule::from_arg_parts(&cmd.favs_name, &cmd.no_favs_name),
                    tag: TagRule {
                        add: cmd.favs_tag.clone(),
                        sub: cmd.no_favs_tag.clone(),
                    },
                    category: RegexRule::from_arg_parts(&cmd.favs_cat, &cmd.no_favs_cat),
                };
                if cmd.overwrite_favs {
                    args_favs
                } else {
                    conf_favs().unwrap_or_default().joined(args_favs)
                }
            }
            _ => unreachable!("Favourite rules should not be relevant here"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct ConfigFile {
    #[serde(default)]
    default_canteen_id: Option<usize>,
    #[serde(default)]
    price_tags: HashSet<PriceTags>,
    #[serde(default)]
    filter: Rule,
    #[serde(default)]
    favs: Rule,
}
arg_enum! {
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize)]
    pub enum PriceTags {
        Student,
        Employee,
        Other,
    }
}

impl ConfigFile {
    pub fn load_or_log<P: AsRef<Path>>(path: P) -> Option<Self> {
        let file = fs::read_to_string(path)
            .map_err(Error::ReadingConfig)
            .log_warn()?;
        toml::from_str(&file)
            .map_err(Error::DeserializingConfig)
            .log_err()
    }
}
