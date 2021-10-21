use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::info;

mod de;
mod ser;

use crate::{
    cache::Fetchable, config::CanteensState, error::Result, geoip, get_sane_terminal_dimensions,
    meal::Meal, pagination::PaginatedList, print_json, ENDPOINT, TTL_CANTEENS,
};

use self::ser::CanteenCompleteWithoutMeals;

pub type CanteenId = usize;

const ADRESS_INDENT: &str = "     ";

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "de::CanteenDeserialized")]
pub struct Canteen {
    id: CanteenId,
    #[serde(flatten)]
    meta: Fetchable<Meta>,
    meals: Fetchable<Vec<Meal>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    name: String,
    city: String,
    address: String,
    coordinates: Option<[f32; 2]>,
}

impl Meta {
    pub fn fetch(id: CanteenId) -> Result<Self> {
        todo!()
    }
}

impl Canteen {
    pub fn print(&mut self, state: &CanteensState) -> Result<()> {
        let (width, _) = get_sane_terminal_dimensions();
        let address = textwrap::fill(
            self.address()?,
            textwrap::Options::new(width)
                .initial_indent(ADRESS_INDENT)
                .subsequent_indent(ADRESS_INDENT),
        );
        println!(
            "{} {}\n{}",
            color!(state: format!("{:>4}", self.id); bold, bright_yellow),
            color!(state: self.meta()?.name; bold),
            color!(state: address; bright_black),
        );
        Ok(())
    }

    pub fn id(&self) -> CanteenId {
        self.id
    }

    pub fn name(&mut self) -> Result<&String> {
        Ok(&self.meta()?.address)
    }

    pub fn address(&mut self) -> Result<&String> {
        Ok(&self.meta()?.address)
    }

    pub fn complete_without_meals(&mut self) -> Result<CanteenCompleteWithoutMeals<'_>> {
        Ok(CanteenCompleteWithoutMeals {
            id: self.id,
            meta: self.meta()?,
        })
    }

    pub fn fetch(state: &CanteensState) -> Result<Vec<Self>> {
        let url = if state.cmd.all {
            info!("Fetching all canteens");
            format!("{}/canteens", ENDPOINT)
        } else {
            let (lat, long) = geoip::fetch(state)?;
            info!(
                "Fetching canteens for lat: {}, long: {} with radius: {}",
                lat, long, state.cmd.geo.radius
            );
            format!(
                "{}/canteens?near[lat]={}&near[lng]={}&near[dist]={}",
                ENDPOINT, lat, long, state.cmd.geo.radius,
            )
        };
        PaginatedList::from(&state.client, url, *TTL_CANTEENS)?.try_flatten_and_collect()
    }

    pub fn print_all(state: &CanteensState, canteens: &mut [Self]) -> Result<()> {
        if state.args.json {
            Self::print_all_json(canteens)
        } else {
            for canteen in canteens {
                println!();
                canteen.print(state)?;
            }
            Ok(())
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
}
