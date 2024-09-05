use serde::{ser::Serializer, Serialize};
use std::{num::ParseIntError, string::FromUtf8Error};
use zip::result::ZipError;

pub type HunterResult<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    IntParse(#[from] ParseIntError),
    #[cfg(target_os = "macos")]
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    URLParse(#[from] url::ParseError),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    Registry(#[from] windows_result::Error),
    #[error(transparent)]
    StringFromUTF8(#[from] FromUtf8Error),
    #[error("toml: {0}")]
    Toml(String),
    #[error(transparent)]
    SerdeJSON(#[from] serde_json::Error),
    #[error(transparent)]
    Zip(#[from] ZipError),
    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),
    #[error(transparent)]
    Update(#[from] tauri_plugin_updater::Error),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Var(#[from] std::env::VarError),
    #[error("{0}")]
    Command(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Other(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl Into<Error> for &str {
    fn into(self) -> Error {
        Error::Other(self.to_string())
    }
}
