use crate::net;
use common::messages::Response;

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

pub fn exit_reboot() {
    const RESTART_CODE: i32 = 42;

    log::info!("Restarting server.");
    std::process::exit(RESTART_CODE);
}
