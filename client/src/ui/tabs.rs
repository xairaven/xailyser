use rust_i18n::t;
use strum_macros::EnumIter;

#[derive(Default, Copy, Clone, EnumIter, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Tab {
    #[default]
    Status,
    Inspector,
    Stats,
    ClientSettings,
    ServerSettings,
    About,
    Logout,
    Exit,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Status => format!("🏠 {}", t!("Tabs.Status")),
            Self::Inspector => format!("🔍 {}", t!("Tabs.Inspector")),
            Self::Stats => format!("🗠 {}", t!("Tabs.Stats")),
            Self::ClientSettings => format!("⚙ {}", t!("Tabs.ClientSettings")),
            Self::ServerSettings => format!("⚙ {}", t!("Tabs.ServerSettings")),
            Self::About => format!("ℹ {}", t!("Tabs.About")),
            Self::Logout => format!("🔓 {}", t!("Tabs.Logout")),
            Self::Exit => format!("🗙 {}", t!("Tabs.Exit")),
        };

        write!(f, "{text}")
    }
}

pub mod about;
pub mod inspector;
pub mod settings_client;
pub mod settings_server;
pub mod stats;
pub mod status;
