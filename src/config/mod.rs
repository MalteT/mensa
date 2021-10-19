use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde::Deserialize;
use structopt::clap::arg_enum;

use std::{collections::HashSet, fs, path::Path, time::Duration as StdDuration};

use crate::{
    error::{Error, Result, ResultExt},
    DIR,
};

use self::{
    args::{Args, CanteensCommand, MealsCommand},
    rule::{RegexRule, Rule, TagRule},
};

pub mod args;
pub mod rule;

lazy_static! {
    static ref REQUEST_TIMEOUT: StdDuration = StdDuration::from_secs(10);
}

#[derive(Debug, Deserialize)]
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

pub type CanteensState<'s> = State<'s, CanteensCommand>;
pub type MealsState<'s> = State<'s, MealsCommand>;

#[derive(Debug)]
pub struct State<'s, Cmd> {
    pub config: Option<ConfigFile>,
    pub client: Client,
    pub args: &'s Args,
    pub cmd: &'s Cmd,
}

impl<'s> State<'s, ()> {
    pub fn assemble(args: &'s Args) -> Result<Self> {
        let default_config_path = || DIR.config_dir().join("config.toml");
        let path = args.config.clone().unwrap_or_else(default_config_path);
        let config = ConfigFile::load_or_log(path);
        let client = Client::builder()
            .timeout(*REQUEST_TIMEOUT)
            .build()
            .map_err(Error::Reqwest)?;
        Ok(Self {
            config,
            client,
            args,
            cmd: &(),
        })
    }
}

impl<'s, Cmd> State<'s, Cmd> {
    pub fn from<OldCmd>(old: State<'s, OldCmd>, cmd: &'s Cmd) -> Self {
        Self {
            config: old.config,
            client: old.client,
            args: old.args,
            cmd,
        }
    }
}

impl MealsState<'_> {
    pub fn canteen_id(&self) -> Result<usize> {
        // Get the default canteen id from the config file
        let default = || self.config.as_ref()?.default_canteen_id;
        self.cmd
            .canteen_id
            .or_else(default)
            .ok_or(Error::CanteenIdMissing)
    }

    pub fn date(&self) -> &chrono::NaiveDate {
        &self.cmd.date
    }

    pub fn price_tags(&self) -> HashSet<PriceTags> {
        let from_file = || Some(self.config.as_ref()?.price_tags.clone());
        match self.cmd.price.clone() {
            Some(prices) => prices.into_iter().collect(),
            None => from_file().unwrap_or_default(),
        }
    }

    pub fn get_filter(&self) -> Rule {
        let conf_filter = || Some(self.config.as_ref()?.filter.clone());
        let args_filter = Rule {
            name: RegexRule::from_arg_parts(&self.cmd.filter_name, &self.cmd.no_filter_name),
            tag: TagRule {
                add: self.cmd.filter_tag.clone(),
                sub: self.cmd.no_filter_tag.clone(),
            },
            category: RegexRule::from_arg_parts(&self.cmd.filter_cat, &self.cmd.no_filter_cat),
        };
        if self.cmd.overwrite_filter {
            args_filter
        } else {
            conf_filter().unwrap_or_default().joined(args_filter)
        }
    }

    pub fn get_favs_rule(&self) -> Rule {
        let conf_favs = || Some(self.config.as_ref()?.favs.clone());
        let args_favs = Rule {
            name: RegexRule::from_arg_parts(&self.cmd.favs_name, &self.cmd.no_favs_name),
            tag: TagRule {
                add: self.cmd.favs_tag.clone(),
                sub: self.cmd.no_favs_tag.clone(),
            },
            category: RegexRule::from_arg_parts(&self.cmd.favs_cat, &self.cmd.no_favs_cat),
        };
        if self.cmd.overwrite_favs {
            args_favs
        } else {
            conf_favs().unwrap_or_default().joined(args_favs)
        }
    }
}
