use serde::Deserialize;

use crate::Producer;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) globals: Globals,
    #[serde(rename = "badge")]
    pub(crate) badges: Vec<Badge>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct Globals {
    pub(crate) scale: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Badge {
    pub(crate) id: Option<String>,
    pub(crate) primary_color: String,
    pub(crate) secondary_color: String,
    pub(crate) producer: Producer,
}
