use core::fmt;

use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use unicode_width::UnicodeWidthStr;

use crate::get_sane_terminal_dimensions;

use super::{MealId, Meta, PRE};

lazy_static! {
    static ref NAME_PRE: &'static str = if_plain!(" ╭───╴", " - ");
    static ref NAME_CONTINUE_PRE: &'static str = if_plain!(" ┊    ", "     ");
    static ref OTHER_NOTE_PRE: &'static str = if_plain!(" ├╴", "   ");
    static ref OTHER_NOTE_CONTINUE_PRE: &'static str = if_plain!(" ┊ ", "     ");
    static ref CATEGORY_PRE: &'static str = if_plain!(" ├─╴", "   ");
    static ref PRICES_PRE: &'static str = if_plain!(" ╰╴", "   ");
}

#[derive(Debug, Serialize)]
pub struct MealComplete<'c> {
    pub id: MealId,
    #[serde(flatten)]
    pub meta: &'c Meta,
}

impl<'c> MealComplete<'c> {
    /// Print this [`MealComplete`] to the terminal.
    pub fn print(&self, highlight: bool) {
        let (width, _height) = get_sane_terminal_dimensions();
        // Print meal name
        self.print_name_to_terminal(width, highlight);
        // Get notes, i.e. allergenes, descriptions, tags
        self.print_category_and_primary_tags(highlight);
        self.print_descriptions(width, highlight);
        self.print_price_and_secondary_tags(highlight);
    }

    fn print_name_to_terminal(&self, width: usize, highlight: bool) {
        let max_name_width = width - NAME_PRE.width() - PRE.width();
        let mut name_parts = textwrap::wrap(&self.meta.name, max_name_width).into_iter();
        // There will always be a first part of the splitted string
        let first_name_part = name_parts.next().unwrap();
        println!(
            "{}{}{}",
            *PRE,
            hl_if(highlight, *NAME_PRE),
            color!(hl_if(highlight, first_name_part); bold),
        );
        for name_part in name_parts {
            let name_part = hl_if(highlight, name_part);
            println!(
                "{}{}{}",
                *PRE,
                hl_if(highlight, *NAME_CONTINUE_PRE),
                color!(name_part; bold),
            );
        }
    }

    fn print_category_and_primary_tags(&self, highlight: bool) {
        let mut tag_str = self
            .meta
            .tags
            .iter()
            .filter(|tag| tag.is_primary())
            .map(|tag| tag.as_id());
        let tag_str_colored =
            if_plain!(color!(tag_str.join(" "); bright_black), tag_str.join(", "));
        let comma_if_plain = if_plain!("", ",");
        println!(
            "{}{}{}{} {}",
            *PRE,
            hl_if(highlight, *CATEGORY_PRE),
            color!(self.meta.category; bright_blue),
            color!(comma_if_plain; bright_black),
            tag_str_colored
        );
    }

    fn print_descriptions(&self, width: usize, highlight: bool) {
        let max_note_width = width - OTHER_NOTE_PRE.width() - PRE.width();
        for note in &self.meta.descs {
            let mut note_parts = textwrap::wrap(note, max_note_width).into_iter();
            // There will always be a first part in the splitted string
            println!(
                "{}{}{}",
                *PRE,
                hl_if(highlight, *OTHER_NOTE_PRE),
                note_parts.next().unwrap()
            );
            for part in note_parts {
                println!(
                    "{}{}{}",
                    *PRE,
                    hl_if(highlight, *OTHER_NOTE_CONTINUE_PRE),
                    part
                );
            }
        }
    }

    fn print_price_and_secondary_tags(&self, highlight: bool) {
        let prices = self.meta.prices.to_terminal_string();
        let mut secondary: Vec<_> = self
            .meta
            .tags
            .iter()
            .filter(|tag| tag.is_secondary())
            .collect();
        secondary.sort_unstable();
        let secondary_str = secondary.iter().map(|tag| tag.as_id()).join(" ");
        println!(
            "{}{}{}  {}",
            *PRE,
            hl_if(highlight, *PRICES_PRE),
            prices,
            color!(secondary_str; bright_black),
        );
    }
}

fn hl_if<S>(highlight: bool, text: S) -> String
where
    S: fmt::Display,
{
    if highlight {
        color!(text; bright_yellow)
    } else {
        format!("{}", text)
    }
}
