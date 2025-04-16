use common::logging;
use log::LevelFilter;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::str::FromStr;
use thiserror::Error;

const CONFIG_FILENAME: &str = "config.toml";

#[derive(Debug, Clone)]
pub struct Config {
    pub compression: bool,
    pub interface: Option<String>,
    pub log_format: String,
    pub log_level: LevelFilter,
    pub password: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compression: true,
            interface: None,
            log_format: logging::DEFAULT_FORMAT.to_string(),
            log_level: LevelFilter::Info,
            password: String::new(),
            port: 8080,
        }
    }
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Config", 5)?;

        state.serialize_field("compression", &self.compression)?;

        if let Some(interface) = &self.interface {
            state.serialize_field("interface", interface)?;
        } else {
            state.serialize_field("interface", "none")?;
        }

        state.serialize_field("log_format", &self.log_format.to_string())?;
        state.serialize_field("log_level", &self.log_level.to_string())?;
        state.serialize_field("password", &self.password)?;
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

        toml::from_str::<ConfigDto>(&data.unwrap_or_default())
            .map_err(ConfigError::TomlDeserializationError)?
            .into_config()
    }

    pub fn save_to_file(&self) -> Result<(), ConfigError> {
        let data = toml::to_string(&self).map_err(ConfigError::TomlSerializationError)?;

        std::fs::write(CONFIG_FILENAME, data).map_err(ConfigError::IOError)?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct ConfigDto {
    compression: bool,
    interface: String,
    log_format: String,
    log_level: String,
    password: String,
    port: u16,
}

impl ConfigDto {
    pub fn into_config(self) -> Result<Config, ConfigError> {
        let interface = if self.interface.trim().eq("none") {
            None
        } else {
            Some(self.interface)
        };

        let config = Config {
            compression: self.compression,
            interface,
            log_format: self.log_format,
            log_level: LevelFilter::from_str(&self.log_level)
                .map_err(|_| ConfigError::UnknownLogLevel)?,
            password: self.password,
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

    #[error("Unknown log level.")]
    UnknownLogLevel,
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
