use strum_macros::{Display, EnumIter};

#[derive(
    Default, Copy, Clone, Display, EnumIter, PartialEq, Eq, PartialOrd, Ord, Debug,
)]
pub enum Tab {
    #[default]
    #[strum(to_string = "🏠 Status")]
    Status,

    #[strum(to_string = "⚙ Settings")]
    Settings,

    #[strum(to_string = "ℹ About")]
    About,

    #[strum(to_string = "🗙 Exit")]
    Exit,
}

pub mod settings;
pub mod status;
