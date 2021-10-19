use chrono::Duration;
use core::fmt;
use itertools::Itertools;
use lazy_static::lazy_static;
use owo_colors::{OwoColorize, Stream};
use serde::Deserialize;
use unicode_width::UnicodeWidthStr;

use std::collections::HashSet;

use crate::{
    cache::fetch_json,
    config::{args::MealsCommand, MealsState, PriceTags},
    error::{pass_info, Result},
    get_sane_terminal_dimensions, State, ENDPOINT,
};

pub mod tag;

use self::tag::Tag;

const NAME_PRE: &str = " ╭───╴";
const NAME_PRE_PLAIN: &str = " - ";
const NAME_CONTINUE_PRE: &str = " ┊    ";
const NAME_CONTINUE_PRE_PLAIN: &str = "     ";
const OTHER_NOTE_PRE: &str = " ├╴";
const OTHER_NOTE_PRE_PLAIN: &str = "   ";
const OTHER_NOTE_CONTINUE_PRE: &str = " ┊ ";
const OTHER_NOTE_CONTINUE_PRE_PLAIN: &str = "     ";
const CATEGORY_PRE: &str = " ├─╴";
const CATEGORY_PRE_PLAIN: &str = "   ";
const PRICES_PRE: &str = " ╰╴";
const PRICES_PRE_PLAIN: &str = "   ";

lazy_static! {
    static ref TTL_MEALS: Duration = Duration::hours(1);
}

#[derive(Debug, Deserialize)]
#[serde(from = "RawMeal")]
pub struct Meal {
    pub _id: usize,
    pub name: String,
    pub tags: HashSet<Tag>,
    pub descs: HashSet<String>,
    pub prices: Prices,
    pub category: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(debug, serde(deny_unknown_fields))]
struct RawMeal {
    id: usize,
    name: String,
    notes: Vec<String>,
    prices: Prices,
    category: String,
}

#[derive(Debug, Deserialize)]
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

impl RawMeal {
    /// Parse notes and return them split into [`Tag`]s and descriptions.
    fn parse_and_split_notes(&self) -> (HashSet<Tag>, HashSet<String>) {
        self.notes
            .iter()
            .cloned()
            .flat_map(|raw| Note::parse_str(&raw))
            .fold(
                (HashSet::new(), HashSet::new()),
                |(mut tags, mut descs), note| {
                    match note {
                        Note::Tag(tag) => {
                            tags.insert(tag);
                        }
                        Note::Desc(other) => {
                            descs.insert(other);
                        }
                    }
                    (tags, descs)
                },
            )
    }
}

impl Meal {
    /// Print this [`Meal`] to the terminal.
    pub fn print(&self, state: &State<MealsCommand>, highlight: bool) {
        let (width, _height) = get_sane_terminal_dimensions();
        // Print meal name
        self.print_name_to_terminal(state, width, highlight);
        // Get notes, i.e. allergenes, descriptions, tags
        self.print_category_and_primary_tags(state, highlight);
        self.print_descriptions(state, width, highlight);
        self.print_price_and_secondary_tags(state, highlight);
    }

    /// Fetch the meals.
    ///
    /// This will respect passed cli arguments and the configuration.
    pub fn fetch(state: &MealsState) -> Result<Vec<Self>> {
        let url = format!(
            "{}/canteens/{}/days/{}/meals",
            ENDPOINT,
            state.canteen_id()?,
            state.date()
        );
        fetch_json(&state.client, url, *TTL_MEALS)
    }

    /// Print the given meals.
    ///
    /// Thi will respect passed cli arguments and the configuration.
    pub fn print_all(state: &MealsState, meals: &[Self]) {
        // Load the filter which is used to select which meals to print.
        let filter = state.get_filter();
        // Load the favourites which will be used for marking meals.
        let favs = state.get_favs_rule();
        for meal in meals {
            if filter.is_match(meal) {
                let is_fav = favs.is_match(meal);
                println!();
                meal.print(state, is_fav);
            }
        }
    }

