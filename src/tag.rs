use lazy_static::lazy_static;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use crate::config::State;

lazy_static! {
    /// These must have the same order as the variants in the [`Tag`] enum.
    static ref TAG_RE: RegexSet = RegexSet::new(&[
        r"(?i)alkohol",
        r"(?i)antioxidation",
        r"(?i)geschwärzt",
        r"(?i)farbstoff",
        r"(?i)rind",
        r"(?i)eier",
        r"(?i)fisch",
        r"(?i)geschmacksverstärker",
        r"(?i)knoblauch",
        r"(?i)gluten",
        r"(?i)lupine?",
        r"(?i)milch",
        r"(?i)senf",
        r"(?i)schalenfrüchte|nüsse",
        r"(?i)phosphat",
        r"(?i)schwein",
        r"(?i)geflügel",
        r"(?i)konservierung",
        r"(?i)sellerie",
        r"(?i)sesam",
        r"(?i)soja",
        r"(?i)sulfit|schwefel",
        r"(?i)süßungsmittel",
        r"(?i)vegan",
        r"(?i)fleischlos|vegetarisch|ohne fleisch",
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
    Alcohol,
    Antioxidant,
    Blackened,
    Coloring,
    Cow,
    Egg,
    Fish,
    FlavorEnhancer,
    Garlic,
    Gluten,
    Lupin,
    Milk,
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
            Alcohol | Antioxidant | Blackened | Coloring | Egg | FlavorEnhancer | Garlic
            | Gluten | Lupin | Milk | Mustard | Nuts | Phosphate | Preservative | Sellery
            | Sesame | Soy | Sulfite | Sweetener => false,
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
            Self::Alcohol => "Contains alcohol",
            Self::Antioxidant => "Contains an antioxidant",
            Self::Blackened => {
                "Contains ingredients that have been blackened, i.e. blackened olives"
            }
            Self::Coloring => "Contains food coloring",
            Self::Cow => "Contains meat from cattle",
            Self::Egg => "Contains egg",
            Self::Fish => "Contains fish",
            Self::FlavorEnhancer => "Contains artificial flavor enhancer",
            Self::Garlic => "Contains garlic",
            Self::Gluten => "Contains gluten",
            Self::Lupin => "Contains lupin",
            Self::Milk => "Contains milk",
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
        }
    }

    /// This formats an identifier for this tag.
    ///
    /// Will respect any settings given, i.e. emojis will be used
    /// unless the output should be plain.
    pub fn as_id<Cmd>(&self, state: &State<Cmd>) -> String {
        match self {
            Self::Vegan => if_plain!(state: "🌱".into(), "Vegan".into()),
            Self::Vegetarian => if_plain!(state:"🧀".into(), "Vegetarian".into()),
            Self::Pig => if_plain!(state:"🐖".into(), "Pig".into()),
            Self::Fish => if_plain!(state:"🐟".into(), "Fish".into()),
            Self::Cow => if_plain!(state:"🐄".into(), "Cow".into()),
            Self::Poultry => if_plain!(state:"🐓".into(), "Poultry".into()),
            _ => {
                // If no special emoji is available, just use the id
                let number: u8 = (*self).into();
                format!("{}", number)
            }
        }
    }
}