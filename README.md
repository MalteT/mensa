<img src="https://raw.githubusercontent.com/MalteT/mensa/main/static/logo.svg?sanitize=true" alt="mensa CLI logo" width="400" align="right">

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


## Configuration

See [config.toml](config.toml) for an example. Copy the file to:
- `$XDG_CONFIG_DIR/mensa/config.toml` on **Linux**,
- `$HOME/Library/Application Support/mensa/config.toml` on **macOS**,
- `{FOLDERID_RoamingAppData}\mensa\config.toml` on **Windows**

License: MIT
