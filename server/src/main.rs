use crate::config::Config;

fn main() {
    let config = match Config::from_file() {
        Ok(value) => value,
        Err(err) => {
            let mut message = format!("Config initialization failed. Error: {err}.");
            if let Some(additional_info) = err.additional_info() {
                message.push_str(&format!(" Additional_info: {additional_info}"));
            }
            println!("{}", message);
            std::process::exit(1);
        },
    };

    logging::setup(&config.log_level().unwrap_or_else(|err| {
        println!("{}", err);
        std::process::exit(1);
    }))
    .unwrap_or_else(|err| {
        let mut message = format!("Logger initialization failed. Error: {err}.");
        if let Some(additional_info) = err.additional_info() {
            message.push_str(&format!(" Additional_info: {additional_info}"));
        }
        println!("{}", message);
        std::process::exit(1);
    });
}

mod config;
mod logging;
