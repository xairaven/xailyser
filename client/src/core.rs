use crate::config::Config;
use crate::ui;

pub fn start(config: Config) {
    ui::start(&config).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });
    log::info!("UI Started.");
}
