use ::svg::Document;
use base64::Engine as b64Engine;
use comrak::nodes::{AstNode, NodeHtmlBlock, NodeLink, NodeValue};
use comrak::{Arena, Options, format_commonmark, parse_document};

use steel::SteelErr;
use steel::SteelVal;
use steel::rerrs::ErrorKind;
use steel::steel_vm::engine::Engine;
use tracing::{info, instrument, warn};

use crate::badger::Badge;
use crate::badger::Globals;
use crate::error::ArmourError;
use crate::svg::{BadgerOptions, badgen};

use std::fs;
use std::io::Write;

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

    let badges = process_badges(&mut engine, &config.badges, &config.globals)?;

    let markdown = include_str!("../README.md");

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.render.r#unsafe = true;
    options.render.escape = false;

    let root = parse_document(&arena, include_str!("../README.md"), &options);

    replace_badger_tags(root, badges[0].clone());

    let mut markdown = String::new();
    format_commonmark(root, &options, &mut markdown).unwrap();

    fs::write("out.md", markdown);
    Ok(())
}

fn replace_badger_tags<'a>(node: &'a AstNode<'a>, svg: Document) {
    for child in node.descendants() {
        let mut ast = child.data.borrow_mut();
        if let NodeValue::HtmlBlock(html) = ast.value.clone() {
            let html = html.literal;
            if html.starts_with("<badger") {
                use base64::prelude::BASE64_STANDARD;
                let encoded_svg = BASE64_STANDARD.encode(svg.to_string());

                let data_uri = format!("data:image/svg+xml;base64,{}", encoded_svg);

                ast.value = NodeValue::Image(Box::new(NodeLink {
                    url: data_uri,
                    title: String::from("hi"),
                }));
            }
        }
    }
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
) -> Result<Vec<Document>, ArmourError> {
    let mut total_badges = Vec::new();
    for badge in badges {
        let raw_entry: SteelVal =
            engine.call_function_by_name_with_args(badge.producer.entry_point(), vec![])?;

        let entry: Entry = raw_entry.try_into().map_err(ArmourError::Steel)?;

        info!(label = %entry.key, status = %entry.value, "generating badge");

        total_badges.push(badgen(BadgerOptions {
            primary_color: Some(badge.primary_color.clone()),
            secondary_color: Some(badge.secondary_color.clone()),
            label: Some(entry.key),
            status: entry.value,
            icon: None,
            scale: Some(globals.scale as f64),
        })?);
    }

    Ok(total_badges)
}
