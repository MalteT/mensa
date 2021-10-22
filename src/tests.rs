//! TODO: These tests fail when run via `nix flake check`

use std::time::Duration;

use assert_cmd::Command;

#[test]
pub fn cmd_mensa_meals() {
    Command::cargo_bin("mensa")
        .unwrap()
        // Prevent loading the config
        .args(&["--config", "/does/not/exist"])
        // Show meals
        .arg("meals")
        // Use canteen id 1
        .args(&["--id", "1"])
        .timeout(Duration::from_secs(10))
        .assert()
        .success();
}

#[test]
pub fn cmd_mensa_meals_json() {
    Command::cargo_bin("mensa")
        .unwrap()
        // Prevent loading the config
        .args(&["--config", "/does/not/exist"])
        // Show meals
        .arg("meals")
        // Use canteen id 1
        .args(&["--id", "1"])
        .arg("--json")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();
}

#[test]
pub fn cmd_mensa_canteens() {
    Command::cargo_bin("mensa")
        .unwrap()
        // Prevent loading the config
        .args(&["--config", "/does/not/exist"])
        // Show meals
        .arg("canteens")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();
}

#[test]
pub fn cmd_mensa_canteens_json() {
    Command::cargo_bin("mensa")
        .unwrap()
        // Prevent loading the config
        .args(&["--config", "/does/not/exist"])
        // Show meals
        .arg("canteens")
        .arg("--json")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();
}

#[test]
pub fn cmd_mensa_tags() {
    Command::cargo_bin("mensa")
        .unwrap()
        // Prevent loading the config
        .args(&["--config", "/does/not/exist"])
        // Show tags
        .arg("tags")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();
}

#[test]
pub fn cmd_mensa_tags_json() {
    Command::cargo_bin("mensa")
        .unwrap()
        // Prevent loading the config
        .args(&["--config", "/does/not/exist"])
        // Show tags
        .arg("tags")
        .arg("--json")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();
}
