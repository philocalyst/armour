use std::fs;
use std::path::PathBuf;

use steel::steel_vm::engine::Engine;

fn main() {
    let scripts_dir = "src/plugins";

    println!("cargo:rerun-if-changed={}", scripts_dir);

    let combined = fs::read_dir(scripts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("scm"))
        .map(|e| {
            println!("cargo:rerun-if-changed={}", e.path().display());
            fs::read_to_string(e.path()).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("all-plugins.scm");
    fs::write(out_path, &combined).unwrap();

    generate_enum(&combined);
}

fn generate_enum(ast_fodder: &str) {
    let plugins_path = PathBuf::from(concat!(env!("OUT_DIR"), "/all-plugins.scm"));

    let mut engine = Engine::new();
    engine.with_contracts(true);

    let core = include_str!("./src/core.scm");

    // Now core is in the module system, so (require "core") resolves
    engine.register_steel_module("core".to_string(), core.to_string());

    // Go ahead and run so AST works??
    engine.run(ast_fodder).unwrap();

    // This expands core AND loads it into the module system as a side effect
    let core_ast = engine
        .emit_expanded_ast_without_optimizations(ast_fodder, Some(plugins_path))
        .unwrap();

    let define = query_top_level_define(&core_ast, "get-edition__doc__");
}
