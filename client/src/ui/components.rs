use strum_macros::{Display, EnumIter};

pub mod settings;
pub mod status;

#[derive(Default, Display, EnumIter)]
pub enum Tab {
    #[default]
    #[strum(to_string = "Status")]
    Status,

    #[strum(to_string = "Settings")]
    Settings,

    #[strum(to_string = "Exit")]
    Exit,
}
