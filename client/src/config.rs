use directories::ProjectDirs;
use log::LevelFilter;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fs};
use thiserror::Error;

const CONFIG_FILENAME: &str = "config.toml";

pub struct Config {
    pub log_level: LevelFilter,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
        }
    }
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Config", 1)?;
        state.serialize_field("log_level", &self.log_level.to_string())?;
        state.end()
    }
}

impl Config {
    pub fn from_file() -> Result<Self, ConfigError> {
        let data = fs::read_to_string(CONFIG_FILENAME);
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
    log_level: String,
}

impl ConfigDto {
    pub fn to_config(&self) -> Result<Config, ConfigError> {
        let config = Config {
            log_level: LevelFilter::from_str(&self.log_level)
                .map_err(|_| ConfigError::BadLogLevel)?,
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
