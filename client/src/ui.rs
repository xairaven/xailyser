use crate::app::App;
use crate::config::Config;

const WINDOW_TITLE: &str = "Xailyser";
const WINDOW_WIDTH: f32 = 900.0;
const WINDOW_HEIGHT: f32 = 550.0;

pub fn start(config: &Config) -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_icon(
                eframe::icon_data::from_png_bytes(
                    &include_bytes!("../assets/icon-64.png")[..],
                )
                .unwrap_or_else(|err| {
                    log::error!("{}", format!("Failed to load app icon. {err}"));
                    std::process::exit(1);
                }),
            ),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, config.theme)))),
    )
}

#[derive(Debug, Default, Clone)]
pub struct Resolution {
    pub width: f32,
    pub height: f32,
}

impl PartialEq for Resolution {
    fn eq(&self, other: &Self) -> bool {
        self.width.eq(&other.width) && self.height.eq(&other.height)
    }
}

impl Resolution {
    pub fn from(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
