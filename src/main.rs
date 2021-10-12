mod config;
mod error;
mod meal;

use config::CONFIG;
use error::Result;
use meal::Meal;

const ENDPOINT: &str = "https://openmensa.org/api/v2";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    // TODO: Actually do what the user wants
    let meals = fetch_meals().await?;
    // TODO: Display meals for mensa
    // TODO: More pizzazz
    print_meals(&meals);
    Ok(())
}

async fn fetch_meals() -> Result<Vec<Meal>> {
    let url = format!(
        "{}/canteens/{}/days/{}/meals",
        ENDPOINT,
        CONFIG.mensa_id(),
        CONFIG.date()
    );
    Ok(reqwest::get(url).await?.json().await?)
}

fn print_meals(meals: &[Meal]) {
    for meal in meals {
        meal.print_to_terminal();
        println!();
    }
}
