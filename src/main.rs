use std::error::Error;
use std::fs::write;

use steel::SteelErr;
use steel::SteelVal;
use steel::rerrs::ErrorKind;
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use svg::BadgerOptions;
use svg::badgen;

use crate::badger::Badge;
use crate::badger::Globals;
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

    let config: badger::Config = toml::from_str(include_str!("../badger.toml"))?;

    process_badges(engine, config.badges, config.globals);

    Ok(())
}

#[derive(Debug)]
struct Entry {
    key: String,
    value: String,
    icon: Option<String>,
}

impl TryFrom<SteelVal> for Entry {
    type Error = SteelErr;

    fn try_from(val: SteelVal) -> Result<Self, Self::Error> {
        let items: Vec<SteelVal> = match val {
            SteelVal::ListV(list) => list.into_iter().collect(),
            _ => {
                return Err(SteelErr::new(
                    ErrorKind::TypeMismatch,
                    "expected a list".to_string(),
                ));
            }
        };

        Ok(Self {
            key: items[0].to_string(),
            value: items[1].to_string(),
            icon: items.get(2).map(|v| v.to_string()),
        })
    }
}
fn process_badges(mut engine: Engine, badges: Vec<Badge>, universal_options: Globals) {
    for badge in badges {
        let entry: Entry = engine
            .call_function_by_name_with_args(badge.producer.entry_point(), vec![])
            .unwrap()
            .try_into()
            .unwrap();

        badgen(BadgerOptions {
            primary_color: Some(badge.primary_color),
            secondary_color: Some(badge.secondary_color),
            label: Some(entry.key),
            status: entry.value,
            icon: None,
            scale: Some(universal_options.scale as f64),
        })
        .unwrap();
    }
}
