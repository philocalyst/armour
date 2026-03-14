use std::error::Error;
use std::fs::write;

use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use svg::BadgerOptions;
use svg::badgen;

use crate::wrappers::toml::parse_toml;

mod badger;
mod colors;
mod documentation;
mod svg;
mod wrappers;

include!(concat!(env!("OUT_DIR"), "/producers.rs"));

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::new();
    engine.with_contracts(true);

    let core = include_str!("./core.scm");
    let plugins = include_str!(concat!(env!("OUT_DIR"), "/all-plugins.scm"));

    // Now core is in the module system, so (require "core") resolves
    engine.register_steel_module("core".to_string(), core.to_string());

    engine.register_fn("parse-toml", parse_toml);
    engine.run(plugins)?;

    let answer = engine.call_function_by_name_with_args("get-edition", vec![])?;

    let config: badger::Config = toml::from_str(include_str!("../badger.toml"))?;

    let badge = badgen(BadgerOptions {
        status: answer.to_string(),
        label: Some("EDITION".to_string()),
        primary_color: Some("blue".to_string()),
        secondary_color: Some("purple".to_string()),
        ..BadgerOptions::default()
    })?;

    write("test.svg", badge.to_string())?;

    Ok(())
}
