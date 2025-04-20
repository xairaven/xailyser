use common::logging::LogError;
use log::LevelFilter;

pub fn setup(log_level: &LevelFilter, format: String) -> Result<(), LogError> {
    if log_level.eq(&LevelFilter::Off) {
        return Ok(());
    }

    let file_name = common::logging::generate_file_name("XAILYSER");
    let file = fern::log_file(file_name).map_err(LogError::IOError)?;

    fern::Dispatch::new()
        .level(*log_level)
        .format(move |out, message, record| {
            let formatted =
                common::logging::parse_format(format.clone(), message, record);

            out.finish(format_args!("{}", formatted))
        })
        .chain(file)
        .apply()
        .map_err(LogError::SetLoggerError)
}

pub fn localize_log_level(log_level: &LevelFilter) -> String {
    match log_level {
        LevelFilter::Off => t!("Logging.Level.Off").to_string(),
        LevelFilter::Error => t!("Logging.Level.Error").to_string(),
        LevelFilter::Warn => t!("Logging.Level.Warn").to_string(),
        LevelFilter::Info => t!("Logging.Level.Info").to_string(),
        LevelFilter::Debug => t!("Logging.Level.Debug").to_string(),
        LevelFilter::Trace => t!("Logging.Level.Trace").to_string(),
    }
}
