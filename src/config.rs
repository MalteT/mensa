use chrono::NaiveDate;
use lazy_static::lazy_static;
use serde::Deserialize;
use structopt::StructOpt;

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{pass_info, Error, Result, ResultExt};

lazy_static! {
    pub static ref CONFIG: Config = Config::assemble().log_panic();
}

#[derive(Debug)]
pub struct Config {
    args: Args,
    file: Option<ConfigFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
struct ConfigFile {
    default_mensa_id: Option<usize>,
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
