use steel::SteelErr;
use steel::SteelVal;
use steel::rerrs::ErrorKind;
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use tracing::{info, instrument, warn};

use crate::badger::Badge;
use crate::badger::Globals;
use crate::error::ArmourError;
use crate::steel_engine::setup;
use crate::svg::{BadgerOptions, badgen};
use crate::wrappers::toml::parse_toml;

mod badger;
mod colors;
mod documentation;
mod error;
mod steel_engine;
mod svg;
mod wrappers;

include!(concat!(env!("OUT_DIR"), "/producers.rs"));

fn main() -> Result<(), ArmourError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("starting armour badge generator");
    let mut engine = steel_engine::setup()?;

    let config: badger::Config = toml::from_str(include_str!("../badger.toml"))?;

    process_badges(&mut engine, &config.badges, &config.globals)?;

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

    fn try_from(val: SteelVal) -> std::result::Result<Self, Self::Error> {
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

#[instrument(skip_all, fields(badge_count = badges.len()))]
fn process_badges(
    engine: &mut Engine,
    badges: &[Badge],
    globals: &Globals,
) -> Result<(), ArmourError> {
    for badge in badges {
        let raw_entry: SteelVal =
            engine.call_function_by_name_with_args(badge.producer.entry_point(), vec![])?;

        let entry: Entry = raw_entry.try_into().map_err(ArmourError::Steel)?;

        info!(label = %entry.key, status = %entry.value, "generating badge");

        badgen(BadgerOptions {
            primary_color: Some(badge.primary_color.clone()),
            secondary_color: Some(badge.secondary_color.clone()),
            label: Some(entry.key),
            status: entry.value,
            icon: None,
            scale: Some(globals.scale as f64),
        })?;
    }

    Ok(())
}
