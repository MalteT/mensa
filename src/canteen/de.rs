use serde::Deserialize;

use crate::cache::Fetchable;

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
