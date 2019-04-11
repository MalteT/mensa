use chrono::Date;
use chrono::Local;
use ego_tree::NodeRef;
use lazy_static::lazy_static;
use reqwest;
use reqwest::Url;
use scraper::node::Element;
use scraper::node::Node;
use scraper::Html;
use scraper::Selector;

use std::error;

/// HOST to fetch information from.
pub const HOST: &'static str = "https://www.studentenwerk-leipzig.de/mensen-cafeterien/speiseplan/";

lazy_static! {
    /// List of all possible locations.
    static ref LOCATIONS: Vec<Location> =
        Location::get_all_locations().expect("Location retrieval failed!");
}

/// Remove bullshit text nodes.
macro_rules! no_bs_kids {
    ( $el:expr ) => {
        $el.children().filter(|el| !el.value().is_text())
    };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
/// A single meal served at some place.
pub struct Meal {
    /// The name of the meal.
    pub name: String,
    /// The location which is serving the meal.
    pub location: Location,
    /// The category this meal is filed under.
    pub category: String,
}

impl Meal {
    /// Fetch meals from
    /// [https://www.studentenwerk-leipzig.de/mensen-cafeterien/speiseplan](https://www.studentenwerk-leipzig.de/mensen-cafeterien/speiseplan).
    /// - FIXME: No food on sundays!
    /// - FIXME: Refactor
    pub fn get_meals(opts: Options) -> Result<Vec<Meal>, Box<error::Error>> {
        // Format url
        let url: Url = HOST.parse().expect("Host not valid");
        let opts_url = opts.as_url_part();
        let url = url
            .join(&format!("?{}", &opts_url))
            .expect("New Url is not valid");
        // Send request
        let resp: String = reqwest::get(url)?.text()?;
        // Parse HTML
        let html = Html::parse_document(&resp);
        // Select the meal sections
        let meal_selector = Selector::parse("section.meals").expect("Meal selector failed");
        let meals = html.select(&meal_selector);
        // Iterate over all locations
        let mut ret: Vec<Meal> = vec![];
        for meal_at in meals {
            let mut location: Option<Location> = None;
            let mut category: Option<String> = None;
            // Iterate over h2, h3 and divs inside
            for el in meal_at.children() {
                if let Node::Element(el_value) = el.value() {
                    match el_value.name() {
                        // All h2 are locations
                        "h2" => {
                            location = el
                                .first_child()
                                .unwrap()
                                .value()
                                .as_text()
                                .map(|x| Location::from_name(x).expect("Location not found"));
                        }
                        // All h3 are categories
                        "h3" => {
                            category = el
                                .first_child()
                                .unwrap()
                                .value()
                                .as_text()
                                .map(|x| x.to_string());
                        }
                        "div" => {
                            let mut meals = parse_meal_names(el)
                                .iter()
                                .map(|name| Meal {
                                    location: location.clone().expect("Meal has no location"),
                                    category: category.clone().expect("Meal has no category"),
                                    name: name.to_string(),
                                })
                                .collect();
                            ret.append(&mut meals);
                        }
                        // There are some _stupid_ text nodes
                        _ => {}
                    }
                }
            }
        }
        Ok(ret)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
/// A Canteen.
pub struct Location {
    /// Unique identifier
    pub id: String,
    /// German name of the place
    pub name: String,
}

impl Location {
    /// Fetch all possible locations.
    pub fn get_all_locations() -> Result<Vec<Location>, Box<error::Error>> {
        // Fetch the webpage and convert it to String
        let resp: String = reqwest::get(HOST)?.text()?;
        // Parse the html response
        let html = Html::parse_document(&resp);
        // Selector for the location dropdown
        let location_selector = Selector::parse("#edit-location").expect("Selector failed");
        // Select the relevant html text
        let locations = html.select(&location_selector).next().unwrap();
        // Go through all children and parse to locations
        let mut list_of_locations = vec![];
        for location in locations.children() {
            let value: &Element = location
                .value()
                .as_element()
                .expect("Location entry is not an element");
            let number: &str = value.attr("value").expect("Location has no value");
            let name = location
                .first_child()
                .expect("Location entry has no name")
                .value()
                .as_text()
                .expect("Location name is not just text");

            list_of_locations.push(Location {
                name: name.to_string(),
                id: number.to_string(),
            });
        }
        Ok(list_of_locations)
    }
    /// Try to create a location from a given name.
    pub fn from_name(name: &str) -> Option<Self> {
        let mut ret = None;
        let name = name.to_string();
        for l in LOCATIONS.iter() {
            if name == l.name {
                ret = Some(l.clone());
                break;
            }
        }
        ret
    }
    /// Try to create a location from a given id.
    pub fn from_id(id: &str) -> Option<Self> {
        let mut ret = None;
        let id = id.to_string();
        for l in LOCATIONS.iter() {
            if id == l.id {
                ret = Some(l.clone());
                break;
            }
        }
        ret
    }
}


/// # Structure
///              div
///   sec:         section
///   header:        header
///   head:            div.meals__head
///   summary:           div.meals__summary
///   name_h4              h4.meals__name
///   name                   [NAME]
///   price_p              p.meals__price
///                          span.u-hidden
///   price                  [PREIS]
///   badges_div         div.meals__badges
///                        span.u-hidde
///   badges_i             [i BADGES]
///                  details
///                    summary
///                    [mixed DESCRIPTION]
/// TODO: Badges
/// TODO: Price
fn parse_meal_names(el: NodeRef<Node>) -> Vec<String> {
    let mut ret = vec![];
    for sec in no_bs_kids!(el) {
        let header = no_bs_kids!(sec).next().expect("sec has no children");
        let head = no_bs_kids!(header).next().expect("header has no child");
        let summary = no_bs_kids!(head).next().expect("head has no child");
        let name_h4 = no_bs_kids!(summary).next().expect("summary has no child");
        let mut name_h4_children = name_h4.children();
        let name = name_h4_children
            .next()
            .expect("no title")
            .value()
            .as_text()
            .expect("name is no text");

        ret.push(name.to_string());
    }
    ret
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Restrictions to match meals against.
pub struct Options {
    /// General restrictions for types of food.
    pub restrictions: Restriction,
    /// Location from at which the meal has to be served.
    pub location: Location,
    /// Date at which the meal has to be served.
    pub date: Date<Local>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
/// General restrictions available at the website.
pub enum Restriction {
    /// Just plants, please.
    Vegan {
        no_alcohol: bool,
    },
    /// No meat, please.
    Vegetarian {
        no_alcohol: bool,
    },
    /// I don't care about our planet 🙄.
    Flexible {
        no_pig: bool,
        no_fish: bool,
        no_alcohol: bool,
    },
}

impl Restriction {
    /// Format as URL part.
    ///
    /// - `Vegan` becomes `&mealtype=58`
    /// - `Vegetarian` becomes `&mealtype=50`
    /// - `Flexible` becomes `&mealtype=all`
    ///
    /// The following options will be formatted `&criteria=x, y, z`
    ///
    /// - `Alcohol` is criteria `44`
    /// - `Fish` is critera `56`
    /// - `Pig` is critera `51`
    pub fn as_url_part(&self) -> String {
        match self {
            Restriction::Vegan { no_alcohol } => {
                let mut ret = String::from("meal_type=58");
                if *no_alcohol {
                    ret += "&critera=44";
                }
                ret
            }
            Restriction::Vegetarian { no_alcohol } => {
                let mut ret = String::from("meal_type=50");
                if *no_alcohol {
                    ret += "&critera=44";
                }
                ret
            }
            Restriction::Flexible {
                no_pig,
                no_fish,
                no_alcohol,
            } => {
                let mut ret = String::from("meal_type=all");
                let mut criteria = vec![];
                if *no_pig {
                    criteria.push(51);
                }
                if *no_fish {
                    criteria.push(56);
                }
                if *no_alcohol {
                    criteria.push(44);
                }
                if *no_pig || *no_fish || *no_alcohol {
                    ret += "&criteria=";
                    for c in criteria {
                        ret += &format!("{},", c)
                    }
                    ret = ret.trim_end_matches(",").to_string();
                }
                ret
            }
        }
    }
    /// Change Restriction as to respect another Restriction.
    /// Thus only the meals that match both restrictions will match
    /// the new one.
    pub fn and(&self, other: &Restriction) -> Self {
        match self {
            Restriction::Vegan { no_alcohol } => match other {
                Restriction::Vegan {
                    no_alcohol: o_no_alcohol,
                }
                | Restriction::Vegetarian {
                    no_alcohol: o_no_alcohol,
                }
                | Restriction::Flexible {
                    no_alcohol: o_no_alcohol,
                    no_pig: _,
                    no_fish: _,
                } => Restriction::Vegan {
                    no_alcohol: *no_alcohol | *o_no_alcohol,
                },
            },
            Restriction::Vegetarian { no_alcohol } => match other {
                Restriction::Vegan {
                    no_alcohol: o_no_alcohol,
                } => Restriction::Vegan {
                    no_alcohol: *o_no_alcohol | *no_alcohol,
                },
                Restriction::Vegetarian {
                    no_alcohol: o_no_alcohol,
                }
                | Restriction::Flexible {
                    no_alcohol: o_no_alcohol,
                    no_pig: _,
                    no_fish: _,
                } => Restriction::Vegetarian {
                    no_alcohol: *no_alcohol | *o_no_alcohol,
                },
            },
            Restriction::Flexible {
                no_alcohol,
                no_fish,
                no_pig,
            } => match other {
                Restriction::Vegan {
                    no_alcohol: o_no_alcohol,
                } => Restriction::Vegan {
                    no_alcohol: *o_no_alcohol | *no_alcohol,
                },
                Restriction::Vegetarian {
                    no_alcohol: o_no_alcohol,
                } => Restriction::Vegetarian {
                    no_alcohol: *o_no_alcohol | *no_alcohol,
                },
                Restriction::Flexible {
                    no_alcohol: o_no_alcohol,
                    no_pig: o_no_pig,
                    no_fish: o_no_fish,
                } => Restriction::Flexible {
                    no_alcohol: *no_alcohol | *o_no_alcohol,
                    no_pig: *no_pig | *o_no_pig,
                    no_fish: *no_fish | *o_no_fish,
                },
            },
        }
    }
}

impl Options {
    /// Format as URL part.
    pub fn as_url_part(&self) -> String {
        let mut url = self.restrictions.as_url_part();
        url += &format!("&location={}", self.location.id);
        url += &format!("&date={}", self.date.format("%Y-%m-%d"));
        url
    }
}

impl Default for Location {
    fn default() -> Self {
        Self {
            id: String::from("all"),
            name: String::from("Alle Mensen"),
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            date: Local::today(),
            location: Location::default(),
            restrictions: Restriction::default(),
        }
    }
}

impl Default for Restriction {
    fn default() -> Self {
        Restriction::Flexible {
            no_pig: false,
            no_fish: false,
            no_alcohol: false,
        }
    }
}
