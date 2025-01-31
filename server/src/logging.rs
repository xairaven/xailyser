use log::LevelFilter;
use xailyser_common::logging::LogError;

pub fn setup(log_level: &LevelFilter, format: String) -> Result<(), LogError> {
    if log_level.eq(&LevelFilter::Off) {
        return Ok(());
    }

    fern::Dispatch::new()
        .level(*log_level)
        .format(move |out, message, record| {
            let formatted =
                xailyser_common::logging::parse_format(format.clone(), message, record);

            out.finish(format_args!("{}", formatted))
        })
        .chain(std::io::stdout())
        .apply()
        .map_err(LogError::SetLoggerError)
}
