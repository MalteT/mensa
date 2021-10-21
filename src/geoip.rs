//! Everything needed to fetch geoip data.

use chrono::Duration;
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{cache::fetch_json, config::CanteensState, error::Result};

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

/// Derive Latitude and Longitude from the [`CanteensState`].
///
/// This will use the cli arguments if given and
/// fetch any missing values from api.geoip.rs.
pub fn fetch(state: &CanteensState) -> Result<(f32, f32)> {
    let geo = &state.cmd.geo;
    Ok(if geo.lat.is_none() || geo.long.is_none() {
        let guessed = fetch_geoip(&state.client)?;
        (
            geo.lat.unwrap_or(guessed.latitude),
            geo.long.unwrap_or(guessed.longitude),
        )
    } else {
        // Cannot panic, due to above if
        (geo.lat.unwrap(), geo.long.unwrap())
    })
}

/// Fetch geoip for current ip.
fn fetch_geoip(client: &Client) -> Result<LatLong> {
    let url = "https://api.geoip.rs";
    fetch_json(client, url, *TTL_GEOIP)
}
