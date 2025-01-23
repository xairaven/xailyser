use chrono::{Datelike, Local, Timelike};
use log::Record;
use std::fmt::Arguments;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("IO Error.")]
    IOError(#[from] std::io::Error),

    #[error("Logger initialization error.")]
    SetLoggerError(log::SetLoggerError),
}

impl LogError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            LogError::IOError(err) => Some(err.to_string()),
            LogError::SetLoggerError(err) => Some(err.to_string()),
        }
    }
}

pub fn generate_file_name(title: &str) -> String {
    let now = Local::now();
    let date = format!(
        "{year:04}-{month:02}-{day:02}",
        year = now.year(),
        month = now.month(),
        day = now.day(),
    );

    let title_formatted = title.trim().replace(" ", "-");
    format!("{title_formatted}_{date}.log")
}

pub fn parse_format(format: String, message: &Arguments, record: &Record) -> String {
    let mut log = format.trim().to_string();

    // Message
    log = log.replace("%MESSAGE", &message.to_string());

    // Level
    log = log.replace("%LEVEL", record.level().as_str());

    // Target
    log = log.replace("%TARGET", record.target());

    // Time
    let time = Local::now();
    log = log.replace("%Y", &format!("{:0>2}", time.year()));
    log = log.replace("%m", &format!("{:0>2}", time.month()));
    log = log.replace("%D", &format!("{:0>2}", time.day()));
    log = log.replace("%H", &format!("{:0>2}", time.hour()));
    log = log.replace("%M", &format!("{:0>2}", time.minute()));
    log = log.replace("%S", &format!("{:0>2}", time.second()));

    log
}
