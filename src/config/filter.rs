use regex::{Regex, RegexSet};
use serde::Deserialize;

use crate::{
    error::{Error, Result},
    meal::{Meal, Tag},
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Filter {
    #[serde(default)]
    pub tag: TagFilter,
    #[serde(default)]
    pub category: CategoryFilter,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TagFilter {
    #[serde(default)]
    pub allow: Vec<Tag>,
    #[serde(default)]
    pub deny: Vec<Tag>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(try_from = "CategoryFilterRaw")]
pub struct CategoryFilter {
    pub allow: Option<RegexSet>,
    pub deny: Option<RegexSet>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CategoryFilterRaw {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

impl Filter {
    pub fn is_allowed(&self, meal: &Meal) -> bool {
        let any_allows = self.tag.allows(meal) || self.category.allows(meal);
        let any_denies = self.tag.denies(meal) || self.category.denies(meal);
        any_allows && !any_denies
    }

    pub fn joined(self, other: Filter) -> Filter {
        Self {
            tag: self.tag.joined(other.tag),
            category: self.category.joined(other.category),
        }
    }
}

impl TagFilter {
    fn allows(&self, meal: &Meal) -> bool {
        self.allow.is_empty() || self.allow.iter().any(|allow| meal.tags.contains(allow))
    }

    fn denies(&self, meal: &Meal) -> bool {
        self.deny.iter().any(|deny| meal.tags.contains(deny))
    }

    fn joined(mut self, other: TagFilter) -> TagFilter {
        self.allow.extend(other.allow);
        self.deny.extend(other.deny);
        self
    }
}

impl CategoryFilter {
    pub fn from_arg_parts(allow: &[Regex], deny: &[Regex]) -> Self {
        let allow: Vec<_> = allow.iter().map(|re| re.as_str().to_owned()).collect();
        let deny: Vec<_> = deny.iter().map(|re| re.as_str().to_owned()).collect();
        // This should not panic, since we're assembling from regexes that were valid before
        let allow = if allow.is_empty() {
            None
        } else {
            Some(RegexSet::new(&allow).unwrap())
        };
        let deny = if deny.is_empty() {
            None
        } else {
            Some(RegexSet::new(&deny).unwrap())
        };
        Self { allow, deny }
    }

    fn allows(&self, meal: &Meal) -> bool {
        match self.allow {
            Some(ref allow) => allow.is_match(&meal.category),
            None => true,
        }
    }

    fn denies(&self, meal: &Meal) -> bool {
        match self.deny {
            Some(ref deny) => deny.is_match(&meal.category),
            None => false,
        }
    }

    fn joined(self, other: CategoryFilter) -> CategoryFilter {
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
            allow: option_and(self.allow, other.allow),
            deny: option_and(self.deny, other.deny),
        }
    }
}

impl TryFrom<CategoryFilterRaw> for CategoryFilter {
    type Error = Error;

    fn try_from(raw: CategoryFilterRaw) -> Result<Self> {
        let allow = if raw.allow.is_empty() {
            None
        } else {
            Some(RegexSet::new(&raw.allow).map_err(Error::ParsingFilterRegex)?)
        };
        let deny = if raw.deny.is_empty() {
            None
        } else {
            Some(RegexSet::new(&raw.deny).map_err(Error::ParsingFilterRegex)?)
        };
        Ok(Self { allow, deny })
    }
}
