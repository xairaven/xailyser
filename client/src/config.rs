use crate::communication::heartbeat;
use crate::ui::themes::ThemePreference;
use common::logging;
use directories::ProjectDirs;
use log::LevelFilter;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fs};
use thiserror::Error;

const CONFIG_FILENAME: &str = "config.toml";

#[derive(Debug, Clone)]
pub struct Config {
    pub compression: bool,
    pub log_format: String,
    pub log_level: LevelFilter,
    pub theme: ThemePreference,
    pub sync_delay_seconds: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compression: true,
            log_format: logging::DEFAULT_FORMAT.to_string(),
            log_level: LevelFilter::Info,
            theme: ThemePreference::default(),

            sync_delay_seconds: heartbeat::DEFAULT_PING_DELAY_SECONDS,
        }
    }
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Config", 3)?;
        state.serialize_field("compression", &self.compression)?;
        state.serialize_field("log_format", &self.log_format.to_string())?;
        state.serialize_field("log_level", &self.log_level.to_string())?;
        state.serialize_field("theme", &self.theme.to_string())?;
        state.serialize_field("sync_delay_seconds", &self.sync_delay_seconds)?;
        state.end()
    }
}

impl Config {
    pub fn from_file() -> Result<Self, ConfigError> {
        match Self::get_config_path() {
            Ok(path) => {
                let data = fs::read_to_string(path);
                if data.is_err() {
                    let config = Config::default();
                    config.save_to_file()?;
                    return Ok(config);
                }

                let dto: Result<ConfigDto, ConfigError> =
                    toml::from_str(&data.unwrap_or_default())
                        .map_err(ConfigError::TomlDeserializationError);
                dto?.into_config()
            },
            Err(_) => Ok(Config::default()),
        }
    }

    pub fn save_to_file(&self) -> Result<(), ConfigError> {
        let data = toml::to_string(&self).map_err(ConfigError::TomlSerializationError)?;

        let path = Self::get_config_path()?;
        if let Some(parent_path) = path.parent() {
            fs::create_dir_all(parent_path).map_err(ConfigError::IOError)?;
        }
        fs::write(path, data).map_err(ConfigError::IOError)?;

        Ok(())
    }

    fn get_config_path() -> Result<PathBuf, ConfigError> {
        let dirs = ProjectDirs::from("dev", "xairaven", "xailyser");
        match dirs {
            None => Ok(Self::get_current_directory()?),
            Some(value) => Ok(value.config_dir().join(CONFIG_FILENAME)),
        }
    }

    fn get_current_directory() -> Result<PathBuf, ConfigError> {
        let mut current_dir = env::current_dir().map_err(ConfigError::IOError)?;
        current_dir.push(CONFIG_FILENAME);
        Ok(current_dir)
    }
}

#[derive(Deserialize)]
struct ConfigDto {
    compression: bool,
    log_format: String,
    log_level: String,
    theme: String,
    sync_delay_seconds: i64,
}

impl ConfigDto {
    pub fn into_config(self) -> Result<Config, ConfigError> {
        let config = Config {
            compression: self.compression,
            log_format: self.log_format.trim().to_string(),
            log_level: LevelFilter::from_str(self.log_level.to_ascii_lowercase().trim())
                .map_err(|_| ConfigError::UnknownLogLevel)?,
            theme: ThemePreference::from_str(self.theme.to_ascii_lowercase().trim())
                .map_err(|_| ConfigError::UnknownTheme)?,
            sync_delay_seconds: self.sync_delay_seconds,
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

    #[error("Unknown theme.")]
    UnknownTheme,
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
