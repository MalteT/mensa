use lazy_static::lazy_static;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use regex::RegexSet;
use serde::Deserialize;
use strum::{Display, EnumIter};

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
    pub fn parse_str(raw: &str) -> Vec<Self> {
        TAG_RE
            .matches(raw)
            .iter()
            .map(|idx| Tag::try_from_primitive(idx as u8).unwrap())
            .collect()
    }

    pub fn is_primary(&self) -> bool {
        use Tag::*;
        match self {
            Cow | Fish | Pig | Poultry | Vegan | Vegetarian => true,
            Alcohol | Antioxidant | Blackened | Coloring | Egg | FlavorEnhancer | Garlic
            | Gluten | Milk | Mustard | Nuts | Phosphate | Preservative | Sellery | Sesame
            | Soy | Sulfite | Sweetener => false,
        }
    }

    pub fn is_secondary(&self) -> bool {
        !self.is_primary()
    }

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

    pub fn as_emoji(&self) -> String {
        match self {
            Self::Vegan => "🌱".into(),
            Self::Vegetarian => "🧀".into(),
            Self::Pig => "🐖".into(),
            Self::Fish => "🐟".into(),
            Self::Cow => "🐄".into(),
            Self::Poultry => "🐓".into(),
            _ => {
                // If no special emoji is available, just use the id
                let number: u8 = (*self).into();
                format!("{}", number)
            }
        }
    }
}
