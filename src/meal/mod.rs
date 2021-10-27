use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

mod complete;
mod de;

use crate::{
    cache::Fetchable,
    canteen::{Canteen, CanteenId},
    config::{PriceTags, CONF},
    error::Result,
    print_json,
    tag::Tag,
};

pub use self::complete::MealComplete;

pub type MealId = usize;

lazy_static! {
    static ref PRE: String = color!(if_plain!(" ┊", " |"); bright_black);
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "de::Meal")]
pub struct Meal {
    pub id: MealId,
    pub meta: Fetchable<Meta>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub tags: HashSet<Tag>,
    pub descs: HashSet<String>,
    pub prices: Prices,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug, serde(deny_unknown_fields))]
pub struct Prices {
    pub students: Option<f32>,
    pub employees: Option<f32>,
    pub pupils: Option<f32>,
    pub others: Option<f32>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Note {
    Tag(Tag),
    Desc(String),
}

impl Meta {
    fn fetch(_id: MealId) -> Result<Meta> {
        todo!()
    }
}

impl Meal {
    pub fn meta(&mut self) -> Result<&Meta> {
        self.meta.fetch(|| Meta::fetch(self.id))
    }

    pub fn complete(&mut self) -> Result<MealComplete<'_>> {
        Ok(MealComplete {
            id: self.id,
            meta: self.meta()?,
        })
    }

    /// Print the given meals.
    ///
    /// This will respect passed cli arguments and the configuration.
    pub fn print_for_all_canteens(canteens: &mut [Canteen]) -> Result<()> {
        if CONF.args.json {
            Self::print_for_all_canteens_json(canteens)
        } else {
            Self::print_for_all_canteens_no_json(canteens)
        }
    }

    fn print_for_all_canteens_no_json(canteens: &mut [Canteen]) -> Result<()> {
        // Load the filter which is used to select which meals to print.
        let filter = CONF.get_filter_rule();
        // Load the favourites which will be used for marking meals.
        let favs = CONF.get_favourites_rule();
        // The day for which to print meals
        let day = CONF.date();
        for canteen in canteens {
            let name = canteen.name()?;
            try_println!("\n {}", color!(name; bright_black))?;
            match canteen.meals_at_mut(day)? {
                Some(meals) => {
                    let mut printed_at_least_one_meal = false;
                    for meal in meals {
                        let complete = meal.complete()?;
                        if filter.is_match(&complete) {
                            let is_fav = favs.is_non_empty_match(&complete);
                            try_println!("{}", *PRE)?;
                            complete.print(is_fav)?;
                            printed_at_least_one_meal = true;
                        }
                    }
                    if !printed_at_least_one_meal {
                        try_println!("{} {}", *PRE, color!("no matching meals found"; dimmed))?
                    }
                }
                None => try_println!("{} {}", *PRE, color!("closed"; dimmed))?,
            }
        }
        Ok(())
    }

    fn print_for_all_canteens_json(canteens: &mut [Canteen]) -> Result<()> {
        // Load the filter which is used to select which meals to print.
        let filter = CONF.get_filter_rule();
        // The day for which to print meals
        let day = CONF.date();
        // Filter all meals
        let meals: HashMap<CanteenId, Vec<_>> = canteens
            .iter_mut()
            .map(|canteen| {
                let id = canteen.id();
                let meals: Vec<_> = match canteen.meals_at_mut(day)? {
                    Some(meals) => meals
                        .iter_mut()
                        .map(|meal| meal.complete())
                        .filter_ok(|meal| filter.is_match(meal))
                        .try_collect()?,
                    None => vec![],
                };
                Result::Ok((id, meals))
            })
            .try_collect()?;
        print_json(&meals)
    }
}

impl Note {
    fn parse_str(raw: &str) -> Vec<Self> {
        let tags: Vec<_> = Tag::parse_str(raw).into_iter().map(Note::Tag).collect();
        if tags.is_empty() {
            vec![Note::Desc(raw.into())]
        } else {
            tags
        }
    }
}

impl Prices {
    fn to_terminal_string(&self) -> String {
        let price_tags = CONF.price_tags();
        let price_tags = if price_tags.is_empty() {
            // Print all of them
            vec![self.students, self.employees, self.pupils, self.others]
        } else {
            let mut values = vec![];
            if price_tags.contains(&PriceTags::Student) {
                values.push(self.students);
            }
            if price_tags.contains(&PriceTags::Employee) {
                values.push(self.employees);
            }
            if price_tags.contains(&PriceTags::Pupil) {
                values.push(self.pupils);
            }
            if price_tags.contains(&PriceTags::Other) {
                values.push(self.others);
            }
            values
        };
        let price_tags: Vec<_> = price_tags
            .into_iter()
            .map(|tag| match tag {
                Some(tag) => color!(format!("{:.2}€", tag); bright_green),
                None => color!(String::from("-.--€"); bright_black),
            })
            .map(|tag| color!(tag; bright_green))
            .collect();
        match price_tags.len() {
            0 => String::new(),
            _ => {
                let slash = color!(" / "; bright_black);
                format!(
                    "{} {} {}",
                    color!("("; bright_black),
                    price_tags.join(&slash),
                    color!(")"; bright_black),
                )
            }
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tag(tag) => write!(f, "{}", tag),
            Self::Desc(s) => write!(f, "{}", s),
        }
    }
}
