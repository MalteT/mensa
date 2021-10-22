use std::collections::HashMap;

use lazy_static::lazy_static;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};
use unicode_width::UnicodeWidthStr;

use crate::{config::CONF, error::Result, get_sane_terminal_dimensions, print_json};

const ID_WIDTH: usize = 4;
const TEXT_INDENT: &str = "     ";

lazy_static! {
    /// These must have the same order as the variants in the [`Tag`] enum.
    static ref TAG_RE: RegexSet = RegexSet::new(&[
        r"(?i)säue?rungsmittel",
        r"(?i)alkohol",
        r"(?i)antiox[iy]dation",
        r"(?i)geschwärzt",
        r"(?i)kakaohaltige fettglasur",
        r"(?i)farbstoff",
        r"(?i)rind",
        r"(?i)hühnerei|eier|^ei$|enthält ei\*",
        r"(?i)fisch",
        r"(?i)geschmacksverstärker",
        r"(?i)knoblauch",
        r"(?i)gluten|weizen|hafer|gerste|roggen",
        r"(?i)milch|laktose",
        r"(?i)lupine?",
        r"(?i)senf",
        r"(?i)schalenfrüchte|nüsse|walnuss|pistazie|mandeln",
        r"(?i)phosphat",
        r"(?i)schwein",
        r"(?i)geflügel",
        r"(?i)konservierung|konserviert",
        r"(?i)sellerie",
        r"(?i)sesam",
        r"(?i)soja",
        r"(?i)sulfit|schwefel",
        r"(?i)süßungsmittel",
        r"(?i)vegan",
        r"(?i)fleischlos|vegetarisch|ohne fleisch",
        r"(?i)(ge)?wachs(t)?",
    ])
    .unwrap();
}

/// A tag describing a meal.
///
/// Contains allergy information, descriptions and categories.
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
    Serialize,
    Deserialize,
    EnumIter,
    Display,
)]
#[repr(u8)]
#[remain::sorted]
pub enum Tag {
    Acidifier,
    Alcohol,
    Antioxidant,
    Blackened,
    #[strum(to_string = "Cacao Containing Fat Glaze")]
    CacaoContainingFatGlaze,
    Coloring,
    Cow,
    Egg,
    Fish,
    #[strum(to_string = "Flavor Enhancer")]
    FlavorEnhancer,
    Garlic,
    Gluten,
    Lactose,
    Lupin,
    Mustard,
    Nuts,
    Phosphate,
    Pig,
    Poultry,
    Preservative,
    Sellery,
    Sesame,
    Soy,
    Sulfite,
    Sweetener,
    Vegan,
    Vegetarian,
    Waxed,
}

impl Tag {
    /// Try deriving [`Tag`]s from the `raw` tag.
    pub fn parse_str(raw: &str) -> Vec<Self> {
        TAG_RE
            .matches(raw)
            .iter()
            .map(|idx| Tag::try_from_primitive(idx as u8).unwrap())
            .collect()
    }

    /// Is this a primary tag?
    ///
    /// Primary tags have an associated emoji and are not allergy information.
    pub fn is_primary(&self) -> bool {
        use Tag::*;
        match self {
            Cow | Fish | Pig | Poultry | Vegan | Vegetarian => true,
            Acidifier
            | Alcohol
            | Antioxidant
            | Blackened
            | CacaoContainingFatGlaze
            | Coloring
            | Egg
            | FlavorEnhancer
            | Garlic
            | Gluten
            | Lupin
            | Lactose
            | Mustard
            | Nuts
            | Phosphate
            | Preservative
            | Sellery
            | Sesame
            | Soy
            | Sulfite
            | Sweetener
            | Waxed => false,
        }
    }

    /// Is this **not** a primary tag?
    pub fn is_secondary(&self) -> bool {
        !self.is_primary()
    }

