use crate::net::heartbeat;
use crate::net::speed::SpeedUnitPerSecond;
use crate::ui::styles::themes;
use common::io::FileKind;
use common::logging;
use log::LevelFilter;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fs;
use std::str::FromStr;
use strum_macros::{Display, EnumIter, EnumString};
use thiserror::Error;

const CONFIG_FILENAME: &str = "config.toml";
const CONFIG_FILETYPE: FileKind = FileKind::Config;

#[derive(Debug, Clone)]
pub struct Config {
    pub compression: bool,
    pub language: Language,
    pub log_format: String,
    pub log_level: LevelFilter,
    pub parsed_frames_limit: Option<usize>,
    pub plot_display_window_seconds: u32,
    pub plot_speed_units: SpeedUnitPerSecond,
    pub sync_delay_seconds: i64,
    pub theme: themes::Preference,
    pub unparsed_frames_drop: bool,
    pub unparsed_frames_threshold: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compression: true,
            language: Language::English,
            log_format: logging::DEFAULT_FORMAT.to_string(),
            log_level: LevelFilter::Info,
            parsed_frames_limit: Some(100000),
            plot_display_window_seconds: 10,
            plot_speed_units: SpeedUnitPerSecond::Kilobytes,
            theme: themes::Preference::default(),
            sync_delay_seconds: heartbeat::DEFAULT_PING_DELAY_SECONDS,
            unparsed_frames_drop: true,
            unparsed_frames_threshold: Some(10000),
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
        state.serialize_field("language", &self.language.to_string())?;
        state.serialize_field("log_format", &self.log_format.to_string())?;
        state.serialize_field("log_level", &self.log_level.to_string())?;
        let limit = match &self.parsed_frames_limit {
            Some(value) => &value.to_string(),
            None => "none",
        };
        state.serialize_field("parsed_frames_limit", limit)?;
        state.serialize_field(
            "plot_display_window_seconds",
            &self.plot_display_window_seconds,
        )?;
        state.serialize_field("plot_speed_units", &self.plot_speed_units.to_string())?;
        state.serialize_field("sync_delay_seconds", &self.sync_delay_seconds)?;
        state.serialize_field("theme", &self.theme.to_string())?;

        state.serialize_field("unparsed_frames_drop", &self.unparsed_frames_drop)?;
        let threshold = match &self.unparsed_frames_threshold {
            Some(value) => &value.to_string(),
            None => "none",
        };
        state.serialize_field("unparsed_frames_threshold", threshold)?;

        state.end()
    }
}

impl Config {
    pub fn from_file() -> Result<Self, ConfigError> {
        match common::io::get_storage_file_path(CONFIG_FILENAME, CONFIG_FILETYPE) {
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
            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save_to_file(&self) -> Result<(), ConfigError> {
        let data = toml::to_string(&self).map_err(ConfigError::TomlSerializationError)?;

        let path = common::io::get_storage_file_path(CONFIG_FILENAME, CONFIG_FILETYPE)?;
        common::io::create_parent_directories(&path)?;
        fs::write(path, data)?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct ConfigDto {
    compression: bool,
    language: String,
    log_format: String,
    log_level: String,
    parsed_frames_limit: String,
    plot_display_window_seconds: u32,
    plot_speed_units: String,
    sync_delay_seconds: i64,
    theme: String,
    unparsed_frames_drop: bool,
    unparsed_frames_threshold: String,
}

impl ConfigDto {
    pub fn into_config(self) -> Result<Config, ConfigError> {
        let config = Config {
            compression: self.compression,
            language: Language::from_str(&self.language)
                .map_err(|_| ConfigError::UnknownLanguage)?,
            log_format: self.log_format.trim().to_string(),
            log_level: LevelFilter::from_str(self.log_level.to_ascii_lowercase().trim())
                .map_err(|_| ConfigError::UnknownLogLevel)?,
            parsed_frames_limit: usize::from_str(&self.parsed_frames_limit).ok(),
            plot_display_window_seconds: self.plot_display_window_seconds,
            plot_speed_units: SpeedUnitPerSecond::try_from(
                self.plot_speed_units.as_str(),
            )
            .map_err(|_| ConfigError::UnknownSpeedUnits)?,
            sync_delay_seconds: self.sync_delay_seconds,
            theme: themes::Preference::from_str(self.theme.to_ascii_lowercase().trim())
                .map_err(|_| ConfigError::UnknownTheme)?,
            unparsed_frames_drop: self.unparsed_frames_drop,
            unparsed_frames_threshold: usize::from_str(&self.unparsed_frames_threshold)
                .ok(),
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

    #[error("Unknown language.")]
    UnknownLanguage,

    #[error("Unknown log level.")]
    UnknownLogLevel,

    #[error("Unknown speed units.")]
    UnknownSpeedUnits,

    #[error("Unknown theme.")]
    UnknownTheme,
}

impl ConfigError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            ConfigError::TomlSerializationError(err) => Some(err.to_string()),
            ConfigError::TomlDeserializationError(err) => Some(err.to_string()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Display, EnumIter, EnumString, PartialEq)]
pub enum Language {
    #[strum(serialize = "English")]
    English,

    #[strum(serialize = "Ukrainian")]
    Ukrainian,
}

impl Language {
    pub fn localize(&self) -> String {
        match self {
            Language::English => t!("Language.English").to_string(),
            Language::Ukrainian => t!("Language.Ukrainian").to_string(),
        }
    }
}
