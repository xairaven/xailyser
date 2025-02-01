use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientToServerMessage {
    RequestInterfaces,      // Request a list of available ethernet interfaces
    SetInterface(String),   // Request to set an interface
    ChangePassword(String), // Request to change a password to another (not encrypted)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerToClientMessage {
    InterfacesList(Vec<String>), // Reply: Available ethernet interfaces
    SetInterfaceResult(Result<(), ServerError>), // Reply: Is interface set?
    ChangePasswordResult(Result<(), ServerError>), // Reply: Is password changed?

    Error(ServerError), // Generic Error.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerError {
    InvalidMessageFormat,

    InvalidInterface,
    FailedToChangePassword,
}
