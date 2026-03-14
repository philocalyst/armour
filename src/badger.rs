use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) globals: Globals,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct Globals {
    pub(crate) scale: u32,
}
