use std::collections::HashMap;

use chrono::NaiveDate;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tracing::info;

mod de;
mod ser;
#[cfg(test)]
mod tests;

use crate::{
    cache::{Cache, Fetchable, CACHE},
    config::{
        args::{CloseCommand, Command, GeoCommand},
        CONF,
    },
    error::Result,
    geoip, get_sane_terminal_dimensions,
    meal::Meal,
    pagination::PaginatedList,
    print_json, OPEN_MENSA_API, TTL_CANTEENS, TTL_MEALS,
};

use self::ser::CanteenCompleteWithoutMeals;

pub type CanteenId = usize;

const ADRESS_INDENT: &str = "     ";

lazy_static! {
    static ref EMPTY: Vec<Meal> = Vec::new();
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "de::CanteenDeserialized")]
pub struct Canteen {
    id: CanteenId,
    #[serde(flatten)]
    meta: Fetchable<Meta>,
    /// A map from dates to lists of meals.
    ///
    /// The list of dates itself is fetchable as are the lists of meals.
    meals: Fetchable<HashMap<NaiveDate, Fetchable<Vec<Meal>>>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    name: String,
    city: String,
    address: String,
    coordinates: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "de::DayDeserialized")]
pub struct Day {
    date: NaiveDate,
    #[serde(rename = "closed")]
    _closed: bool,
}

impl Meta {
    pub fn fetch(id: CanteenId) -> Result<Self> {
        let url = format!("{}/canteens/{}", OPEN_MENSA_API, id);
        CACHE.fetch_json(url, *TTL_CANTEENS)
    }
}

impl Canteen {
    /// Infer canteens from the config.
    ///
    /// # Command
    /// - Meals:
    ///   - Close: Canteens close to the current location
    ///   - Else: Canteen given by id
    /// - Else: Panic!
    pub fn infer() -> Result<Vec<Self>> {
        match CONF.cmd() {
            Command::Meals(cmd) => match cmd.close {
                Some(CloseCommand::Close(ref geo)) => Self::fetch_for_geo(geo, false),
                None => {
                    let id = CONF.canteen_id()?;
                    Ok(vec![id.into()])
                }
            },
            Command::Canteens(cmd) => Self::fetch_for_geo(&cmd.geo, cmd.all),
            Command::Tags => unreachable!("BUG: This is not relevant here"),
        }
    }

    pub fn print(&mut self) -> Result<()> {
        let (width, _) = get_sane_terminal_dimensions();
        let address = textwrap::fill(
            self.address()?,
            textwrap::Options::new(width)
                .initial_indent(ADRESS_INDENT)
                .subsequent_indent(ADRESS_INDENT),
        );
        println!(
            "{} {}\n{}",
            color!(format!("{:>4}", self.id); bold, bright_yellow),
            color!(self.meta()?.name; bold),
            color!(address; bright_black),
        );
        Ok(())
    }

    pub fn id(&self) -> CanteenId {
        self.id
    }

    pub fn address(&mut self) -> Result<&String> {
        Ok(&self.meta()?.address)
    }

    pub fn name(&mut self) -> Result<&String> {
        Ok(&self.meta()?.name)
    }

    pub fn complete_without_meals(&mut self) -> Result<CanteenCompleteWithoutMeals<'_>> {
        Ok(CanteenCompleteWithoutMeals {
            id: self.id,
            meta: self.meta()?,
        })
    }

    pub fn print_all(canteens: &mut [Self]) -> Result<()> {
        if CONF.args.json {
            Self::print_all_json(canteens)
        } else {
            for canteen in canteens {
                println!();
                canteen.print()?;
            }
            Ok(())
        }
    }

    pub fn meals_at_mut(&mut self, date: &NaiveDate) -> Result<Option<&mut Vec<Meal>>> {
        let id = self.id();
        let dates = self.meals.fetch_mut(|| fetch_dates_for_canteen(self.id))?;
        match dates.get_mut(date) {
            Some(meals) => {
                let meals = meals.fetch_mut(|| fetch_meals(id, date))?;
                Ok(Some(meals))
            }
            None => Ok(None),
        }
    }

    fn print_all_json(canteens: &mut [Self]) -> Result<()> {
        let serializable: Vec<_> = canteens
            .iter_mut()
            .map(|c| c.complete_without_meals())
            .try_collect()?;
        print_json(&serializable)
    }

    fn meta(&mut self) -> Result<&Meta> {
        self.meta.fetch(|| Meta::fetch(self.id))
    }

    fn fetch_for_geo(geo: &GeoCommand, all: bool) -> Result<Vec<Self>> {
        let url = if all {
            info!("Fetching all canteens");
            format!("{}/canteens", OPEN_MENSA_API)
        } else {
            let (lat, long) = geoip::infer()?;
            info!(
                "Fetching canteens for lat: {}, long: {} with radius: {}",
                lat, long, geo.radius
            );
            format!(
                "{}/canteens?near[lat]={}&near[lng]={}&near[dist]={}",
                OPEN_MENSA_API, lat, long, geo.radius,
            )
        };
        PaginatedList::new(url, *TTL_CANTEENS).consume()
    }
}

fn fetch_dates_for_canteen(id: CanteenId) -> Result<HashMap<NaiveDate, Fetchable<Vec<Meal>>>> {
    let url = format!("{}/canteens/{}/days", OPEN_MENSA_API, id,);
    let days: Vec<Day> = PaginatedList::new(url, *TTL_MEALS).consume()?;
    Ok(days
        .into_iter()
        .map(|day| (day.date, Fetchable::None))
        .collect())
}

fn fetch_meals(id: CanteenId, date: &NaiveDate) -> Result<Vec<Meal>> {
    let url = format!("{}/canteens/{}/days/{}/meals", OPEN_MENSA_API, id, date);
    PaginatedList::new(url, *TTL_MEALS).consume()
}

impl From<CanteenId> for Canteen {
    fn from(id: CanteenId) -> Self {
        Self {
            id,
            meta: Fetchable::None,
            meals: Fetchable::None,
        }
    }
}
