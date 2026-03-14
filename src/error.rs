use thiserror::Error;

#[derive(Debug, Error)]
pub enum BadgerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("Font parsing error: {0}")]
    FontParse(String),

    #[error("Steel engine error: {0}")]
    Steel(#[from] steel::SteelErr),

    #[error("SVG generation error: {0}")]
    Svg(String),

    #[error("Config error: {0}")]
    Config(String),
}

pub type BadgerResult<T> = std::result::Result<T, BadgerError>;
