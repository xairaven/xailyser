use crate::config::Config;
use app::App;

pub const MIN_WINDOW_WIDTH: f32 = 900.0;
pub const MIN_WINDOW_HEIGHT: f32 = 550.0;
const WINDOW_TITLE: &str = "Xailyser";

pub fn start(config: &Config) -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT])
            .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT])
            .with_icon(
                eframe::icon_data::from_png_bytes(
                    &include_bytes!("../assets/icon-64.png")[..],
                )
                .unwrap_or_else(|err| {
                    log::error!("{}", format!("Failed to load app icon. {err}"));
                    std::process::exit(1);
                }),
            ),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, config.theme)))),
    )
}

mod app;

pub mod auth;
pub mod components {
    pub mod settings;
    pub mod status;
}
pub mod menu;
pub mod root;
pub mod styles;
pub mod windows;
