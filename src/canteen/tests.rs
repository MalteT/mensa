use chrono::Duration;
use pretty_assertions::assert_eq;

use crate::cache::{Fetchable, API};

use super::*;

#[test]
fn it_parses_a_canteen() {
    let url = "http://invalid.local/canteen/1";
    let value = r#"
            {
                "id": 1,
                "name": "Awesome Canteen",
                "city": "Lummerland",
                "address": "Some place!",
                "coordinates": [
                    52.13,
                    11.64
                ]
            }
        "#;
    API.register(url, value, None, Some(1), None, Some(1));
    let canteen: Canteen = CACHE.fetch_json(url, Duration::zero()).unwrap();
    assert_eq!(
        canteen,
        Canteen {
            id: 1,
            meta: Fetchable::Fetched(Meta {
                name: String::from("Awesome Canteen"),
                city: String::from("Lummerland"),
                address: String::from("Some place!"),
                coordinates: Some([52.13, 11.64]),
            }),
            meals: Fetchable::None,
        }
    );
}
