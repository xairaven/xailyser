use common::io::FileKind;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use thiserror::Error;

const PROFILES_FILENAME: &str = "profiles.toml";
const PROFILES_FILETYPE: FileKind = FileKind::Data;

#[derive(Default, Serialize, Deserialize)]
pub struct ProfilesStorage {
    pub profiles: Vec<Profile>,
}

impl ProfilesStorage {
    pub fn from_file() -> Result<Self, ProfileError> {
        match common::io::get_storage_file_path(PROFILES_FILENAME, PROFILES_FILETYPE) {
            Ok(path) => {
                let data = fs::read_to_string(path);
                if data.is_err() {
                    let storage = Self::default();
                    storage.save_to_file()?;
                    return Ok(storage);
                }

                let storage: Self = toml::from_str(&data.unwrap_or_default())
                    .map_err(ProfileError::TomlDeserialization)?;
                Ok(storage)
            },
            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save_to_file(&self) -> Result<(), ProfileError> {
        let data = toml::to_string(&self).map_err(ProfileError::TomlSerialization)?;

        let path =
            common::io::get_storage_file_path(PROFILES_FILENAME, PROFILES_FILETYPE)?;
        common::io::create_parent_directories(&path)?;
        fs::write(path, data)?;

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
    pub title: String,
    pub ip: IpAddr,
    pub port: u16,
    pub password: String,
}

#[derive(Error, Debug)]
pub enum ProfileError {
    #[error("IO Error.")]
    IO(#[from] std::io::Error),

    #[error("TOML Serialization Error.")]
    TomlSerialization(#[from] toml::ser::Error),

    #[error("TOML Deserialization Error.")]
    TomlDeserialization(#[from] toml::de::Error),
}

impl ProfileError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            ProfileError::IO(err) => Some(err.to_string()),
            ProfileError::TomlSerialization(err) => Some(err.to_string()),
            ProfileError::TomlDeserialization(err) => Some(err.to_string()),
        }
    }
}
