use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("IO Error.")]
    IO(#[from] std::io::Error),

    #[error("TOML Serialization Error.")]
    TomlSerialization(#[from] toml::ser::Error),

    #[error("TOML Deserialization Error.")]
    TomlDeserialization(#[from] toml::de::Error),
}

impl FileError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            Self::IO(err) => Some(err.to_string()),
            Self::TomlSerialization(err) => Some(err.to_string()),
            Self::TomlDeserialization(err) => Some(err.to_string()),
        }
    }
}
