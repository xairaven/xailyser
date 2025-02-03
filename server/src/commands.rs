use xailyser_common::messages::{ServerError, ServerResponse};

pub fn spawn_new_process() -> ServerResponse {
    let args: Vec<String> = std::env::args().collect();
    log::info!("Restarting server.");

    match std::process::Command::new(&args[0])
        .args(&args[1..])
        .spawn()
    {
        Ok(_) => ServerResponse::RebootResult(Ok(())),
        Err(err) => {
            ServerResponse::RebootResult(Err(ServerError::RebootFailure(err.to_string())))
        },
    }
}
