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
    let answer = steel_engine.run(
        "(+ 1 2 3 4)
         (+ 5 6 7 8)",
    );
    assert_eq!(answer, Ok(vec![SteelVal::IntV(10), SteelVal::IntV(26)]));

    let badge = badgen(BadgerOptions {
        status: String::from("SYN"),
        label: Some(String::from("DOCS.RS")),
        ..BadgerOptions::default()
    })?;

    std::fs::write("test.svg", badge.to_string())?;

    Ok(())
}
