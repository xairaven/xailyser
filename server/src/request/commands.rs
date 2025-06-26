use crate::net;
use common::messages::ServerError;

pub fn interfaces() -> Result<Vec<String>, ServerError> {
    let list = match net::interface::usable_sorted() {
        Ok(list) => list,
        Err(err) => {
            log::error!("Error listing interfaces: {err}");
            return Err(ServerError::FailedToGetInterfaces);
        },
    };

    let interfaces = list
        .into_iter()
        .map(|interface| net::interface::get_network_interface_name(&interface))
        .collect();
    Ok(interfaces)
}

pub fn exit_reboot() {
    const RESTART_CODE: i32 = 42;

    log::info!("Restarting server.");
    std::process::exit(RESTART_CODE);
}
