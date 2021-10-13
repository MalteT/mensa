use core::fmt;
use serde::Deserialize;
use termion::{color, style};
use unicode_width::UnicodeWidthStr;

use std::collections::HashSet;

use crate::{
    config::{PriceTags, CONFIG},
    error::pass_info,
    get_sane_terminal_dimensions,
};

pub mod tag;

use self::tag::Tag;

const NAME_PRE: &str = "╭───╴";
const NAME_CONTINUE_PRE: &str = "┊    ";
const OTHER_NOTE_PRE: &str = "├╴";
const OTHER_NOTE_CONTINUE_PRE: &str = "┊ ";
const CATEGORY_PRE: &str = "├─╴";
const PRICES_PRE: &str = "╰╴";

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

#[derive(Debug, Clone, Default)]
struct SplittedNotes {
    tags: HashSet<Tag>,
    descs: HashSet<String>,
}

impl RawMeal {
    fn parse_and_split_notes(&self) -> SplittedNotes {
        self.notes
            .iter()
            .cloned()
            .flat_map(|raw| Note::parse_str(&raw))
            .fold(SplittedNotes::default(), |mut sn, note| {
                match note {
                    Note::Tag(tag) => {
                        sn.tags.insert(tag);
                    }
                    Note::Desc(other) => {
                        sn.descs.insert(other);
                    }
                }
                sn
            })
    }
}

impl Meal {
    pub fn print_to_terminal(&self, highlight: bool) {
        let (width, _height) = get_sane_terminal_dimensions();
        // Print meal name
        self.print_name_to_terminal(width, highlight);
        // Get notes, i.e. allergenes, descriptions, tags
        self.print_category_and_primary_tags(highlight);
        self.print_descriptions(width, highlight);
        self.print_price_and_secondary_tags(highlight);
    }

    fn print_name_to_terminal(&self, width: usize, highlight: bool) {
        let max_name_width = width - NAME_PRE.width();
        let mut name_parts = textwrap::wrap(&self.name, max_name_width).into_iter();
        // There will always be a first part of the splitted string
        let first_name_part = name_parts.next().unwrap();
        println!(
            "{}{}{}{}",
            hl_if(highlight, NAME_PRE),
            style::Bold,
            hl_if(highlight, first_name_part),
            style::Reset
        );
        for name_part in name_parts {
            println!(
                "{}{}{}{}",
                hl_if(highlight, NAME_CONTINUE_PRE),
                style::Bold,
                hl_if(highlight, name_part),
                style::Reset
            );
        }
    }

    fn print_category_and_primary_tags(&self, highlight: bool) {
        let tag_str = self
            .tags
            .iter()
            .filter(|tag| tag.is_primary())
            .fold(String::from(" "), |s, e| s + &format!("{} ", e.as_emoji()));
        println!(
            "{}{}{}{}{}",
            hl_if(highlight, CATEGORY_PRE),
            color::Fg(color::LightBlue),
            self.category,
            color::Fg(color::Reset),
            tag_str
        );
    }

    fn print_descriptions(&self, width: usize, highlight: bool) {
        let max_note_width = width - OTHER_NOTE_PRE.width();
        for note in &self.descs {
            let mut note_parts = textwrap::wrap(note, max_note_width).into_iter();
            // There will always be a first part in the splitted string
            println!(
                "{}{}",
                hl_if(highlight, OTHER_NOTE_PRE),
                note_parts.next().unwrap()
            );
            for part in note_parts {
                println!("{}{}", hl_if(highlight, OTHER_NOTE_CONTINUE_PRE), part);
            }
        }
    }

    fn print_price_and_secondary_tags(&self, highlight: bool) {
        let prices = self.prices.to_terminal_string();
        let mut secondary: Vec<_> = self.tags.iter().filter(|tag| tag.is_secondary()).collect();
        secondary.sort_unstable();
        let secondary_str = secondary
            .iter()
            .fold(String::new(), |s, a| s + &format!("{} ", a.as_emoji()));
        println!(
            "{}{}  {}{}{}",
            hl_if(highlight, PRICES_PRE),
            prices,
            color::Fg(color::LightBlack),
            secondary_str,
            color::Fg(color::Reset),
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
    fn to_terminal_string(&self) -> String {
        let price_style = format!("{}", color::Fg(color::LightGreen));
        let reset = format!("{}", color::Fg(color::Reset));
        let price_tags = CONFIG.price_tags();
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
            .map(|tag| format!("{}{:.2}€{}", &price_style, tag, &reset))
            .collect();
        match price_tags.len() {
            0 => String::new(),
            _ => {
                let slash = format!(
                    " {}/{} ",
                    color::Fg(color::LightBlack),
                    color::Fg(color::Reset)
                );
                format!(
                    "{0}({2} {1} {0}){2}",
                    color::Fg(color::LightBlack),
                    price_tags.join(&slash),
                    color::Fg(color::Reset)
                )
            }
        }
    }
}

fn hl_if<S>(highlight: bool, text: S) -> String
where
    S: fmt::Display,
{
    if highlight {
        format!(
            "{}{}{}",
            color::Fg(color::LightYellow),
            text,
            color::Fg(color::Reset)
        )
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
        let splitted_notes = pass_info(&raw).parse_and_split_notes();
        Self {
            _id: raw.id,
            name: raw.name,
            prices: raw.prices,
            category: raw.category,
            tags: splitted_notes.tags,
            descs: splitted_notes.descs,
        }
    }
}
