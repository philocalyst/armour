use std::collections::HashMap;
use steel::SteelVal;
use steel::gc::Gc;
use steel::rvals::{IntoSteelVal, SteelHashMap};
use steel::steel_vm::register_fn::RegisterFn;

pub fn parse_toml(input: String) -> Result<HashMap<String, SteelVal>, String> {
    let value: toml::Value = toml::from_str(&input).map_err(|e| e.to_string())?;
    toml_to_steel(value)
}

pub fn toml_to_steel(value: toml::Value) -> Result<HashMap<String, SteelVal>, String> {
    match value {
        toml::Value::Table(table) => {
            let mut map = HashMap::new();
            for (k, v) in table {
                map.insert(k, toml_value_to_steelval(v)?);
            }
            Ok(map)
        }
        _ => Err("Top-level TOML must be a table".to_string()),
    }
}

pub fn toml_value_to_steelval(value: toml::Value) -> Result<SteelVal, String> {
    match value {
        toml::Value::String(s) => Ok(SteelVal::StringV(s.into())),
        toml::Value::Integer(i) => Ok(SteelVal::IntV(i as isize)),
        toml::Value::Float(f) => Ok(SteelVal::NumV(f)),
        toml::Value::Boolean(b) => Ok(SteelVal::BoolV(b)),
        toml::Value::Array(arr) => {
            let vals: Result<Vec<SteelVal>, _> =
                arr.into_iter().map(toml_value_to_steelval).collect();
            Ok(SteelVal::ListV(vals?.into()))
        }
        toml::Value::Table(table) => {
            let map: Result<HashMap<String, SteelVal>, _> = table
                .into_iter()
                .map(|(k, v)| toml_value_to_steelval(v).map(|v| (k, v)))
                .collect();
            map?.into_steelval().map_err(|e| e.to_string())
        }
        toml::Value::Datetime(dt) => Ok(SteelVal::StringV(dt.to_string().into())),
    }
}
