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
    let mut steel_engine = Engine::new();
    steel_engine.with_contracts(true);

    let core = include_str!("./core.scm").to_string();
    let plugins = include_str!(concat!(env!("OUT_DIR"), "/all-plugins.scm"));

    let ast = Engine::emit_ast(&core);

    let mut expanded_ast: Vec<ExprKind> = steel_engine.emit_expanded_ast_without_optimizations(
        &core,
        Some(PathBuf::from("/Users/philocalyst/proj/armour/src/core.scm")),
    )?;

    let define = query_top_level_define(&expanded_ast, "my-fun__doc__");

    steel_engine.register_steel_module("core".to_string(), core);

    steel_engine.register_fn("parse-toml", parse_toml);

    steel_engine.run(plugins).unwrap();

    let answer = steel_engine.call_function_by_name_with_args("get-edition", vec![]);

    let badge = badgen(BadgerOptions {
        status: answer.unwrap().to_string(),
        label: Some("EDITION".to_string()),
        ..BadgerOptions::default()
    })?;

    write("test.svg", badge.to_string())?;

    Ok(())
}
