use std::str::FromStr;

use rhai::{Array, Dynamic, Engine, Map, Scope};
/// Maps Rhai type strings to a conceptual validation strategy.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum RhaiType {
    Number,
    String,
    Bool,
    Dynamic, // Catch-all for unknown or flexible types
    Char,
    Unknown,
    Float,
}

impl From<&str> for RhaiType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "i64" | "i32" | "int" | "integer" => RhaiType::Number,
            "f32" | "f64" | "float" | "double" => RhaiType::Float,
            "string" | "str" => RhaiType::String,
            "bool" | "boolean" => RhaiType::Bool,
            "dynamic" | "any" => RhaiType::Dynamic,
            "letter" | "char" => RhaiType::Char,
            _ => RhaiType::Unknown,
        }
    }
}

/// Validates and converts a string argument based on expected RhaiType.
fn parse_arg_to_dynamic(
    arg_name: &str,
    arg_value_str: &str,
    expected_type: &RhaiType,
) -> Result<Dynamic, String> {
    match expected_type {
        RhaiType::Number => arg_value_str
            .parse::<i64>()
            .map(Dynamic::from_int)
            .map_err(|e| {
                format!(
                    "Invalid value for argument '{}': '{}' is not an integer. ({})",
                    arg_name, arg_value_str, e
                )
            }),
        RhaiType::Float => arg_value_str
            .parse::<f64>()
            .map(Dynamic::from_float)
            .map_err(|e| {
                format!(
                    "Invalid value for argument '{}': '{}' is not a float. ({})",
                    arg_name, arg_value_str, e
                )
            }),
        RhaiType::String => Ok(Dynamic::from_str(arg_value_str).unwrap()),
        RhaiType::Bool => arg_value_str
            .to_lowercase()
            .parse::<bool>()
            .map(Dynamic::from_bool)
            .map_err(|e| {
                format!(
                    "Invalid value for argument '{}': '{}' is not a boolean. ({})",
                    arg_name, arg_value_str, e
                )
            }),
        RhaiType::Dynamic | RhaiType::Unknown => {
            // For dynamic or unknown types, we'll try to guess
            Ok(Dynamic::from_str(arg_value_str).unwrap())
        }
        RhaiType::Char => arg_value_str
            .parse::<char>()
            .map(Dynamic::from_char)
            .map_err(|e| format!("Not a char{}", e)),
    }
}
