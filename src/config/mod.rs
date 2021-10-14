use lazy_static::lazy_static;
use serde::Deserialize;
use structopt::{clap::arg_enum, StructOpt};

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    error::{pass_info, Error, Result, ResultExt},
    DIR,
};

use self::{
    args::Args,
    rule::{RegexRule, Rule, TagRule},
};

pub mod args;
pub mod rule;

lazy_static! {
    pub static ref CONFIG: Config = Config::assemble().log_panic();
}

#[derive(Debug)]
pub struct Config {
    pub args: Args,
    pub file: Option<ConfigFile>,
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
    fn load_or_log<P: AsRef<Path>>(path: P) -> Option<Self> {
        let file = fs::read_to_string(path)
            .map_err(Error::ReadingConfig)
            .log_warn()?;
        toml::from_str(&file)
            .map_err(Error::DeserializingConfig)
            .log_err()
    }
}

impl Config {
    pub fn canteen_id(&self) -> Result<usize> {
        // Get the default canteen id from the config file
        let default = self
            .file
            .as_ref()
            .map(|conf| conf.default_canteen_id)
            .flatten();
        self.args
            .canteen_id
            .or(default)
            .ok_or(Error::CanteenIdMissing)
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

    pub fn get_filter(&self) -> Rule {
        let conf_filter = self
            .file
            .as_ref()
            .map(|file| &file.filter)
            .cloned()
            .unwrap_or_default();
        let args_filter = Rule {
            name: RegexRule::from_arg_parts(&self.args.filter_name, &self.args.no_filter_name),
            tag: TagRule {
                add: self.args.filter_tag.clone(),
                sub: self.args.no_filter_tag.clone(),
            },
            category: RegexRule::from_arg_parts(&self.args.filter_cat, &self.args.no_filter_cat),
        };
        if self.args.overwrite_filter {
            args_filter
        } else {
            conf_filter.joined(args_filter)
        }
    }

    pub fn get_favs_rule(&self) -> Rule {
        let conf_favs = self
            .file
            .as_ref()
            .map(|file| &file.favs)
            .cloned()
            .unwrap_or_default();
        let args_favs = Rule {
            name: RegexRule::from_arg_parts(&self.args.favs_name, &self.args.no_favs_name),
            tag: TagRule {
                add: self.args.favs_tag.clone(),
                sub: self.args.no_favs_tag.clone(),
            },
            category: RegexRule::from_arg_parts(&self.args.favs_cat, &self.args.no_favs_cat),
        };
        if self.args.overwrite_favs {
            args_favs
        } else {
            conf_favs.joined(args_favs)
        }
    }

    fn assemble() -> Result<Self> {
        let args = Args::from_args();
        let path = args.config.clone().unwrap_or_else(default_config_path);
        let file = ConfigFile::load_or_log(pass_info(path));
        let config = pass_info(Config { file, args });
        Ok(config)
    }
}

fn default_config_path() -> PathBuf {
    DIR.config_dir().join("config.toml")
}
