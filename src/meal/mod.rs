use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

mod de;
mod ser;

use crate::{
    cache::Fetchable,
    canteen::{Canteen, CanteenId},
    config::{PriceTags, CONF},
    error::Result,
    print_json,
    tag::Tag,
};

pub use self::ser::MealComplete;

pub type MealId = usize;

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "de::Meal")]
pub struct Meal {
    pub id: MealId,
    pub meta: Fetchable<Meta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub tags: HashSet<Tag>,
    pub descs: HashSet<String>,
    pub prices: Prices,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(debug, serde(deny_unknown_fields))]
pub struct Prices {
    students: f32,
    employees: f32,
    others: f32,
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
        // Load the filter which is used to select which meals to print.
        let filter = CONF.get_filter_rule();
        // Load the favourites which will be used for marking meals.
        let favs = CONF.get_favourites_rule();
        // The day for which to print meals
        let day = CONF.date();
        // Filter all meals
        let meals = canteens.iter_mut().map(|canteen| {
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
        });
        if CONF.args.json {
            let map: HashMap<CanteenId, Vec<_>> = meals.try_collect()?;
            print_json(&map)
        } else {
            for res in meals {
                let (canteen_id, meals) = res?;
                println!("{}", canteen_id);
                for meal in meals {
                    let is_fav = favs.is_match(&meal);
                    println!();
                    meal.print(is_fav);
                }
            }
            Ok(())
        }
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
            vec![self.students, self.employees, self.others]
        } else {
            let mut values = vec![];
            if price_tags.contains(&PriceTags::Student) {
                values.push(self.students);
            }
            if price_tags.contains(&PriceTags::Employee) {
                values.push(self.employees);
            }
            if price_tags.contains(&PriceTags::Other) {
                values.push(self.others);
            }
            values
        };
        let price_tags: Vec<_> = price_tags
            .into_iter()
            .map(|tag| format!("{:.2}€", tag))
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