    /// Describe this [`Tag`] with english words.
    ///
    /// This should add information where the enum variant itself
    /// does not suffice.
    pub fn describe(&self) -> &'static str {
        match self {
            Self::Acidifier => "Contains artificial acidifier",
            Self::Alcohol => "Contains alcohol",
            Self::Antioxidant => "Contains an antioxidant",
            Self::Blackened => {
                "Contains ingredients that have been blackened, i.e. blackened olives"
            }
            Self::CacaoContainingFatGlaze => {
                "Contains a cheap oil-based glazing instead of real chocolate glazing"
            }
            Self::Coloring => "Contains food coloring",
            Self::Cow => "Contains meat from cattle",
            Self::Egg => "Contains egg",
            Self::Fish => "Contains fish",
            Self::FlavorEnhancer => "Contains artificial flavor enhancer",
            Self::Garlic => "Contains garlic",
            Self::Gluten => "Contains gluten",
            Self::Lactose => "Contains milk",
            Self::Lupin => "Contains lupin",
            Self::Mustard => "Contains mustard",
            Self::Nuts => "Contains nuts",
            Self::Phosphate => "Contains phosphate",
            Self::Pig => "Contains meat from pig",
            Self::Poultry => "Contains poultry meat",
            Self::Preservative => "Contains artificial preservatives",
            Self::Sellery => "Contains sellery",
            Self::Sesame => "Contains sesame",
            Self::Soy => "Contains soy",
            Self::Sulfite => "Contains sulfite",
            Self::Sweetener => "Contains artificial sweetener",
            Self::Vegan => "Does not contain any animal produce",
            Self::Vegetarian => "Does not contain any meat",
            Self::Waxed => "Some ingredients have been waxed",
        }
    }

    /// This formats an identifier for this tag.
    ///
    /// Will respect any settings given, i.e. emojis will be used
    /// unless the output should be plain.
    pub fn as_id(&self) -> String {
        match self {
            Self::Vegan => if_plain!("🌱".into(), "Vegan".into()),
            Self::Vegetarian => if_plain!("🧀".into(), "Vegetarian".into()),
            Self::Pig => if_plain!("🐖".into(), "Pig".into()),
            Self::Fish => if_plain!("🐟".into(), "Fish".into()),
            Self::Cow => if_plain!("🐄".into(), "Cow".into()),
            Self::Poultry => if_plain!("🐓".into(), "Poultry".into()),
            _ => {
                // If no special emoji is available, just use the id
                let number: u8 = (*self).into();
                format!("{}", number)
            }
        }
    }

    /// Print this tag.
    ///
    /// Does **not** respect `--json`, use [`Self::print_all`].
    pub fn print(&self) {
        let emoji = if CONF.args.plain && self.is_primary() {
            format!("{:>width$}", "-", width = ID_WIDTH)
        } else {
            let emoji = self.as_id();
            let emoji_len = emoji.width();
            format!(
                "{}{}",
                " ".repeat(ID_WIDTH.saturating_sub(emoji_len)),
                emoji
            )
        };
        let description_width = get_sane_terminal_dimensions().0;
        let description = textwrap::fill(
            self.describe(),
            textwrap::Options::new(description_width)
                .initial_indent(TEXT_INDENT)
                .subsequent_indent(TEXT_INDENT),
        );
        println!(
            "{} {}\n{}",
            color!(emoji; bright_yellow, bold),
            color!(self; bold),
            color!(description; bright_black),
        );
    }

    /// Print all tags.
    pub fn print_all() -> Result<()> {
        if CONF.args.json {
            Self::print_all_json()
        } else {
            for tag in Tag::iter() {
                println!();
                tag.print();
            }
            Ok(())
        }
    }

    /// Print all tags as json.
    ///
    /// This will result in a list of objects containing the following keys:
    /// - id: An identifier, like 'Vegan' or '22'
    /// - name: The name of the tag.
    /// - desc: A simple description.
    ///
    fn print_all_json() -> Result<()> {
        let tags: Vec<HashMap<&str, String>> = Tag::iter()
            .map(|tag| {
                vec![
                    ("id", tag.as_id()),
                    ("name", tag.to_string()),
                    ("desc", tag.describe().to_owned()),
                ]
                .into_iter()
                .collect()
            })
            .collect();
        print_json(&tags)
    }
}
