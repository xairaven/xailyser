// Project lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

use crate::config::Config;

fn main() {
    let config = match Config::from_file() {
        Ok(value) => value,
        Err(err) => {
            let mut message = format!("Config initialization failed. Error: {err}.");
            if let Some(additional_info) = err.additional_info() {
                message.push_str(&format!(" Additional_info: {additional_info}"));
            }
            eprintln!("{}", message);
            std::process::exit(1);
        },
    };

    logging::setup(&config.log_level, config.log_format.clone()).unwrap_or_else(|err| {
        let mut message = format!("Logger initialization failed. Error: {err}.");
        if let Some(additional_info) = err.additional_info() {
            message.push_str(&format!(" Additional_info: {additional_info}"));
        }
        println!("{}", message);
        std::process::exit(1);
    });

    log::info!("Config loaded.");
    log::info!("Logger initialized.");

    ui::start(&config).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });
}

mod config;
mod context;
mod logging;
mod ui;
mod ws;
