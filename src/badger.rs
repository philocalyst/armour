use serde::Deserialize;

use crate::Producer;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    globals: Globals,
    #[serde(rename = "badge")]
    badges: Vec<Badge>,
}

#[derive(Debug, Deserialize, Default)]
struct Globals {
    scale: u32,
}

#[derive(Debug, Deserialize)]
struct Badge {
    primary_color: String,
    secondary_color: String,
    producer: Producer,
}
