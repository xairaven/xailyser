use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientRequest {
    RequestInterfaces,      // List of available ethernet interfaces
    SetInterface(String),   // Set an ethernet interface
    ChangePassword(String), // Change a password to another (not encrypted)
    Reboot, // Reboot server (needed to apply changing password, for example)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerResponse {
    InterfacesList(Vec<String>), // Available ethernet interfaces
    SetInterfaceResult(Result<(), ServerError>), // Is interface set by request?
    ChangePasswordResult(Result<(), ServerError>), // Is password changed by request?
    RebootResult(Result<(), ServerError>), // Can reboot at the moment?

    Error(ServerError), // Generic Error.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerError {
    InvalidMessageFormat,

    InvalidInterface,
    FailedToChangePassword,
    FailedToReboot,
}
