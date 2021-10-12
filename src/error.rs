use thiserror::Error;
use tracing::error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("reqwest error: {_0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("could not parse date")]
    InvalidDateInArgs,
}

pub trait ResultExt<T> {
    fn log_err(self) -> Option<T>;
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
}
