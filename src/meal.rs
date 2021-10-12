use core::fmt;
use lazy_static::lazy_static;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use regex::RegexSet;
use serde::Deserialize;
use termion::{color, style};
use unicode_width::UnicodeWidthStr;

use std::collections::HashSet;

use crate::error::{Error, ResultExt};

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
#[cfg_attr(debug, serde(deny_unknown_fields))]
pub struct Meal {
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
    allergenes: Vec<Allergene>,
    others: HashSet<String>,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
enum Tag {
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
enum Allergene {
    Alkohol,
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

impl Note {
    fn is_other(&self) -> bool {
        if let Note::Other(_) = self {
            true
        } else {
            false
        }
    }
}

impl Meal {
    pub fn print_to_terminal(&self) {
        use termion::{color::*, style::*};
        let (width, _height) = get_sane_terminal_dimensions();
        // Print meal name
        self.print_name_to_terminal(width);
        // Get notes, i.e. allergenes, descriptions, tags
        let notes = self.parse_and_split_notes();
        self.print_category_and_tags(&notes.tags);
        self.prices.print_to_terminal();
        self.print_other_notes(width, &notes.others);
        let allergene_str = notes
            .allergenes
            .iter()
            .fold(String::new(), |s, a| s + &format!("{} ", a));
        println!(
            "╰╴{}{}{}",
            Fg(LightBlack),
            if allergene_str.is_empty() {
                format!(
                    "{}{}no allergenes / miscellaneous{}{}",
                    Italic,
                    Fg(LightBlack),
                    Fg(color::Reset),
                    style::Reset
                )
            } else {
                allergene_str
            },
            Fg(color::Reset),
        );
    }

    fn print_name_to_terminal(&self, width: usize) {
        let max_name_width = width.saturating_sub(NAME_PRE.width()).max(MIN_TERM_WIDTH);
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

    fn print_category_and_tags(&self, tags: &HashSet<Tag>) {
        let tag_str = tags
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

    fn print_other_notes(&self, width: usize, others: &HashSet<String>) {
        let max_note_width = width - OTHER_NOTE_PRE.width();
        for note in others {
            let mut note_parts = textwrap::wrap(note, max_note_width).into_iter();
            // There will always be a first part in the splitted string
            println!("{}{}", OTHER_NOTE_PRE, note_parts.next().unwrap());
            for part in note_parts {
                println!("{}{}", OTHER_NOTE_CONTINUE_PRE, part);
            }
        }
    }

    fn parse_and_split_notes(&self) -> SplittedNotes {
        let mut splitted_notes = self
            .notes
            .iter()
            .cloned()
            .flat_map(|raw| Note::from_str(&raw))
            .fold(SplittedNotes::default(), |mut sn, note| {
                match note {
                    Note::Tag(tag) => {
                        sn.tags.insert(tag);
                    }
                    Note::Allergene(all) => sn.allergenes.push(all),
                    Note::Other(other) => {
                        sn.others.insert(other);
                    }
                }
                sn
            });
        splitted_notes.allergenes.sort_unstable();
        splitted_notes.allergenes.dedup();
        splitted_notes
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

    fn print_legend(notes: Vec<Self>) {
        todo!()
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
    fn print_to_terminal(&self) {
        use termion::{
            color::{self, *},
            style::{self, *},
        };
        let name_style = format!("{}", Fg(LightBlack));
        let price_style = format!("{}", Bold);
        let reset = format!("{}{}", style::Reset, Fg(color::Reset));
        println!(
            "│   {}Students {}{:>5.2}€{}",
            &name_style, &price_style, self.students, &reset
        );
        println!(
            "│  {}Employees {}{:>5.2}€{}",
            &name_style, &price_style, self.employees, &reset
        );
        println!(
            "│     {}Others {}{:>5.2}€{}",
            &name_style, &price_style, self.others, &reset
        );
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
