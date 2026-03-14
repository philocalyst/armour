use std::error::Error;

use steel::SteelVal;
use steel::steel_vm::engine::Engine;
use svg::BadgerOptions;
use svg::badgen;

mod colors;
mod rhai;
mod svg;

fn main() -> Result<(), Box<dyn Error>> {
    let mut steel_engine = Engine::new();
    steel_engine.with_contracts(true);

    steel_engine.register_steel_module("core".to_string(), include_str!("./core.scm").to_string());

    let answer = steel_engine.run("(require \"core\")(my-fun 1 2 3)");

    let badge = badgen(BadgerOptions {
        status: String::from("SYN"),
        label: Some(String::from("DOCS.RS")),
        ..BadgerOptions::default()
    })?;

    std::fs::write("test.svg", badge.to_string())?;

    Ok(())
}
