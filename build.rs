use chumsky::prelude::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::fs;
use std::path::PathBuf;
use steel::compiler::passes::analysis::query_top_level_define;
use steel::steel_vm::engine::Engine;
use toml;

fn extract_quoted_string(input: &str) -> &str {
    let start = input.find('"').unwrap() + 1;
    let end = input[start..].rfind('"').unwrap() + start;
    &input[start..end]
}

include!("./src/documentation.rs");

use chumsky::prelude::*;

fn parser<'a>() -> impl Parser<'a, &'a str, Item, extra::Err<Rich<'a, char>>> {
    let ident = text::ascii::ident().map(|s: &str| s.to_string());

    // rest of line (trimmed), stops before \n
    let rest_of_line = any()
        .filter(|c: &char| *c != '\n')
        .repeated()
        .collect::<String>()
        .map(|s| s.trim().to_string());

    // optional {TypeName} — only handles simple Named types for now
    let ty = ident
        .clone()
        .delimited_by(just('{'), just('}'))
        .padded()
        .map(TypeExpr::Named)
        .or_not();

    // The very first non-tag, non-empty line is the summary.

    let summary = rest_of_line
        .clone()
        .filter(|s| !s.is_empty() && !s.starts_with('@'));

    // Zero or more non-tag lines after the summary.

    let desc_line = rest_of_line
        .clone()
        .filter(|s: &String| !s.starts_with('@'))
        .then_ignore(just('\n').or_not());

    let description = desc_line
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|lines| lines.join("\n"))
        .map(|s| s.trim().to_string())
        .or_not();

    let param = just("@param")
        .then(just(' ').repeated().at_least(1))
        .ignore_then(ident.clone()) // name
        .then(ty) // optional {Type}
        .then_ignore(just(' ').repeated())
        .then(rest_of_line.clone().or_not()) // description
        .then_ignore(just('\n').or_not())
        .map(|((name, param_type), description)| Param {
            name,
            param_type,
            description: description.filter(|s| !s.is_empty()),
            modifiers: vec![],
        });

    let ret = just("@return")
        .then(just(' ').repeated())
        .ignore_then(rest_of_line.clone().or_not())
        .then_ignore(just('\n').or_not())
        .map(|description| Return {
            description: description.filter(|s| !s.is_empty()),
            return_type: None,
            group: None,
        });

    let see = just("@see")
        .then(just(' ').repeated().at_least(1))
        .ignore_then(rest_of_line.clone())
        .then_ignore(just('\n').or_not())
        .map(|target| See {
            reference: Ref {
                target,
                display: None,
            },
        });

    #[derive(Debug)]
    enum Tag {
        Param(Param),
        Return(Return),
        See(See),
    }

    let tag = choice((
        param.map(Tag::Param),
        ret.map(Tag::Return),
        see.map(Tag::See),
    ));

    // A full document comment block
    // Layout:
    //   <summary>
    //   [<description lines>]
    //   [@tag ...]*

    summary
        .then_ignore(just('\n'))
        .then(description)
        .then(tag.repeated().collect::<Vec<_>>())
        .map(|((summary, description), tags)| {
            let mut doc = DocComment {
                summary,
                description,
                ..DocComment::default()
            };

            for tag in tags {
                match tag {
                    Tag::Param(p) => doc.params.push(p),
                    Tag::Return(r) => doc.returns.push(r),
                    Tag::See(s) => doc.see.push(s),
                }
            }

            Item {
                name: doc.name.clone().unwrap_or_default(),
                doc,
                kind: ItemKind::Function,
                location: None,
            }
        })
}

include!("./src/wrappers/toml.rs");
fn main() {
    let scripts_dir = "src/plugins";
    println!("cargo:rerun-if-changed={}", scripts_dir);

    let mut stems = Vec::new();
    let mut combined_parts = Vec::new();

    for entry in fs::read_dir(scripts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("scm"))
    {
        let path = entry.path();
        println!("cargo:rerun-if-changed={}", path.display());
        stems.push(path.file_stem().unwrap().to_string_lossy().into_owned());
        combined_parts.push(fs::read_to_string(&path).unwrap());
    }

    let combined = combined_parts.join("\n");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let plugins_path = out_dir.join("all-plugins.scm");

    fs::write(&plugins_path, &combined).unwrap();

    let mut engine = Engine::new();
    engine.with_contracts(true);
    engine.register_steel_module(
        "core".to_string(),
        include_str!("./src/core.scm").to_string(),
    );

    engine.register_fn("parse-toml", parse_toml);

    engine.run(combined.clone()).unwrap();

    let ast = engine
        .emit_expanded_ast_without_optimizations(&combined, Some(plugins_path))
        .unwrap();

    let plugins: Vec<PluginInfo> = stems
        .into_iter()
        .map(|stem| {
            let doc = query_top_level_define(&ast, &format!("{}__doc__", stem)).and_then(|node| {
                let node = node.to_string();
                Some(
                    parser()
                        .parse(extract_quoted_string(node.as_ref()))
                        .into_result()
                        .unwrap(),
                )
            });

            PluginInfo {
                entry_point: stem,
                doc,
            }
        })
        .collect();

    fs::write(
        out_dir.join("producers.rs"),
        generate_enum(&plugins).to_string(),
    )
    .unwrap();
}

struct PluginInfo {
    entry_point: String,
    doc: Option<Item>,
}

fn to_variant_name(entry_point: &str) -> String {
    entry_point
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn generate_enum(plugins: &[PluginInfo]) -> TokenStream {
    let variants: Vec<Ident> = plugins
        .iter()
        .map(|p| Ident::new(&to_variant_name(&p.entry_point), Span::call_site()))
        .collect();

    let entry_points: Vec<&str> = plugins.iter().map(|p| p.entry_point.as_str()).collect();

    let entry_point_arms = variants
        .iter()
        .zip(entry_points.iter())
        .map(|(v, ep)| quote! { Producer::#v => #ep, });

    let doc_arms = variants
        .iter()
        .zip(plugins.iter())
        .map(|(v, p)| match &p.doc {
            Some(item) => {
                let doc = item.doc.summary.as_str();
                quote! { Producer::#v => Some(#doc), }
            }
            None => quote! { Producer::#v => None, },
        });

    let try_from_arms = variants
        .iter()
        .zip(entry_points.iter())
        .map(|(v, ep)| quote! { #ep => Ok(Producer::#v), });

    quote! {
        #[derive(Debug, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Producer {
            #( #variants,)*
        }

        impl Producer {
            pub fn entry_point(&self) -> &'static str {
                match self {
                    #(#entry_point_arms)*
                }
            }

            pub fn doc(&self) -> Option<&'static str> {
                match self {
                    #(#doc_arms)*
                }
            }
        }

        impl TryFrom<&str> for Producer {
            type Error = String;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                match s {
                    #(#try_from_arms)*
                    _ => Err(format!("unknown producer: {}", s)),
                }
            }
        }
    }
}
