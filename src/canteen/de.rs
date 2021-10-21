use chrono::NaiveDate;
use serde::Deserialize;

use crate::{cache::Fetchable, error::Error};

use super::{CanteenId, Meta};

/// For deserializing responses from API/canteens.
#[derive(Debug, Deserialize)]
pub struct CanteenDeserialized {
    id: CanteenId,
    name: String,
    city: String,
    address: String,
    coordinates: Option<[f32; 2]>,
}

impl From<CanteenDeserialized> for super::Canteen {
    fn from(raw: CanteenDeserialized) -> Self {
        Self {
            id: raw.id,
            meta: Fetchable::Fetched(Meta {
                name: raw.name,
                city: raw.city,
                address: raw.address,
                coordinates: raw.coordinates,
            }),
            meals: Fetchable::None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DayDeserialized {
    date: String,
    closed: bool,
}

impl TryFrom<DayDeserialized> for super::Day {
    type Error = Error;

    fn try_from(raw: DayDeserialized) -> Result<Self, Self::Error> {
        Ok(Self {
            date: NaiveDate::parse_from_str(&raw.date, "%Y-%m-%d").map_err(Error::InvalidDate)?,
            closed: raw.closed,
        })
    }
}
