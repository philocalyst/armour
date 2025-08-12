use std::error::Error;

use svg::BadgerOptions;
use svg::badgen;

use rhai_doc::{FunctionDoc, RhaiDocBuilder};

use crate::rhai::RhaiType;

mod colors;
mod rhai;
mod svg;

fn main() -> Result<(), Box<dyn Error>> {
    let project = RhaiDocBuilder::new()
        .with_source_dir("/home/miles/Downloads/armour/test")
        .scan()?;

    let page = project.scripts().get(0).unwrap();

    let badge = badgen(BadgerOptions {
        status: String::from("SYN"),
        label: Some(String::from("DOCS.RS")),
        ..BadgerOptions::default()
    })?;

    std::fs::write("test.svg", badge.to_string())?;

    Ok(())
}
