use chrono::Local;
use log::LevelFilter;
use xailyser_common::logging::error::LogError;

pub fn setup(log_level: LevelFilter) -> Result<(), LogError> {
    if log_level == LevelFilter::Off {
        return Ok(());
    }

    fern::Dispatch::new()
        .level(log_level)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply()
        .map_err(LogError::SetLoggerError)
}
