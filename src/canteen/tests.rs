use std::collections::HashSet;

use chrono::Duration;
use pretty_assertions::assert_eq;

use crate::{
    cache::{Fetchable, API},
    meal::{self, Prices},
    tag::Tag,
};

use super::*;

macro_rules! uniq_id {
    () => {{
        use std::hash::{Hash, Hasher};
        struct SomeStruct;
        let id = ::std::any::TypeId::of::<SomeStruct>();
        let mut hasher = ::std::collections::hash_map::DefaultHasher::new();
        id.hash(&mut hasher);
        let hash: u64 = hasher.finish() % (usize::MAX as u64);
        hash as usize
    }};
}

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
    API.register_single(url, value, None);
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

#[test]
fn it_parses_a_list_of_canteens() {
    let url = "http://invalid.local/canteen/list";
    let value = r#"
        [
            {
                "id": 0,
                "name": "0",
                "city": "Leipzig",
                "address": "Some address",
                "coordinates": null
            },
            {
                "id": 10,
                "name": "Another canteen",
                "city": "Lummerland",
                "address": "Some place!",
                "coordinates": [
                    52.13,
                    11.64
                ]
            }
        ]
    "#;
    API.register_single(url, value, None);
    let canteens: Vec<Canteen> = CACHE.fetch_json(url, Duration::zero()).unwrap();
    assert_eq!(
        canteens,
        &[
            Canteen {
                id: 0,
                meta: Fetchable::Fetched(Meta {
                    name: String::from("0"),
                    city: String::from("Leipzig"),
                    address: String::from("Some address"),
                    coordinates: None,
                }),
                meals: Fetchable::None,
            },
            Canteen {
                id: 10,
                meta: Fetchable::Fetched(Meta {
                    name: String::from("Another canteen"),
                    city: String::from("Lummerland"),
                    address: String::from("Some place!"),
                    coordinates: Some([52.13, 11.64]),
                }),
                meals: Fetchable::None,
            }
        ]
    );
}

#[test]
fn it_parses_multipage_canteen_lists() {
    let first_url = "http://invalid.local/canteen/multipage";
    let map = &[
        (
            first_url,
            r#"
            [
                {
                    "id": 0,
                    "name": "First",
                    "city": "Leipzig",
                    "address": "",
                    "coordinates": null
                }
            ]
        "#,
            None,
        ),
        (
            "http://invalid.local/canteen/multipage/2",
            r#"
            [
                {
                    "id": 1,
                    "name": "Second",
                    "city": "Hannover",
                    "address": "address",
                    "coordinates": [
                        1.1, 2.2
                    ]
                }
            ]
        "#,
            None,
        ),
        (
            "http://invalid.local/canteen/multipage/3",
            r#"
            [
                {
                    "id": 2,
                    "name": "Third",
                    "city": "London",
                    "address": "Some place cool",
                    "coordinates": null
                }
            ]
        "#,
            None,
        ),
    ];
    API.register_pages(map);
    let canteens: Vec<Canteen> = PaginatedList::new(first_url, Duration::zero())
        .consume()
        .unwrap();
    assert_eq!(
        canteens,
        &[
            Canteen {
                id: 0,
                meta: Fetchable::Fetched(Meta {
                    name: String::from("First"),
                    city: String::from("Leipzig"),
                    address: String::from(""),
                    coordinates: None,
                }),
                meals: Fetchable::None,
            },
            Canteen {
                id: 1,
                meta: Fetchable::Fetched(Meta {
                    name: String::from("Second"),
                    city: String::from("Hannover"),
                    address: String::from("address"),
                    coordinates: Some([1.1, 2.2]),
                }),
                meals: Fetchable::None,
            },
            Canteen {
                id: 2,
                meta: Fetchable::Fetched(Meta {
                    name: String::from("Third"),
                    city: String::from("London"),
                    address: String::from("Some place cool"),
                    coordinates: None,
                }),
                meals: Fetchable::None,
            }
        ]
    )
}

#[test]
fn it_parses_empty_lists() {
    let url = "http://invalid.local/canteen/empty";
    API.register_single(url, "[]", None);
    let canteens: Vec<Canteen> = CACHE.fetch_json(url, Duration::zero()).unwrap();
    assert_eq!(canteens, &[]);
}

#[test]
fn it_errors_on_empty_body() {
    let url = "http://invalid.local/canteen/null";
    API.register_single(url, "", None);
    let res: Result<Vec<Canteen>> = CACHE.fetch_json(url, Duration::zero());
    assert!(res.is_err());
}

#[test]
fn it_fetches_metadata() {
    let id = uniq_id!();
    let url = format!("{}/canteens/{}", OPEN_MENSA_API, id);
    let value = format!(
        r#"{{
        "id": {},
        "name": "Fetchable",
        "city": "Leer",
        "address": "Whooper",
        "coordinates": null
    }}"#,
        id
    );
    API.register_single(&url, &value, None);
    let mut canteen = Canteen::from(id);
    // Trigger fetch
    canteen.meta().unwrap();
    assert_eq!(
        canteen,
        Canteen {
            id,
            meta: Fetchable::Fetched(Meta {
                name: String::from("Fetchable"),
                city: String::from("Leer"),
                address: String::from("Whooper"),
                coordinates: None,
            }),
            meals: Fetchable::None,
        }
    );
}

#[test]
fn it_fetches_meals() {
    let id = uniq_id!();
    let date = chrono::NaiveDate::from_ymd(2021, 10, 27);

    // Add dates to canteen
    let url = format!("{}/canteens/{}/days", OPEN_MENSA_API, id);
    let value = r#"[
        {
            "date": "2021-10-27",
            "closed": false
        },
        {
            "date": "2021-10-28",
            "closed": false
        }
    ]"#;
    API.register_single(&url, value, None);

    // Add meals to one date
    let url = format!("{}/canteens/{}/days/2021-10-27/meals", OPEN_MENSA_API, id);
    let value = r#"[
        {
            "id": 8442313,
            "name": "Schweinebraten mit Rotkohl und Kartoffelklößen",
            "category": "Hauptgerichte",
            "prices": {
                "students": 3.1,
                "employees": 4.8,
                "pupils": null,
                "others": 6.2
            },
            "notes": [
                "Schwein"
            ]
        }
    ]"#;
    API.register_single(&url, value, None);

    let mut canteen = Canteen::from(id);
    // Empty at first
    assert_eq!(
        canteen,
        Canteen {
            id,
            meta: Fetchable::None,
            meals: Fetchable::None,
        }
    );
    // Trigger fetch
    canteen.meals_at_mut(&date).unwrap();
    assert_eq!(
        canteen,
        Canteen {
            id,
            meta: Fetchable::None,
            meals: Fetchable::Fetched(
                vec![
                    (
                        date,
                        Fetchable::Fetched(vec![Meal {
                            id: 8442313,
                            meta: Fetchable::Fetched(meal::Meta {
                                name: String::from(
                                    "Schweinebraten mit Rotkohl und Kartoffelklößen"
                                ),
                                tags: vec![Tag::Pig].into_iter().collect(),
                                descs: HashSet::new(),
                                category: String::from("Hauptgerichte"),
                                prices: Prices {
                                    students: Some(3.1),
                                    employees: Some(4.8),
                                    pupils: None,
                                    others: Some(6.2),
                                },
                            })
                        }])
                    ),
                    (NaiveDate::from_ymd(2021, 10, 28), Fetchable::None),
                ]
                .into_iter()
                .collect()
            ),
        }
    );
}
