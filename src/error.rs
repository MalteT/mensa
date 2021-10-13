use core::fmt;

use thiserror::Error;
use tracing::{error, info, warn};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("reqwest error: {_0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("could not parse date")]
    InvalidDateInArgs,
    #[error("no default mensa id is defined and `--id` was not given")]
    MensaIdMissing,
    #[error("could not read configuration file: {_0}")]
    ReadingConfig(#[source] std::io::Error),
    #[error("could not deserialize configuration file: {_0}")]
    DeserializingConfig(#[source] toml::de::Error),
    #[error("no configuration directory found. On Linux, try setting $XDG_CONFIG_DIR")]
    NoConfigDir,
    #[error("failed to read terminal size for standard output")]
    UnableToGetTerminalSize(#[source] std::io::Error),
    #[error("failed parsing regexes specified in the configuration: {_0}")]
    ParsingFilterRegex(#[source] regex::Error),
    #[error("failed to read geo ip database at api.geoip.rs")]
    ReadingGeoIP(#[source] reqwest::Error),
    #[error("failed to fetch list of mensas")]
    FetchingMensas(#[source] reqwest::Error),
}

pub trait ResultExt<T> {
    fn log_err(self) -> Option<T>;
    fn log_warn(self) -> Option<T>;
    fn log_panic(self) -> T;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn log_err(self) -> Option<T> {
        match self {
            Ok(inner) => Some(inner),
            Err(why) => {
                error!("{}", why.into());
                None
            }
        }
    }

    fn log_panic(self) -> T {
        match self {
            Ok(inner) => inner,
            Err(why) => {
                error!("{}", why.into());
                panic!();
            }
        }
    }

    fn log_warn(self) -> Option<T> {
        match self {
            Ok(inner) => Some(inner),
            Err(why) => {
                warn!("{}", why.into());
                None
            }
        }
    }
}

/// Debug print the given value using [`info`].
pub fn pass_info<T: fmt::Debug>(t: T) -> T {
    info!("{:#?}", &t);
    t
}
