[![tests](https://github.com/MalteT/mensa/actions/workflows/rust.yml/badge.svg)](https://github.com/MalteT/mensa/actions/workflows/rust.yml)

# mensa

CLI tool to query the menu of canteens contained in the [OpenMensa](https://openmensa.org) database.

![example](https://user-images.githubusercontent.com/11077981/137278085-75ec877a-dba0-44bb-a8dc-6c802e24178c.png)

## Features

- [X] Custom filters and favourites using CLI flags or the
      optional configuration file.
- [X] List canteens close to you based on GeoIP.
- [X] All request are cached locally.
- [X] Fuzzy date parsing based on
      [date_time_parser](https://lib.rs/crates/date_time_parser).
- [ ] List your favourite meals in canteens close to your location.

## Installation

### Cargo

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

- `mensa` will show meals served today for the default canteen mentioned
  in the configuration.
  If no such configuration exists, try `mensa --id 63`.
  You can find the id for your canteen using
- `mensa canteens` lists canteens near you based on your current
  IP in a default radius of 10km.
- `mensa tags` will list the currently known meal tags like "**12** Nuts".


## Configuration

See [config.toml](config.toml) for an example. Copy the file to:
- `$XDG_CONFIG_DIR/mensa/config.toml` on **Linux**,
- `$HOME/Library/Application Support/mensa/config.toml` on **macOS**,
- ~~`{FOLDERID_RoamingAppData}\mensa\config.toml` on **Windows**~~
  I don't think it'll run on Windows.. 🤷‍♀️

License: MIT OR Apache-2.0
