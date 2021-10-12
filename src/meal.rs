use core::fmt;
use lazy_static::lazy_static;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use regex::RegexSet;
use serde::Deserialize;
use termion::{color, style};
use unicode_width::UnicodeWidthStr;

use std::collections::HashSet;

use crate::{
    config::{PriceTags, CONFIG},
    error::{pass_info, Error, ResultExt},
};

const MIN_TERM_WIDTH: usize = 20;
const NAME_PRE: &str = "╭───╴";
const NAME_CONTINUE_PRE: &str = "┊    ";
const OTHER_NOTE_PRE: &str = "├╴";
const OTHER_NOTE_CONTINUE_PRE: &str = "┊ ";

lazy_static! {
    static ref ALLERGENE_RE: RegexSet = RegexSet::new(&[
        r"(?i)alkohol",
        r"(?i)antioxidation",
        r"(?i)geschwärzt",
        r"(?i)farbstoff",
        r"(?i)eier",
        r"(?i)geschmacksverstärker",
        r"(?i)knoblauch",
        r"(?i)gluten",
        r"(?i)milch",
        r"(?i)senf",
        r"(?i)schalenfrüchte|nüsse",
        r"(?i)phosphat",
        r"(?i)konservierung",
        r"(?i)sellerie",
        r"(?i)sesam",
        r"(?i)soja",
        r"(?i)sulfit|schwefel",
        r"(?i)süßungsmittel",
    ])
    .unwrap();
    static ref TAG_RE: RegexSet = RegexSet::new(&[
        r"(?i)rind",
        r"(?i)fisch",
        r"(?i)schwein",
        r"(?i)geflügel",
        r"(?i)vegan",
        r"(?i)fleischlos|vegetarisch|ohne fleisch",
    ])
    .unwrap();
}

#[derive(Debug, Deserialize)]
#[serde(from = "RawMeal")]
pub struct Meal {
    pub _id: usize,
    pub name: String,
    pub tags: HashSet<Tag>,
    pub allergenes: HashSet<Allergene>,
    pub other_notes: HashSet<String>,
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
    Allergene(Allergene),
    Other(String),
}

#[derive(Debug, Clone, Default)]
struct SplittedNotes {
    tags: HashSet<Tag>,
    allergenes: HashSet<Allergene>,
    others: HashSet<String>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    IntoPrimitive,
    TryFromPrimitive,
    Deserialize,
)]
#[repr(u8)]
pub enum Tag {
    Cow,
    Fish,
    Pig,
    Poultry,
    Vegan,
    Vegetarian,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
pub enum Allergene {
    Alcohol,
    Antioxidant,
    Blackened,
    Coloring,
    Egg,
    FlavorEnhancer,
    Garlic,
    Gluten,
    Milk,
    Mustard,
    Nuts,
    Phosphate,
    Preservative,
    Sellery,
    Sesame,
    Soy,
    Sulfite,
    Sweetener,
}

impl RawMeal {
    fn parse_and_split_notes(&self) -> SplittedNotes {
        self.notes
            .iter()
            .cloned()
            .flat_map(|raw| Note::from_str(&raw))
            .fold(SplittedNotes::default(), |mut sn, note| {
                match note {
                    Note::Tag(tag) => {
                        sn.tags.insert(tag);
                    }
                    Note::Allergene(all) => {
                        sn.allergenes.insert(all);
                    }
                    Note::Other(other) => {
                        sn.others.insert(other);
                    }
                }
                sn
            })
    }
}

impl Meal {
    pub fn print_to_terminal(&self) {
        let (width, _height) = get_sane_terminal_dimensions();
        // Print meal name
        self.print_name_to_terminal(width);
        // Get notes, i.e. allergenes, descriptions, tags
        self.print_category_and_tags();
        self.print_other_notes(width);
        self.print_price_and_allergenes();
    }

