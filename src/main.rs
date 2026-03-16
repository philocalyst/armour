use steel::SteelErr;
use steel::SteelVal;
use steel::rerrs::ErrorKind;
use steel::steel_vm::engine::Engine;
use tracing::{info, instrument, warn};

use crate::badger::{Badge, Globals};
use crate::error::BadgerError;
use crate::svg::{BadgerOptions, badgen};

use std::fs;
use std::path::Path;

mod badger;
mod colors;
mod documentation;
mod error;
mod steel_engine;
mod svg;
mod wrappers;

include!(concat!(env!("OUT_DIR"), "/producers.rs"));

fn main() -> Result<(), BadgerError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("starting armour badge generator");
    let mut engine = steel_engine::setup()?;

    let config: badger::Config = toml::from_str(include_str!("../badger.toml"))?;

    let badges_dir = Path::new("badges");
    fs::create_dir_all(badges_dir)?;

    let img_tags = process_badges(&mut engine, &config.badges, &config.globals, badges_dir)?;

    let markdown = fs::read_to_string("README.md")?;
    let updated = replace_badge_placeholder(&markdown, &img_tags);
    fs::write("README.md", updated)?;

    Ok(())
}

/// For each badge, generate the SVG file and return the image tags in order.
#[instrument(skip_all, fields(badge_count = badges.len()))]
fn process_badges(
    engine: &mut Engine,
    badges: &[Badge],
    globals: &Globals,
    badges_dir: &Path,
) -> Result<Vec<String>, BadgerError> {
    let mut img_tags = Vec::new();

    for badge in badges {
        let raw_entry: SteelVal =
            engine.call_function_by_name_with_args(badge.producer.entry_point(), vec![])?;

        let entry: Entry = raw_entry.try_into().map_err(BadgerError::Steel)?;

        info!(id = %badge.id.clone().unwrap_or("NONE".to_string()), label = %entry.key, status = %entry.value, "generating badge");

        let svg_doc = badgen(BadgerOptions {
            primary_color: Some(&badge.primary_color),
            secondary_color: Some(&badge.secondary_color),
            label: Some(entry.key.clone().trim_matches('"')),
            status: entry.value.clone().trim_matches('"'),
            icon: None,
            scale: Some(globals.scale as f64),
        })?;

        let filename = format!(
            "{}.svg",
            badge
                .id
                .clone()
                .unwrap_or(badge.producer.entry_point().to_string())
        );
        let svg_path = badges_dir.join(&filename);
        fs::write(&svg_path, svg_doc.to_string())?;

        info!(path = %svg_path.display(), "wrote badge SVG");

        img_tags.push(format!(
            "![{}: {}](badges/{})",
            entry.key, entry.value, filename
        ));
    }

    Ok(img_tags)
}

/// Find the single `<div badges="true">...</div>` and replace its inner content
/// with all generated badge image links, preserving the wrapper div so users can move it.
fn replace_badge_placeholder(markdown: &str, img_tags: &[String]) -> String {
    let open_tag = "<div badges=\"true\">";
    let close_tag = "</div>";

    let Some(open_start) = markdown.find(open_tag) else {
        warn!("no <div data-badger> found in markdown");
        return markdown.to_string();
    };

    let after_open = open_start + open_tag.len();

    let Some(close_offset) = markdown[after_open..].find(close_tag) else {
        warn!("found <div data-badger> but no closing </div>");
        return markdown.to_string();
    };

    let close_start = after_open + close_offset;
    let after_close = close_start + close_tag.len();

    let inner = img_tags.join("\n");

    info!("updated badge placeholder with {} badge(s)", img_tags.len());

    format!(
        "{}{}\n{}\n{}{}",
        &markdown[..open_start],
        open_tag,
        inner,
        close_tag,
        &markdown[after_close..],
    )
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
