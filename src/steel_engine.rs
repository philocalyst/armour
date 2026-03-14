use steel::steel_vm::{engine::Engine, register_fn::RegisterFn};

use crate::{error::BadgerError, wrappers::toml::parse_toml};

pub(crate) fn setup() -> Result<Engine, BadgerError> {
    let mut engine = Engine::new();
    engine.with_contracts(true);

    let core = include_str!("./core.scm");
    let plugins = include_str!(concat!(env!("OUT_DIR"), "/all-plugins.scm"));

    engine.register_steel_module("core".to_string(), core.to_string());
    engine.register_fn("parse-toml", parse_toml);
    engine.run(plugins)?;

    Ok(engine)
}
