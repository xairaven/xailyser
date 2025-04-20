use rust_i18n::t;
use strum_macros::EnumIter;

#[derive(Default, Copy, Clone, EnumIter, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Tab {
    #[default]
    Status,
    ClientSettings,
    ServerSettings,
    About,
    Logout,
    Exit,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Tab::Status => format!("🏠 {}", t!("Tabs.Status")),
            Tab::ClientSettings => format!("⚙ {}", t!("Tabs.ClientSettings")),
            Tab::ServerSettings => format!("⚙ {}", t!("Tabs.ServerSettings")),
            Tab::About => format!("ℹ {}", t!("Tabs.About")),
            Tab::Logout => format!("🔓 {}", t!("Tabs.Logout")),
            Tab::Exit => format!("🗙 {}", t!("Tabs.Exit")),
        };

        write!(f, "{}", text)
    }
}

pub mod about;
pub mod settings_client;
pub mod settings_server;
pub mod status;
