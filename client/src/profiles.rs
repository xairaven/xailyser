use crate::errors::FileError;
use common::io::FileKind;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;

const PROFILES_FILENAME: &str = "profiles.toml";
const PROFILES_FILETYPE: FileKind = FileKind::Data;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ProfilesStorage {
    pub profiles: Vec<Profile>,
}

impl ProfilesStorage {
    pub fn from_file() -> Result<Self, FileError> {
        match common::io::get_storage_file_path(PROFILES_FILENAME, PROFILES_FILETYPE) {
            Ok(path) => {
                let data = fs::read_to_string(path);
                if data.is_err() {
                    let storage = Self::default();
                    storage.save_to_file()?;
                    return Ok(storage);
                }

                let storage: Self = toml::from_str(&data.unwrap_or_default())
                    .map_err(FileError::TomlDeserialization)?;
                Ok(storage)
            },
            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save_to_file(&self) -> Result<(), FileError> {
        let data = toml::to_string(&self).map_err(FileError::TomlSerialization)?;

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
