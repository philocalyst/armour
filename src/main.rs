use std::error::Error;

use svg::BadgerOptions;
use svg::badgen;

mod colors;
mod parser;
mod svg;

fn main() -> Result<(), Box<dyn Error>> {
    // Create the lua struct for managing lua state

    let badge = badgen(BadgerOptions {
        status: String::from("."),
        label: Some(String::from("HIIIIIII")),
        ..BadgerOptions::default()
    })?;

    std::fs::write("test.svg", badge.to_string())?;

    Ok(())
}
