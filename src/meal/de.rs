use serde::Deserialize;

use std::collections::HashSet;

use crate::{cache::Fetchable, tag::Tag};

use super::{MealId, Meta, Note, Prices};

#[derive(Debug, Deserialize)]
#[cfg_attr(debug, serde(deny_unknown_fields))]
pub struct Meal {
    id: MealId,
    name: String,
    notes: Vec<String>,
    prices: Prices,
    category: String,
}

impl Meal {
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

impl From<Meal> for super::Meal {
    fn from(raw: Meal) -> Self {
        let (tags, descs) = raw.parse_and_split_notes();
        Self {
            id: raw.id,
            meta: Fetchable::Fetched(Meta {
                name: raw.name,
                prices: raw.prices,
                category: raw.category,
                tags,
                descs,
            }),
        }
    }
}
