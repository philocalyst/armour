use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, format_commonmark, parse_document};

use steel::SteelErr;
use steel::SteelVal;
use steel::rerrs::ErrorKind;
use steel::steel_vm::engine::Engine;
use tracing::{info, warn};

use crate::badger::Globals;
use crate::error::ArmourError;
use crate::svg::{BadgerOptions, badgen};

use std::collections::HashMap;
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

    let badges_dir = Path::new("badges");
    fs::create_dir_all(badges_dir)?;

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.render.r#unsafe = true;
    options.render.escape = false;

    let root = parse_document(&arena, include_str!("../README.md"), &options);

    replace_badger_tags(root, &mut engine, &config.globals, badges_dir)?;

    let mut markdown = String::new();
    format_commonmark(root, &options, &mut markdown).unwrap();

    fs::write("out.md", markdown);
    Ok(())
}

fn parse_badger_attrs(html: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    // Strip the tag name: find first space after "<badger"
    let inner = match html.find("<badger") {
        Some(start) => {
            let rest = &html[start + "<badger".len()..];
            // Take everything up to '>' or '/>'
            let end = rest.find('>').unwrap_or(rest.len());
            &rest[..end]
        }
        None => return attrs,
    };

    // Simple attribute parser for key="value" pairs
    let mut chars = inner.chars().peekable();
    loop {
        // Skip whitespace
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        // Read key
        let key: String = chars
            .by_ref()
            .take_while(|c| *c != '=' && !c.is_whitespace())
            .collect();
        if key.is_empty() {
            break;
        }
        // Skip '='
        while chars.peek().is_some_and(|c| *c == '=') {
            chars.next();
        }
        // Skip opening quote
        let quote = match chars.peek() {
            Some('"') | Some('\'') => {
                let q = *chars.peek().unwrap();
                chars.next();
                q
            }
            _ => continue,
        };
        // Read value until closing quote
        let value: String = chars.by_ref().take_while(|c| *c != quote).collect();
        attrs.insert(key, value);
    }
    attrs
}

fn replace_badger_tags<'a>(
    node: &'a AstNode<'a>,
    engine: &mut Engine,
    globals: &Globals,
    badges_dir: &Path,
) -> Result<(), ArmourError> {
    for child in node.descendants() {
        let mut ast = child.data.borrow_mut();
        if let NodeValue::HtmlBlock(ref html) = ast.value.clone() {
            let literal = &html.literal;
            if literal.starts_with("<badger") {
                let attrs = parse_badger_attrs(literal);

                let producer_name = match attrs.get("producer") {
                    Some(name) => name.clone(),
                    None => {
                        warn!("skipping <badger> tag without producer attribute");
                        continue;
                    }
                };

                let producer: Producer = Producer::try_from(producer_name.as_str())
                    .map_err(|e| ArmourError::Config(e))?;

                let raw_entry: SteelVal =
                    engine.call_function_by_name_with_args(producer.entry_point(), vec![])?;

                let entry: Entry = raw_entry.try_into().map_err(ArmourError::Steel)?;

                info!(label = %entry.key, status = %entry.value, "generating badge");

                let primary_color = attrs
                    .get("primary_color")
                    .cloned()
                    .unwrap_or_else(|| "blue".to_string());
                let secondary_color = attrs
                    .get("secondary_color")
                    .cloned()
                    .unwrap_or_else(|| "green".to_string());

                let svg_doc = badgen(BadgerOptions {
                    primary_color: Some(primary_color),
                    secondary_color: Some(secondary_color),
                    label: Some(entry.key.clone()),
                    status: entry.value.clone(),
                    icon: None,
                    scale: Some(globals.scale as f64),
                })?;

                let filename = format!("{}.svg", producer_name);
                let svg_path = badges_dir.join(&filename);
                fs::write(&svg_path, svg_doc.to_string())?;

                info!(path = %svg_path.display(), "wrote badge SVG");

                let img_markdown = format!(
                    "![{}: {}](badges/{})\n",
                    entry.key, entry.value, filename
                );

                ast.value = NodeValue::HtmlBlock(comrak::nodes::NodeHtmlBlock {
                    block_type: 6,
                    literal: img_markdown,
                });
            }
        }
    }
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
