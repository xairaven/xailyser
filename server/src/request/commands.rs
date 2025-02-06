use crate::net;
use xailyser_common::messages::{Response, ServerError};

pub fn interfaces() -> Response {
    let interfaces = net::interface::usable_sorted()
        .into_iter()
        .map(|interface| {
            #[cfg(target_os = "windows")]
            return interface.description;

            #[cfg(target_os = "linux")]
            return interface.name;
        })
        .collect();
    Response::InterfacesList(interfaces)
}

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
