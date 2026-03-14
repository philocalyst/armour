use std::error::Error;
use std::fs::write;
use std::path::PathBuf;

use steel::SteelErr;
use steel::SteelVal;
use steel::compiler::passes::analysis::query_top_level_define;
use steel::parser::ast::ExprKind;
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
    let mut engine = Engine::new();
    engine.with_contracts(true);

    let core = include_str!("./core.scm");
    let plugins = include_str!(concat!(env!("OUT_DIR"), "/all-plugins.scm"));
    let plugins_path = PathBuf::from(concat!(env!("OUT_DIR"), "/all-plugins.scm"));

    // Now core is in the module system, so (require "core") resolves
    engine.register_steel_module("core".to_string(), core.to_string());

    engine.register_fn("parse-toml", parse_toml);
    engine.run(plugins)?;

    // This expands core AND loads it into the module system as a side effect
    let core_ast = engine.emit_expanded_ast_without_optimizations(plugins, Some(plugins_path))?;

    let define = query_top_level_define(&core_ast, "get-edition__doc__");
    let answer = engine.call_function_by_name_with_args("get-edition", vec![])?;

    let badge = badgen(BadgerOptions {
        status: answer.to_string(),
        label: Some("EDITION".to_string()),
        ..BadgerOptions::default()
    })?;

    write("test.svg", badge.to_string())?;

    Ok(())
}
