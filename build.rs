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

fn parser<'a>() -> impl Parser<'a, &'a str, Item, extra::Err<Rich<'a, char>>> {
    // An identifier: ASCII alphanumeric or underscore
    let ident = text::ascii::ident().map(|s: &str| s.to_string());

    // Optional type annotation in {braces}
    let ty = ident.delimited_by(just('{'), just('}')).padded().or_not();

    // Description: everything to end of line
    let desc = any()
        .filter(|c: &char| *c != '\n')
        .repeated()
        .collect::<String>()
        .map(|s| s.trim().to_string());

    let param = just("@param")
        .padded()
        .ignore_then(ident) // param name
        .then(ty) // optional {Type}
        .then(desc) // rest of line as description
        .map(|((name, param_type), description)| Param {
            name,
            param_type: param_type.map(|n| TypeExpr::Named(n)),
            description: Some(description),
            modifiers: vec![],
        });

    Item {
        name: "hi",
        doc: DocComment {
            summary: (),
            description: (),
            kind: (),
            name: (),
            location: (),
            params: (),
            returns: (),
            errors: (),
            raises: (),
            fields: (),
            see: (),
            usage: (),
            is_local: (),
            within: (),
            section: (),
            annotations: (),
            module_tags: (),
        },
        kind: ItemKind::Function,
        location: None,
    }
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
            let doc = query_top_level_define(&ast, &format!("{}__doc__", stem))
                .and_then(|node| Some(extract_quoted_string(&node.to_string()).to_string()));

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
    doc: Option<String>,
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

    fs::write(
        "out",
        plugins
            .iter()
            .map(|each| each.doc.clone().unwrap_or_default())
            .collect::<Vec<String>>()
            .join("\n"),
    );

    let doc_attrs: Vec<TokenStream> = plugins
        .iter()
        .map(|p| match &p.doc {
            Some(doc) => quote! { #[doc = #doc] },
            None => quote! {},
        })
        .collect();

    let entry_point_arms = variants
        .iter()
        .zip(entry_points.iter())
        .map(|(v, ep)| quote! { Producer::#v => #ep, });

    let doc_arms = variants
        .iter()
        .zip(plugins.iter())
        .map(|(v, p)| match &p.doc {
            Some(doc) => quote! { Producer::#v => Some(#doc), },
            None => quote! { Producer::#v => None, },
        });

    let try_from_arms = variants
        .iter()
        .zip(entry_points.iter())
        .map(|(v, ep)| quote! { #ep => Ok(Producer::#v), });

    quote! {
        #[derive(Debug, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Producer {
            #(#doc_attrs #variants,)*
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
