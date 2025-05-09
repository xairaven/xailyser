// Project lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

#[macro_use]
extern crate rust_i18n;

// Defining folder with locales. Path: crate-root/locales
rust_i18n::i18n!("locales", fallback = "English");

use crate::config::Config;

fn main() {
    // Reading config
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

    // Setting language
    rust_i18n::set_locale(&config.language.to_string());

    // Logging setup
    logging::setup(&config.log_level, config.log_format.clone()).unwrap_or_else(|err| {
        let mut message = format!("Logger initialization failed. Error: {err}.");
        if let Some(additional_info) = err.additional_info() {
            message.push_str(&format!(" Additional_info: {additional_info}"));
        }
        println!("{}", message);
        std::process::exit(1);
    });

    log::info!("Starting...");
    log::info!("Config loaded: {:#?}", config);
    log::info!("Logger initialized.");

    ui::start(config).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });
}

mod config;
mod context;
mod logging;
mod net;
mod profiles;
mod ui;
mod ws;
