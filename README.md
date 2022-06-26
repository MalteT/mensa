<img src="https://raw.githubusercontent.com/MalteT/mensa/main/static/logo.png" alt="mensa CLI logo" width="400" align="right">

[![tests](https://github.com/MalteT/mensa/actions/workflows/rust.yml/badge.svg)](https://github.com/MalteT/mensa/actions/workflows/rust.yml)


# mensa

CLI tool to query the menu of canteens contained in the
[OpenMensa](https://openmensa.org) database.

## Features

- [X] Runs on Linux, macOS and Windows.
- [X] Custom filters and favourites using CLI flags or the
      optional configuration file.
- [X] List canteens close to you based on GeoIP.
- [X] All request are cached locally.
- [X] Fuzzy date parsing based on
      [date_time_parser](https://lib.rs/crates/date_time_parser).
- [X] List your favourite meals in canteens close to your location.
- [X] JSON Output

![example](https://raw.githubusercontent.com/MalteT/mensa/main/static/example-collection.png)


## Installation

### Cargo

**Only nightly Rust supported at the moment**.

```console
$ cargo install --git https://github.com/MalteT/mensa
```

### Nix

This is a [Nix Flake](https://nixos.wiki/wiki/Flakes), add it
to your configuration or just test the application with:

```console
$ nix run github:MalteT/mensa
```


## Usage

See `mensa --help`.

- `mensa meals` will show meals served today for the default canteen
  mentioned in the configuration.
  If no such configuration exists, try `mensa meals --id 63`.
  You can find the id for your canteen using
- `mensa canteens` lists canteens near you based on your current
  IP in a default radius of 10km.
- `mensa tags` will list the currently known meal tags like "**12** Nuts".

### Examples

####
<details>
  <summary><b>Meals on monday</b> (<i>Click me!</i>)</summary>

  You can omit the `-i/--id` if you've configured a default id in the config.toml.

  ```console
  $ mensa meals -d mon -i 63

   Leipzig, Mensa am Park
   ┊
   ┊ ╭───╴Bohnengemüse
   ┊ ├─╴Gemüsebeilage 🌱
   ┊ ╰╴( 0.55€ )
   ...
  ```
</details>

<details>
  <summary><b>Canteens near your location</b> (<i>Click me!</i>)</summary>

  ```console
  $ mensa canteens

  70 Leipzig, Cafeteria Dittrichring
     Dittrichring 21, 04109 Leipzig

  63 Leipzig, Mensa am Park
     Universitätsstraße 5, 04109 Leipzig
  ...
  ```
</details>

<details>
  <summary><b>All currently known tags</b> (<i>Click me!</i>)</summary>

  ```console
  $ mensa tags

     0 Acidifier
       Contains artificial acidifier

     1 Alcohol
       Contains alcohol

     2 Antioxidant
       Contains an antioxidant
    ...
  ```
</details>

<details>
  <summary><b>Meals of canteens close to your location next sunday</b> (<i>Click me!</i>)</summary>

  ```console
  $ mensa meals close --date sun

   Leipzig, Cafeteria Dittrichring
   ┊
   ┊ ╭───╴Vegetarisch gefüllte Zucchini
   ┊ ├─╴Vegetarisches Gericht 🧀
   ┊ ├╴Rucola-Kartoffelpüree
   ┊ ├╴Tomaten-Ratatouille-Soße
   ┊ ╰╴( 2.65€ )  2 11 12 19

   Leipzig, Mensa am Park
   ┊
   ┊ ╭───╴Apfelrotkohl
   ┊ ├─╴Gemüsebeilage 🌱
   ┊ ╰╴( 0.55€ )  2
   ...
  ```
</details>

<details>
  <summary><b>Count OpenMensa's canteens</b> (<i>Click me!</i>)</summary>

  ```console
  $ mensa canteens --all --json | jq '.[].id' | wc -l
  704
  ```
</details>

## Configuration *(Optional)*

See [config.toml](config.toml) for an example. Copy the file to:
- `$XDG_CONFIG_DIR/mensa/config.toml` on **Linux**,
- `$HOME/Library/Application Support/mensa/config.toml` on **macOS**,
- `{FOLDERID_RoamingAppData}\mensa\config.toml` on **Windows**

License: MIT
