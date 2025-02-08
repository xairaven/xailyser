use strum_macros::{Display, EnumIter};

#[derive(
    Default, Copy, Clone, Display, EnumIter, PartialEq, Eq, PartialOrd, Ord, Debug,
)]
pub enum Tab {
    #[default]
    #[strum(to_string = "🏠 Status")]
    Status,

    #[strum(to_string = "⚙ Client Settings")]
    ClientSettings,

    #[strum(to_string = "⚙ Server Settings")]
    ServerSettings,

    #[strum(to_string = "ℹ About")]
    About,

    #[strum(to_string = "🔓 Logout")]
    Logout,

    #[strum(to_string = "🗙 Exit")]
    Exit,
}

pub mod about;
pub mod settings_client;
pub mod settings_server;
pub mod status;
