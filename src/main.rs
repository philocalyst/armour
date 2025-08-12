use std::error::Error;

use svg::BadgerOptions;
use svg::badgen;

mod colors;
mod svg;

fn main() -> Result<(), Box<dyn Error>> {
    // Create the lua struct for managing lua state

    let badge = badgen(BadgerOptions {
        status: String::from("SYN"),
        label: Some(String::from("DOCS.RS")),
        ..BadgerOptions::default()
    })?;

    std::fs::write("test.svg", badge.to_string())?;

    Ok(())
}
