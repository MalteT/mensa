use reqwest::blocking::Client;
use serde::Deserialize;
use tracing::info;

use crate::{
    cache::get_json, complete_lat_long, config::args::CanteensCommand, error::Result, ENDPOINT,
    TTL_CANTEENS,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Canteen {
    id: usize,
    name: String,
    //city: String,
    address: String,
    //oordinates: [f32; 2],
}

impl Canteen {
    pub fn print_to_terminal(&self) {
        use termion::{color, style};
        println!(
            "{}{}{:>4} {}{}{}\n     {}{}{}",
            style::Bold,
            color::Fg(color::LightYellow),
            self.id,
            color::Fg(color::Reset),
            self.name,
            style::Reset,
            color::Fg(color::LightBlack),
            self.address,
            color::Fg(color::Reset),
        );
    }

    pub fn fetch(client: &Client, cmd: &CanteensCommand) -> Result<Vec<Self>> {
        let (lat, long) = complete_lat_long(client, cmd)?;
        let url = format!(
            "{}/canteens?near[lat]={}&near[lng]={}&near[dist]={}",
            ENDPOINT, lat, long, cmd.radius,
        );
        info!(
            "Fetching canteens for lat: {}, long: {} with radius: {}",
            lat, long, cmd.radius
        );
        get_json(client, url, *TTL_CANTEENS)
    }
}
