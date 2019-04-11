use colored::*;

use std::env::args;
use std::error;

mod meal;
mod parser;

use meal::Meal;
use parser::SearchParser;

fn main() -> Result<(), Box<error::Error>> {
    // Fold all arguments for parsing
    let args = args().skip(1);
    let input = args.fold(String::new(), |acc, arg| format!("{} {}", acc, arg));
    // Parse options into Option struct
    let opts = SearchParser::parse_options(&input)?;
    // Fetch meals from webpage
    let meals = Meal::get_meals(opts)?;

    let mut last_location = None;
    let mut last_category = None;
    // Print everything
    println!("");
    for meal in meals {
        if last_location != Some(meal.location.clone()) {
            last_location = Some(meal.location.clone());
            println!("{}:", meal.location.name.bold());
        }
        if last_category != Some(meal.category.clone()) {
            last_category = Some(meal.category.clone());
            print!("{:>30}  ", meal.category.green());
        } else {
            print!("                                ");
        }
        println!("{}", meal.name);
    }
    println!("");

    Ok(())
}
