use crate::config::ConfigError::BadLogLevel;
use log::LevelFilter;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::str::FromStr;
use thiserror::Error;

const CONFIG_FILENAME: &str = "config.toml";

pub struct Config {
    pub log_level: LevelFilter,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
            port: 8080,
        }
    }
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Config", 2)?;
        state.serialize_field("log_level", &self.log_level.to_string())?;
        state.serialize_field("port", &self.port)?;
        state.end()
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

        let dto: Result<ConfigDto, ConfigError> =
            toml::from_str(&data.unwrap_or_default())
                .map_err(ConfigError::TomlDeserializationError);
        dto?.to_config()
    }

    pub fn save_to_file(&self) -> Result<(), ConfigError> {
        let data = toml::to_string(&self).map_err(ConfigError::TomlSerializationError)?;

        std::fs::write(CONFIG_FILENAME, data).map_err(ConfigError::IOError)?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct ConfigDto {
    log_level: String,
    port: u16,
}

impl ConfigDto {
    pub fn to_config(&self) -> Result<Config, ConfigError> {
        let config = Config {
            log_level: LevelFilter::from_str(&self.log_level).map_err(|_| BadLogLevel)?,
            port: self.port,
        };

        Ok(config)
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

    #[error("Bad log level.")]
    BadLogLevel,
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
