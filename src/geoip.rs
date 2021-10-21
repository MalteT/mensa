//! Everything needed to fetch geoip data.

use chrono::Duration;
use lazy_static::lazy_static;
use serde::Deserialize;

use crate::{
    cache::fetch_json,
    config::{
        args::{CloseCommand, Command},
        CONF,
    },
    error::Result,
};

lazy_static! {
    static ref TTL_GEOIP: Duration = Duration::minutes(5);
}

/// Latitude and Longitude
///
/// This is only used to easily parse the json returned
/// by the api.geoip.rs endpoint.
#[derive(Debug, Clone, Deserialize)]
struct LatLong {
    latitude: f32,
    longitude: f32,
}

/// Infer Latitude and Longitude from the config.
///
/// This will use the cli arguments if given and
/// fetch any missing values from api.geoip.rs.
pub fn infer() -> Result<(f32, f32)> {
    let (lat, long) = match CONF.cmd() {
        Command::Canteens(cmd) => (cmd.geo.lat, cmd.geo.long),
        Command::Meals(cmd) => match &cmd.close {
            Some(CloseCommand::Close(geo)) => (geo.lat, geo.long),
            None => (None, None),
        },
        Command::Tags => (None, None),
    };
    let (lat, long) = if lat.is_none() || long.is_none() {
        let guessed = fetch_geoip()?;
        (
            lat.unwrap_or(guessed.latitude),
            long.unwrap_or(guessed.longitude),
        )
    } else {
        // Cannot panic, due to above if
        (lat.unwrap(), long.unwrap())
    };
    Ok((lat, long))
}

/// Fetch geoip for current ip.
fn fetch_geoip() -> Result<LatLong> {
    let url = "https://api.geoip.rs";
    fetch_json(url, *TTL_GEOIP)
}
