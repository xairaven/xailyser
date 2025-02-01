use crate::config::Config;
use log::LevelFilter;
use xailyser_common::logging::LogError;

pub fn setup(config: &Config) -> Result<(), LogError> {
    if config.log_level.eq(&LevelFilter::Off) {
        return Ok(());
    }

    let log_format = config.log_format.clone();
    fern::Dispatch::new()
        .level(config.log_level)
        .format(move |out, message, record| {
            let formatted = xailyser_common::logging::parse_format(
                log_format.clone(),
                message,
                record,
            );

            out.finish(format_args!("{}", formatted))
        })
        .chain(std::io::stdout())
        .apply()
        .map_err(LogError::SetLoggerError)
}
