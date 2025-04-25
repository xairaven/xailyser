use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

pub const CONNECTION_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    ChangePassword(String), // Change a password to another (not encrypted)
    Reboot,         // Reboot server (needed to apply changing password, for example)
    SaveConfig,     // Save the config
    ServerSettings, // Interfaces, etc.
    SetCompression(bool), // Compression: On or Off
    SetInterface(String), // Set an ethernet interface
    SetSendUnparsedFrames(bool), // Set "Send unparsed frames" option
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    // Data itself
    Data(dpi::frame::FrameType),

    // Pong (Heartbeat)
    SuccessSync,

    // Settings: Interfaces, etc.
    ServerSettings(ServerSettingsDto), // Interfaces, etc.

    // Success
    SuccessChangePassword,
    SuccessSaveConfig,
    SuccessSetCompression(bool),
    SuccessSetInterface(String),
    SuccessSetSendUnparsedFrames(bool),

    // Error
    Error(ServerError),
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ServerError {
    #[error("Failed to change password.")]
    FailedToChangePassword,

    #[error("Failed to get server network interfaces list.")]
    FailedToGetInterfaces,

    #[error("Failed to save config file.")]
    FailedToSaveConfig,

    #[error("Invalid message format.")]
    InvalidMessageFormat,

    #[error("Invalid interface.")]
    InvalidInterface,

    #[error("Mutex poisoned.")]
    MutexPoisoned,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerSettingsDto {
    pub compression_active: bool,
    pub compression_config: bool,
    pub interface_active: Option<String>,
    pub interface_config: Option<String>,
    pub interfaces_available: Vec<String>,
    pub link_type: Option<i32>,
    pub send_unparsed_frames_active: bool,
    pub send_unparsed_frames_config: bool,
}
