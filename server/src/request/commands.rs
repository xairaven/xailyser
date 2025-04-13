use crate::net;
use common::messages::{Response, ServerError};

pub fn interfaces() -> Response {
    let list = match net::interface::usable_sorted() {
        Ok(list) => list,
        Err(err) => {
            log::error!("Error listing interfaces: {}", err);
            return Response::Error(ServerError::FailedToGetInterfaces);
        },
    };

    let interfaces = list
        .into_iter()
        .map(|interface| net::interface::get_network_interface_name(&interface))
        .collect();
    Response::InterfacesList(interfaces)
}

pub fn exit_reboot() {
    const RESTART_CODE: i32 = 42;

    log::info!("Restarting server.");
    std::process::exit(RESTART_CODE);
}
