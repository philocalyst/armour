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

fn file_to_string(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut steel_engine = Engine::new();
    steel_engine.with_contracts(true);

    steel_engine.register_steel_module("core".to_string(), include_str!("./core.scm").to_string());

    steel_engine.register_fn("parse-toml", parse_toml);
    steel_engine.register_fn("file->string", file_to_string);

    let answer = steel_engine.run(
        "(hash-try-get (hash-try-get (parse-toml (file->string (car (read-dir \".\")))) \"package\") \"edition\")",
    );

    let result = steel_engine
        .call_function_by_name_with_args("(read-dir", vec![SteelVal::StringV(".".into())]);

    let badge = badgen(BadgerOptions {
        status: String::from(String::from(answer.unwrap()[0].to_string())),
        label: Some("EDITION".to_string()),
        ..BadgerOptions::default()
    })?;

    std::fs::write("test.svg", badge.to_string())?;

    Ok(())
}
