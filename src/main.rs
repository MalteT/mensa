use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    // TODO: Enable logger
    // TODO: Read config
    // TODO: Read args
    // TODO: Fetch meals for mensa
    // TODO: Display meals for mensa
    Ok(())
}

#[derive(Debug, Error)]
enum Error {

}
