use xailyser_common::messages::{Response, ServerError};

pub fn spawn_new_process() -> Response {
    let args: Vec<String> = std::env::args().collect();
    log::info!("Restarting server.");

    match std::process::Command::new(&args[0])
        .args(&args[1..])
        .spawn()
    {
        Ok(_) => Response::RebootResult(Ok(())),
        Err(err) => {
            Response::RebootResult(Err(ServerError::RebootFailure(err.to_string())))
        },
    }
}
