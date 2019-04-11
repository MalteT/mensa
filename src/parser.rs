use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::offset::TimeZone;
use chrono::Date;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use std::error::Error;

use crate::meal::Location;
use crate::meal::Options;
use crate::meal::Restriction;

#[derive(Parser)]
#[grammar = "../search_format.pest"]
pub struct SearchParser;

impl SearchParser {
    /// Parse Search Options into Options struct.
    pub fn parse_options(input: &str) -> Result<Options, Box<Error>> {
        let parsed = Self::parse(Rule::input, input)?
            .next()
            .unwrap()
            .into_inner();
        let mut opts: Options = Default::default();

        for pair in parsed {
            match pair.as_rule() {
                Rule::location => opts.location = SearchParser::parse_location(&pair),
                Rule::exclude => {
                    opts.restrictions = opts.restrictions.and(&SearchParser::parse_exclude(&pair))
                }
                Rule::date => opts.date = SearchParser::parse_date(&pair),
                Rule::restriction => {
                    opts.restrictions = opts
                        .restrictions
                        .and(&SearchParser::parse_restriction(&pair))
                }
                Rule::EOI => {}
                _ => println!("{:?}", pair),
            }
        }
        Ok(opts)
    }
    /// Parse a restriction rule into [Restriction].
    fn parse_restriction(pair: &Pair<Rule>) -> Restriction {
        let pair = pair
            .clone()
            .into_inner()
            .next()
            .expect("restriction rule has no inner rules");
        match pair.as_rule() {
            Rule::restriction_vegan => Restriction::Vegan { no_alcohol: false },
            Rule::restriction_vegetarian => Restriction::Vegetarian { no_alcohol: false },
            Rule::restriction_flexible => Restriction::Flexible {
                no_alcohol: false,
                no_fish: false,
                no_pig: false,
            },
            _ => unreachable!(),
        }
    }
    /// Parse an exclude rule into [Restriction].
    fn parse_exclude(pair: &Pair<Rule>) -> Restriction {
        let mut no_fish = false;
        let mut no_pig = false;
        let mut no_alcohol = false;
        let pair = pair
            .clone()
            .into_inner()
            .next()
            .expect("exclude has no inner rule");
        match pair.as_rule() {
            Rule::exclude_pig => no_pig = true,
            Rule::exclude_fish => no_fish = true,
            Rule::exclude_alcohol => no_alcohol = true,
            _ => unreachable!(),
        }
        Restriction::Flexible {
            no_fish,
            no_pig,
            no_alcohol,
        }
    }
    /// Parse a date rule into a [Date].
    fn parse_date(pair: &Pair<Rule>) -> Date<Local> {
        let pair = pair
            .clone()
            .into_inner()
            .next()
            .expect("date contains no date_spec");
        let pair_str = pair.as_str().to_lowercase();
        let pair_str = pair_str.as_str();
        if pair_str == "today" {
            Local::today()
        } else if pair_str == "tomorrow" {
            Local::today().succ()
        } else {
            let pair = pair
                .into_inner()
                .next()
                .expect("date_spec has unhandled options");
            if pair.as_rule() == Rule::date_yyyymmdd {
                let date = NaiveDate::parse_from_str(pair.as_str(), "%Y-%m-%d")
                    .expect("date_yyyymmdd date format broken");
                Local::from_local_date(&Local, &date)
                    .earliest()
                    .expect("Local to timezone aware date failed")
            } else if pair.as_rule() == Rule::date_weekday {
                let mut date = Local::today();
                let mut weekday = format!("{}", date.format("%a"));
                while !pair_str.starts_with(&weekday) {
                    date = date.succ();
                    weekday = format!("{}", date.format("%a"));
                    weekday = weekday.to_lowercase();
                }
                date
            } else {
                println!("{} -> {}", pair, pair.as_str());
                unreachable!()
            }
        }
    }
    /// Parse a location rule into a location
    fn parse_location(pair: &Pair<Rule>) -> Location {
        let pair = pair
            .clone()
            .into_inner()
            .next()
            .expect("location rule does not contain a mensa")
            .into_inner()
            .next()
            .expect("mensa name does not contain a specific rule");
        let id = match pair.as_rule() {
            Rule::mensa_153 => "153",
            Rule::mensa_127 => "127",
            Rule::mensa_118 => "118",
            Rule::mensa_106 => "106",
            Rule::mensa_115 => "115",
            Rule::mensa_162 => "162",
            Rule::mensa_111 => "111",
            Rule::mensa_140 => "140",
            Rule::mensa_170 => "170",
            Rule::mensa_all => "all",
            _ => unreachable!(),
        };
        Location::from_id(id).expect("No location from id found. this should never happen")
    }
}

#[cfg(test)]
mod test {
    use super::Rule;
    use super::SearchParser;
    use pest::Parser;
    use std::error::Error;

    macro_rules! parse {
        ($rule:expr, $parse:expr, $res:expr) => {{
            let x = SearchParser::parse($rule, $parse);
            if let Ok(ref x) = x {
                assert_eq!(x.concat(), $res);
            }
            x
        }};
    }

    macro_rules! parse_err {
        ($rule:expr, $parse:expr) => {{
            let x = SearchParser::parse($rule, $parse);
            assert!(x.is_err());
        }};
    }

    #[test]
    fn test_mensa_106() -> Result<(), Box<Error>> {
        use Rule::mensa_106;

        parse!(mensa_106, "main", "main")?;
        parse!(mensa_106, "mensa AM PaRk", "mensa AM PaRk")?;
        parse!(mensa_106, "mensa AM PaRk", "mensa AM PaRk")?;
        Ok(())
    }

    #[test]
    fn test_location() -> Result<(), Box<Error>> {
        use Rule::location;

        parse!(location, "at main", "at main")?;
        parse!(location, "at Mensa am Park", "at Mensa am Park")?;
        parse!(location, "at park", "at park")?;
        parse!(location, "at Tierklinik", "at Tierklinik")?;
        Ok(())
    }

    #[test]
    fn test_exclude() -> Result<(), Box<Error>> {
        use Rule::exclude;

        parse!(exclude, "no fish", "no fish")?;
        parse!(exclude, "no pig", "no pig")?;
        parse!(exclude, "no alcohol", "no alcohol")?;
        parse_err!(exclude, "no diff");
        Ok(())
    }

    #[test]
    fn test_date() -> Result<(), Box<Error>> {
        use Rule::date;

        parse!(date, "on today", "on today")?;
        parse_err!(date, "on some other day");
        Ok(())
    }

    #[test]
    fn test_restriction() -> Result<(), Box<Error>> {
        use Rule::restriction;

        parse!(restriction, "vegan", "vegan")?;
        parse!(restriction, "VEGGIe", "VEGGIe")?;
        parse!(restriction, "vegetarian", "vegetarian")?;
        parse!(restriction, "flexible", "flexible")?;
        Ok(())
    }

    #[test]
    fn test_input() -> Result<(), Box<Error>> {
        use Rule::input;

        parse!(
            input,
            "no alcohol at all vegan no alcohol",
            "no alcohol at all vegan no alcohol"
        )?;
        Ok(())
    }
}
