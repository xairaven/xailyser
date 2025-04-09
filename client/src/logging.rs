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
