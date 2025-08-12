use std::error::Error;

use rhai_doc::{FunctionParam, FunctionReturn};
use svg::BadgerOptions;
use svg::badgen;

use rhai_doc::{FunctionDoc, RhaiDocBuilder};

use crate::rhai::RhaiType;

mod colors;
mod rhai;
mod svg;

fn main() -> Result<(), Box<dyn Error>> {
    let project = RhaiDocBuilder::new()
        .with_source_dir("/home/miles/Downloads/armour/test")
        .scan()?;

    let page = project.scripts().get(0).unwrap();

    let function_data: Vec<(Vec<FunctionParam>, Vec<FunctionReturn>)> = page
        .functions
        .iter()
        .filter_map(|func: &FunctionDoc| {
            if func.name == "greet" {
                return Some((func.params.clone(), func.returns.clone()));
            } else {
                return None;
            }
        })
        .collect();

    let mut args: Vec<rhai::RhaiType> = Vec::new();

    if function_data.len() > 1 {
        println!("There should only be one main function")
    } else {
        let main = function_data.get(0).unwrap();

        for param in main.0.clone() {
            args.push(RhaiType::from(param.type_name.unwrap().as_str()));
        }
    }

    println!("{:?}", args);

    let badge = badgen(BadgerOptions {
        status: String::from("SYN"),
        label: Some(String::from("DOCS.RS")),
        ..BadgerOptions::default()
    })?;

    std::fs::write("test.svg", badge.to_string())?;

    Ok(())
}
