use core::fmt;

use thiserror::Error;
use tracing::{error, info, warn};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("reqwest error: {_0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("serialization failed while {_1}: {_0}")]
    Serializing(#[source] serde_json::Error, &'static str),
    #[error("deserialization failed while {_1}: {_0}")]
    Deserializing(#[source] serde_json::Error, &'static str),
    #[error("cache error while {_1}: {_0}")]
    Cache(#[source] cacache::Error, &'static str),
    #[error("io error while {_1}: {_0}")]
    Io(#[source] std::io::Error, &'static str),
    #[error("could not parse date")]
    InvalidDateInArgs,
    #[error("no default canteen id is defined and `--id` was not given")]
    CanteenIdMissing,
    #[error("could not read configuration file: {_0}")]
    ReadingConfig(#[source] std::io::Error),
    #[error("could not deserialize configuration file: {_0}")]
    DeserializingConfig(#[source] toml::de::Error),
    #[error("failed to read terminal size for standard output")]
    UnableToGetTerminalSize,
    #[error("failed parsing regexes specified in the configuration: {_0}")]
    ParsingFilterRegex(#[source] regex::Error),
    #[error("Url {_0:?} returned status {_1}")]
    NonSuccessStatusCode(String, reqwest::StatusCode),
    #[error("read invalid utf8 bytes")]
    DecodingUtf8(#[source] std::string::FromUtf8Error),
    #[error("invalid date encountered: {_0}")]
    InvalidDate(#[source] chrono::ParseError),
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
                let why = why.into();
                error!("{}", why);
                None
            }
        }
    }

    fn log_panic(self) -> T {
        match self {
            Ok(inner) => inner,
            Err(why) => {
                let why = why.into();
                error!("{}", why);
                panic!();
            }
        }
    }

    fn log_warn(self) -> Option<T> {
        match self {
            Ok(inner) => Some(inner),
            Err(why) => {
                let why = why.into();
                warn!("{}", why);
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
