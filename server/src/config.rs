use crate::config::ConfigError::WrongLogLevel;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

const CONFIG_FILENAME: &str = "config.toml";

const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_PORT: u16 = 8080;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub log_level: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            port: DEFAULT_PORT,
        }
    }
}

impl Config {
    pub fn from_file() -> Result<Self, ConfigError> {
        let data = std::fs::read_to_string(CONFIG_FILENAME);
        if data.is_err() {
            let config = Config::default();
            config.save_to_file()?;
            return Ok(config);
        }

        toml::from_str(&data.unwrap_or_default())
            .map_err(ConfigError::TomlDeserializationError)
    }

    pub fn save_to_file(&self) -> Result<(), ConfigError> {
        let data = toml::to_string(&self).map_err(ConfigError::TomlSerializationError)?;

        std::fs::write(CONFIG_FILENAME, data).map_err(ConfigError::IOError)?;

        Ok(())
    }
}

impl Config {
    pub fn log_level(&self) -> Result<LevelFilter, ConfigError> {
        LevelFilter::from_str(&self.log_level).map_err(|_| WrongLogLevel)
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO Error.")]
    IOError(#[from] std::io::Error),

    #[error("TOML Serialization Error.")]
    TomlSerializationError(#[from] toml::ser::Error),

    #[error("TOML Deserialization Error.")]
    TomlDeserializationError(#[from] toml::de::Error),

    #[error("Wrong log level.")]
    WrongLogLevel,
}

impl ConfigError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            ConfigError::IOError(err) => Some(err.to_string()),
            ConfigError::TomlSerializationError(err) => Some(err.to_string()),
            ConfigError::TomlDeserializationError(err) => Some(err.to_string()),
            _ => None,
        }
    }
}
