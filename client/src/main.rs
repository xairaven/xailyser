// Project lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

fn main() {
    let level_filter = log::LevelFilter::Info;

    logging::setup(&level_filter).unwrap_or_else(|err| {
        let mut message = format!("Logger initialization failed. Error: {err}.");
        if let Some(additional_info) = err.additional_info() {
            message.push_str(&format!(" Additional_info: {additional_info}"));
        }
        println!("{}", message);
        std::process::exit(1);
    });
}

mod logging;
