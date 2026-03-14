use std::error::Error;

use steel::SteelErr;
use steel::SteelVal;
use steel::rerrs::ErrorKind;
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use svg::BadgerOptions;
use svg::badgen;

use crate::toml::parse_toml;

mod colors;
mod rhai;
mod svg;
mod toml;

fn main() -> Result<(), Box<dyn Error>> {
    let mut steel_engine = Engine::new();
    steel_engine.with_contracts(true);

    steel_engine.register_steel_module("core".to_string(), include_str!("./core.scm").to_string());

    steel_engine.register_fn("parse-toml", parse_toml);

    steel_engine
        .run(include_str!(concat!(env!("OUT_DIR"), "/plugin.scm")))
        .unwrap();

    let answer = steel_engine.call_function_by_name_with_args("get-edition", vec![]);

    dbg!(&answer);

    let badge = badgen(BadgerOptions {
        status: String::from(String::from(answer.unwrap().to_string())),
        label: Some("EDITION".to_string()),
        ..BadgerOptions::default()
    })?;

    std::fs::write("test.svg", badge.to_string())?;

    Ok(())
}
