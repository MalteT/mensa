use core::fmt;
use lazy_static::lazy_static;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use regex::RegexSet;
use serde::Deserialize;

use std::collections::HashSet;

lazy_static! {
    static ref ALLERGENE_RE: RegexSet = RegexSet::new(&[
        r"(?i)alkohol",
        r"(?i)antioxidation",
        r"(?i)geschwärzt",
        r"(?i)farbstoff",
        r"(?i)eier",
        r"(?i)geschmacksverstärker",
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
        r"(?i)fleischlos",
    ])
    .unwrap();
}

#[derive(Debug, Deserialize)]
pub struct Meal {
    id: usize,
    name: String,
    notes: Vec<String>,
    prices: Prices,
    category: String,
}

#[derive(Debug, Deserialize)]
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
        use termion::{
            color::{self, *},
            style::{self, *},
        };
        let name = format!("{}{}{}", Bold, self.name, style::Reset);
        let (tags, mut allergenes, others) = self
            .notes
            .iter()
            .cloned()
            .flat_map(|raw| Note::from_str(&raw))
            .fold(
                (HashSet::new(), Vec::new(), HashSet::new()),
                |(mut tags, mut allergenes, mut others), note| {
                    match note {
                        Note::Tag(tag) => {
                            tags.insert(tag);
                        }
                        Note::Allergene(all) => allergenes.push(all),
                        Note::Other(other) => {
                            others.insert(other);
                        }
                    }
                    (tags, allergenes, others)
                },
            );
        allergenes.sort_unstable();
        allergenes.dedup();
        let tag_str = tags
            .iter()
            .fold(String::from(" "), |s, e| s + &format!("{} ", e));
        println!("╭─╴{}{}", name, tag_str);
        self.prices.print_to_terminal();
        for note in &others {
            println!("├╴{}", note);
        }
        let allergene_str = allergenes
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

impl From<String> for Note {
    fn from(note: String) -> Self {
        let raw = note.to_lowercase();
        if raw.contains("vegan") {
            Self::Tag(Tag::Vegan)
        } else if raw.contains("fleischlos") {
            Self::Tag(Tag::Vegetarian)
        } else if raw.contains("eier") {
            Self::Allergene(Allergene::Egg)
        } else if raw.contains("milch") {
            Self::Allergene(Allergene::Milk)
        } else if raw.contains("schwein") {
            Self::Tag(Tag::Pig)
        } else if raw.contains("fisch") {
            Self::Tag(Tag::Fish)
        } else if raw.contains("rind") {
            Self::Tag(Tag::Cow)
        } else if raw.contains("geflügel") {
            Self::Tag(Tag::Poultry)
        } else if raw.contains("soja") {
            Self::Allergene(Allergene::Soy)
        } else if raw.contains("gluten") {
            Self::Allergene(Allergene::Gluten)
        } else if raw.contains("antioxidation") {
            Self::Allergene(Allergene::Antioxidant)
        } else if raw.contains("sulfit") || raw.contains("schwefel") {
            Self::Allergene(Allergene::Sulfite)
        } else if raw.contains("senf") {
            Self::Allergene(Allergene::Mustard)
        } else if raw.contains("farbstoff") {
            Self::Allergene(Allergene::Coloring)
        } else if raw.contains("sellerie") {
            Self::Allergene(Allergene::Sellery)
        } else if raw.contains("konservierung") {
            Self::Allergene(Allergene::Preservative)
        } else {
            Self::Other(note)
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