    fn print_name_to_terminal(&self, width: usize) {
        let max_name_width = width - NAME_PRE.width();
        let mut name_parts = textwrap::wrap(&self.name, max_name_width).into_iter();
        // There will always be a first part of the splitted string
        let first_name_part = name_parts.next().unwrap();
        println!(
            "{}{}{}{}",
            NAME_PRE,
            style::Bold,
            first_name_part,
            style::Reset
        );
        for name_part in name_parts {
            println!(
                "{}{}{}{}",
                NAME_CONTINUE_PRE,
                style::Bold,
                name_part,
                style::Reset
            );
        }
    }

    fn print_category_and_tags(&self) {
        let tag_str = self
            .tags
            .iter()
            .fold(String::from(" "), |s, e| s + &format!("{} ", e));
        println!(
            "├─╴{}{}{}{}",
            color::Fg(color::LightBlue),
            self.category,
            color::Fg(color::Reset),
            tag_str
        );
    }

    fn print_other_notes(&self, width: usize) {
        let max_note_width = width - OTHER_NOTE_PRE.width();
        for note in &self.other_notes {
            let mut note_parts = textwrap::wrap(note, max_note_width).into_iter();
            // There will always be a first part in the splitted string
            println!("{}{}", OTHER_NOTE_PRE, note_parts.next().unwrap());
            for part in note_parts {
                println!("{}{}", OTHER_NOTE_CONTINUE_PRE, part);
            }
        }
    }

    fn print_price_and_allergenes(&self) {
        let prices = self.prices.to_terminal_string();
        let mut allergenes: Vec<_> = self.allergenes.clone().into_iter().collect();
        allergenes.sort_unstable();
        let allergene_str = allergenes
            .iter()
            .fold(String::new(), |s, a| s + &format!("{} ", a));
        println!(
            "╰╴{}  {}{}{}",
            prices,
            color::Fg(color::LightBlack),
            if allergene_str.is_empty() {
                format!(
                    "{}{}no allergenes / miscellaneous{}{}",
                    style::Italic,
                    color::Fg(color::LightBlack),
                    color::Fg(color::Reset),
                    style::Reset
                )
            } else {
                allergene_str
            },
            color::Fg(color::Reset),
        );
    }
}

fn get_sane_terminal_dimensions() -> (usize, usize) {
    termion::terminal_size()
        .map(|(w, h)| (w as usize, h as usize))
        .map(|(w, h)| (w.max(MIN_TERM_WIDTH), h))
        .map_err(Error::UnableToGetTerminalSize)
        .log_warn()
        .unwrap_or((80, 80))
}

impl Note {
    fn from_str(raw: &str) -> Vec<Self> {
        let mut not_others: Vec<_> = Allergene::from_str(raw)
            .into_iter()
            .map(Note::Allergene)
            .collect();
        let tags: Vec<_> = Tag::from_str(raw).into_iter().map(Note::Tag).collect();
        not_others.extend(tags);
        if not_others.is_empty() {
            vec![Note::Other(raw.into())]
        } else {
            not_others
        }
    }
}

impl Tag {
    fn from_str(raw: &str) -> Vec<Self> {
        TAG_RE
            .matches(raw)
            .iter()
            .map(|idx| Tag::try_from_primitive(idx as u8).unwrap())
            .collect()
    }
}

impl Allergene {
    fn from_str(raw: &str) -> Vec<Self> {
        ALLERGENE_RE
            .matches(raw)
            .iter()
            .map(|idx| Allergene::try_from_primitive(idx as u8).unwrap())
            .collect()
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

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tag(tag) => write!(f, "{}", tag),
            Self::Allergene(allergene) => write!(f, "{}", allergene),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vegan => write!(f, "🌱"),
            Self::Vegetarian => write!(f, "🧀"),
            Self::Pig => write!(f, "🐖"),
            Self::Fish => write!(f, "🐟"),
            Self::Cow => write!(f, "🐄"),
            Self::Poultry => write!(f, "🐓"),
        }
    }
}

impl fmt::Display for Allergene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number: u8 = (*self).into();
        write!(f, "{}", number)
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
            allergenes: splitted_notes.allergenes,
            other_notes: splitted_notes.others,
        }
    }
}
