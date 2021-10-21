use core::fmt;

use itertools::Itertools;
use serde::Serialize;
use unicode_width::UnicodeWidthStr;

use crate::get_sane_terminal_dimensions;

use super::{MealId, Meta};

const NAME_PRE: &str = " ╭───╴";
const NAME_PRE_PLAIN: &str = " - ";
const NAME_CONTINUE_PRE: &str = " ┊    ";
const NAME_CONTINUE_PRE_PLAIN: &str = "     ";
const OTHER_NOTE_PRE: &str = " ├╴";
const OTHER_NOTE_PRE_PLAIN: &str = "   ";
const OTHER_NOTE_CONTINUE_PRE: &str = " ┊ ";
const OTHER_NOTE_CONTINUE_PRE_PLAIN: &str = "     ";
const CATEGORY_PRE: &str = " ├─╴";
const CATEGORY_PRE_PLAIN: &str = "   ";
const PRICES_PRE: &str = " ╰╴";
const PRICES_PRE_PLAIN: &str = "   ";

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
        let max_name_width = width - NAME_PRE.width();
        let mut name_parts = textwrap::wrap(&self.meta.name, max_name_width).into_iter();
        // There will always be a first part of the splitted string
        let first_name_part = name_parts.next().unwrap();
        let pre = if_plain!(NAME_PRE, NAME_PRE_PLAIN);
        println!(
            "{}{}",
            hl_if(highlight, pre),
            color!(hl_if(highlight, first_name_part); bold),
        );
        for name_part in name_parts {
            let name_part = hl_if(highlight, name_part);
            let pre = if_plain!(NAME_CONTINUE_PRE, NAME_CONTINUE_PRE_PLAIN);
            println!("{}{}", hl_if(highlight, pre), color!(name_part; bold),);
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
        let pre = if_plain!(CATEGORY_PRE, CATEGORY_PRE_PLAIN);
        let comma_if_plain = if_plain!("", ",");
        println!(
            "{}{}{} {}",
            hl_if(highlight, pre),
            color!(self.meta.category; bright_blue),
            color!(comma_if_plain; bright_black),
            tag_str_colored
        );
    }

    fn print_descriptions(&self, width: usize, highlight: bool) {
        let pre = if_plain!(OTHER_NOTE_PRE, OTHER_NOTE_PRE_PLAIN);
        let pre_continue = if_plain!(OTHER_NOTE_CONTINUE_PRE, OTHER_NOTE_CONTINUE_PRE_PLAIN);
        let max_note_width = width - OTHER_NOTE_PRE.width();
        for note in &self.meta.descs {
            let mut note_parts = textwrap::wrap(note, max_note_width).into_iter();
            // There will always be a first part in the splitted string
            println!("{}{}", hl_if(highlight, pre), note_parts.next().unwrap());
            for part in note_parts {
                println!("{}{}", hl_if(highlight, pre_continue), part);
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
        let pre = if_plain!(PRICES_PRE, PRICES_PRE_PLAIN);
        println!(
            "{}{}  {}",
            hl_if(highlight, pre),
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
