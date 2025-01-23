use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("Logger initialization error.")]
    SetLoggerError(log::SetLoggerError),
}

impl LogError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            LogError::SetLoggerError(err) => Some(err.to_string()),
        }
    }
}