    fn print_name_to_terminal(&self, state: &MealsState, width: usize, highlight: bool) {
        let max_name_width = width - NAME_PRE.width();
        let mut name_parts = textwrap::wrap(&self.name, max_name_width).into_iter();
        // There will always be a first part of the splitted string
        let first_name_part = name_parts.next().unwrap();
        let pre = if_plain!(state: NAME_PRE, NAME_PRE_PLAIN);
        println!(
            "{}{}",
            hl_if(state, highlight, pre),
            color!(state: hl_if(state, highlight, first_name_part); bold),
        );
        for name_part in name_parts {
            let name_part = hl_if(state, highlight, name_part);
            let pre = if_plain!(state: NAME_CONTINUE_PRE, NAME_CONTINUE_PRE_PLAIN);
            println!(
                "{}{}",
                hl_if(state, highlight, pre),
                color!(state: name_part; bold),
            );
        }
    }

    fn print_category_and_primary_tags(&self, state: &MealsState, highlight: bool) {
        let mut tag_str = self
            .tags
            .iter()
            .filter(|tag| tag.is_primary())
            .map(|tag| tag.as_id(state));
        let tag_str_colored = if_plain!(
            state: color!(state: tag_str.join(" "); bright_black),
            tag_str.join(", ")
        );
        let pre = if_plain!(state: CATEGORY_PRE, CATEGORY_PRE_PLAIN);
        let comma_if_plain = if_plain!(state: "", ",");
        println!(
            "{}{}{} {}",
            hl_if(state, highlight, pre),
            color!(state: self.category; bright_blue),
            color!(state: comma_if_plain; bright_black),
            tag_str_colored
        );
    }

    fn print_descriptions(&self, state: &MealsState, width: usize, highlight: bool) {
        let pre = if_plain!(state: OTHER_NOTE_PRE, OTHER_NOTE_PRE_PLAIN);
        let pre_continue = if_plain!(
            state: OTHER_NOTE_CONTINUE_PRE,
            OTHER_NOTE_CONTINUE_PRE_PLAIN
        );
        let max_note_width = width - OTHER_NOTE_PRE.width();
        for note in &self.descs {
            let mut note_parts = textwrap::wrap(note, max_note_width).into_iter();
            // There will always be a first part in the splitted string
            println!(
                "{}{}",
                hl_if(state, highlight, pre),
                note_parts.next().unwrap()
            );
            for part in note_parts {
                println!("{}{}", hl_if(state, highlight, pre_continue), part);
            }
        }
    }

    fn print_price_and_secondary_tags(&self, state: &State<MealsCommand>, highlight: bool) {
        let prices = self.prices.to_terminal_string(state);
        let mut secondary: Vec<_> = self.tags.iter().filter(|tag| tag.is_secondary()).collect();
        secondary.sort_unstable();
        let secondary_str = secondary.iter().map(|tag| tag.as_id(state)).join(" ");
        let pre = if_plain!(state: PRICES_PRE, PRICES_PRE_PLAIN);
        println!(
            "{}{}  {}",
            hl_if(state, highlight, pre),
            prices,
            color!(state: secondary_str; bright_black),
        );
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
    fn to_terminal_string(&self, state: &State<MealsCommand>) -> String {
        let price_tags = state.price_tags();
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
            .map(|tag| color!(state: tag; bright_green))
            .collect();
        match price_tags.len() {
            0 => String::new(),
            _ => {
                let slash = color!(state: " / "; bright_black);
                format!(
                    "{} {} {}",
                    color!(state: "("; bright_black),
                    price_tags.join(&slash),
                    color!(state: ")"; bright_black),
                )
            }
        }
    }
}

fn hl_if<Cmd, S>(state: &State<Cmd>, highlight: bool, text: S) -> String
where
    S: fmt::Display,
{
    if highlight {
        color!(state: text; bright_yellow)
    } else {
        format!("{}", text)
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

impl From<RawMeal> for Meal {
    fn from(raw: RawMeal) -> Self {
        let (tags, descs) = pass_info(&raw).parse_and_split_notes();
        Self {
            _id: raw.id,
            name: raw.name,
            prices: raw.prices,
            category: raw.category,
            tags,
            descs,
        }
    }
}
