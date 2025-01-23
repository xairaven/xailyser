use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("IO Error.")]
    IOError(#[from] std::io::Error),

    #[error("Logger initialization error.")]
    SetLoggerError(log::SetLoggerError),
}

impl LogError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            LogError::IOError(err) => Some(err.to_string()),
            LogError::SetLoggerError(err) => Some(err.to_string()),
        }
    }
}
