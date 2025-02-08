use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub const CONNECTION_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    RequestInterfaces,      // List of available ethernet interfaces
    RequestActiveInterface, // Active Interface
    SetInterface(String),   // Set an ethernet interface
    SaveConfig,             // Save the config
    ChangePassword(String), // Change a password to another (not encrypted)
    Reboot, // Reboot server (needed to apply changing password, for example)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    InterfacesList(Vec<String>), // Available ethernet interfaces
    InterfaceActive(Option<String>), // Active interface
    SetInterfaceResult(Result<String, ServerError>), // Is interface set by request?
    SaveConfigResult(Result<(), ServerError>), // Is config was saved by request?
    ChangePasswordConfirmation,  // Is password changed by request?

    Error(ServerError), // Generic Error.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerError {
    InvalidMessageFormat,

    InvalidInterface,
    FailedToChangePassword,
    FailedToSaveConfig,
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ServerError::InvalidMessageFormat => {
                "Invalid message format. Please, report this to developers.".to_string()
            },
            ServerError::InvalidInterface => "Invalid interface.".to_string(),
            ServerError::FailedToChangePassword => {
                "Failed to change password.".to_string()
            },
            ServerError::FailedToSaveConfig => "Failed to save config.".to_string(),
        };

        write!(f, "{}", msg)
    }
}
