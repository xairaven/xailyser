use log::LevelFilter;
use xailyser_common::logging::LogError;

const FORMAT: &str = "[%Y-%m-%D %H-%M %LEVEL] %MESSAGE";

pub fn setup(log_level: &LevelFilter) -> Result<(), LogError> {
    if log_level.eq(&LevelFilter::Off) {
        return Ok(());
    }

    let file_name = xailyser_common::logging::generate_file_name("XAILYSER");
    let file = fern::log_file(file_name).map_err(LogError::IOError)?;

    fern::Dispatch::new()
        .level(*log_level)
        .format(move |out, message, record| {
            let formatted = xailyser_common::logging::parse_format(
                FORMAT.to_string(),
                message,
                record,
            );

            out.finish(format_args!("{}", formatted))
        })
        .chain(file)
        .apply()
        .map_err(LogError::SetLoggerError)
}
