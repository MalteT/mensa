# Mensa

CLI tool to query the menu of cafeterias in Leipzig that are listed [here](https://www.studentenwerk-leipzig.de/mensen-cafeterien/speiseplan).

# Usage

Use [Rust](https://www.rust-lang.org/)s build tool [Cargo](https://crates.io/) to build and, optionally, install this program:
```
 > cargo build --release
 > cargo install --path . --force
```

```
mensa [ at LOCATION ]
      [ no [ fish | pig | alcohol ] ]
      [ vegan | veggie | vegetarian | flexible ]
      [ on [ today | tomorrow | yyyy-mm-dd ]
```
/A complete syntax can be found [here](/search_format.pest)./

# TODO

```
src/meal.rs
44:    /// - FIXME: No food on sundays!
45:    /// - FIXME: Refactor
194:/// TODO: Badges
195:/// TODO: Price

```

# Bugs

This piece of software contains carefully selected bugs. If some of these offend you or keep you from finding the right place to eat lunch, please do report them. *This is an alpha.*
