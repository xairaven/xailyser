use crate::config::Config;

fn main() {
    let config = match Config::from_file() {
        Ok(value) => value,
        Err(err) => {
            let mut message = format!("Config initialization failed. Error: {err}.");
            if let Some(additional_info) = err.additional_info() {
                message.push_str(&format!(" Additional_info: {additional_info}"));
            }
            eprintln!("{message}");
            std::process::exit(1);
        },
    };

    logging::setup(&config).unwrap_or_else(|err| {
        let mut message = format!("Logger initialization failed. Error: {err}.");
        if let Some(additional_info) = err.additional_info() {
            message.push_str(&format!(" Additional_info: {additional_info}"));
        }
        eprintln!("Error: {message}");
        std::process::exit(1);
    });

    log::info!("Starting...");
    log::info!("Config loaded: {config:#?}");
    log::info!("Logger initialized.");

    core::start(config);
}

mod config;
mod context;
mod core;
mod logging;
mod net;
mod request {
    pub mod commands;
    pub mod core;
}
mod tcp;
mod ws;
