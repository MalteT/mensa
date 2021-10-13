use regex::{Regex, RegexSet};
use serde::Deserialize;
use std::convert::TryFrom;

use crate::{
    error::{Error, Result},
    meal::{tag::Tag, Meal},
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Rule {
    #[serde(default)]
    pub name: RegexRule,
    #[serde(default)]
    pub tag: TagRule,
    #[serde(default)]
    pub category: RegexRule,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TagRule {
    #[serde(default)]
    pub add: Vec<Tag>,
    #[serde(default)]
    pub sub: Vec<Tag>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(try_from = "RawRegexRule")]
pub struct RegexRule {
    pub add: Option<RegexSet>,
    pub sub: Option<RegexSet>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RawRegexRule {
    #[serde(default)]
    pub add: Vec<String>,
    #[serde(default)]
    pub sub: Vec<String>,
}

impl Rule {
    pub fn is_match(&self, meal: &Meal) -> bool {
        let all_adds_empty =
            self.tag.is_empty_add() && self.category.is_empty_add() && self.name.is_empty_add();
        let any_add = self.tag.is_match_add(meal)
            || self.category.is_match_add(meal)
            || self.name.is_match_add(meal);
        let any_sub = self.tag.is_match_sub(meal)
            || self.category.is_match_sub(meal)
            || self.name.is_match_sub(meal);
        (all_adds_empty || any_add) && !any_sub
    }

    pub fn joined(self, other: Self) -> Self {
        Self {
            name: self.name.joined(other.name),
            tag: self.tag.joined(other.tag),
            category: self.category.joined(other.category),
        }
    }
}

impl TagRule {
    fn is_match_add(&self, meal: &Meal) -> bool {
        self.add.iter().any(|tag| meal.tags.contains(tag))
    }

    fn is_match_sub(&self, meal: &Meal) -> bool {
        self.sub.iter().any(|tag| meal.tags.contains(tag))
    }

    fn is_empty_add(&self) -> bool {
        self.add.is_empty()
    }

    fn joined(mut self, other: Self) -> Self {
        self.add.extend(other.add);
        self.sub.extend(other.sub);
        self
    }
}

impl RegexRule {
    pub fn from_arg_parts(add: &[Regex], sub: &[Regex]) -> Self {
        let add: Vec<_> = add.iter().map(|re| re.as_str().to_owned()).collect();
        let sub: Vec<_> = sub.iter().map(|re| re.as_str().to_owned()).collect();
        // This should not panic, since we're assembling from regexes that were valid before
        let add = if add.is_empty() {
            None
        } else {
            Some(RegexSet::new(&add).unwrap())
        };
        let sub = if sub.is_empty() {
            None
        } else {
            Some(RegexSet::new(&sub).unwrap())
        };
        Self { add, sub }
    }

    fn is_match_add(&self, meal: &Meal) -> bool {
        match self.add {
            Some(ref rset) => rset.is_match(&meal.category),
            None => false,
        }
    }

    fn is_match_sub(&self, meal: &Meal) -> bool {
        match self.sub {
            Some(ref rset) => rset.is_match(&meal.category),
            None => false,
        }
    }

    fn is_empty_add(&self) -> bool {
        self.add.is_none()
    }

    fn joined(self, other: RegexRule) -> RegexRule {
        let option_and = |this: Option<RegexSet>, other: Option<RegexSet>| {
            match (this, other) {
                (Some(this), Some(other)) => {
                    let mut patterns = this.patterns().to_vec();
                    patterns.extend(other.patterns().to_vec());
                    // This should not panic, as it was valid before
                    Some(RegexSet::new(patterns).unwrap())
                }
                (Some(this), None) => Some(this),
                (None, Some(other)) => Some(other),
                (None, None) => None,
            }
        };
        Self {
            add: option_and(self.add, other.add),
            sub: option_and(self.sub, other.sub),
        }
    }
}

impl TryFrom<RawRegexRule> for RegexRule {
    type Error = Error;

    fn try_from(raw: RawRegexRule) -> Result<Self> {
        let add = slice_to_option(
            &raw.add,
            RegexSet::new(&raw.add).map_err(Error::ParsingFilterRegex)?,
        );
        let sub = slice_to_option(
            &raw.sub,
            RegexSet::new(&raw.sub).map_err(Error::ParsingFilterRegex)?,
        );
        Ok(Self { add, sub })
    }
}

fn slice_to_option<T, V>(vec: &[T], val: V) -> Option<V> {
    if vec.is_empty() {
        None
    } else {
        Some(val)
    }
}
