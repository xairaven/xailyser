use chrono::{Datelike, Local, Timelike};
use log::Record;
use std::fmt::Arguments;
use thiserror::Error;

pub const DEFAULT_FORMAT: &str = "[$Y-$m-$D $H:$M $LEVEL] $MESSAGE";

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

    // Time
    let time = Local::now();
    log = log.replacen("$Y", &format!("{:0>2}", time.year()), 1);
    log = log.replacen("$m", &format!("{:0>2}", time.month()), 1);
    log = log.replacen("$D", &format!("{:0>2}", time.day()), 1);
    log = log.replacen("$H", &format!("{:0>2}", time.hour()), 1);
    log = log.replacen("$M", &format!("{:0>2}", time.minute()), 1);
    log = log.replacen("$S", &format!("{:0>2}", time.second()), 1);

    // Level
    log = log.replacen("$LEVEL", record.level().as_str(), 1);

    // Target
    log = log.replacen("$TARGET", record.target(), 1);

    // Message
    log = log.replacen("$MESSAGE", &message.to_string(), 1);

    log
}
