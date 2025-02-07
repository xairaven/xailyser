use crate::context::Context;
use crate::net::interface;
use crate::request::commands;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use xailyser_common::messages::{Request, Response, ServerError};

pub fn process(ctx: &mut Context, request: Request) {
    match request {
        Request::RequestInterfaces => {
            log::info!("Commands: Interfaces requested.");
            let response = commands::interfaces();
            let _ = ctx.server_response_tx.try_send(response);
        },
        Request::RequestActiveInterface => {
            log::info!("Commands: Active interface requested.");
            let interface = ctx.network_interface.clone();
            let name: Option<String> =
                interface.map(|iface| interface::get_network_interface_name(&iface));
            let response = Response::InterfaceActive(name);
            let _ = ctx.server_response_tx.try_send(response);
        },
        Request::SetInterface(interface_name) => {
            let network_interface = interface::get_network_interface(&interface_name);
            let network_interface = match network_interface {
                Ok(value) => {
                    log::info!("Commands: Set new interface!");

                    #[cfg(target_os = "windows")]
                    let name = value.description.clone();

                    #[cfg(target_os = "linux")]
                    let name = value.name.clone();

                    ctx.config.interface = Some(name.clone());

                    let response = Response::SetInterfaceResult(Ok(name));
                    let _ = ctx.server_response_tx.try_send(response);

                    value
                },
                Err(err) => {
                    log::error!("Commands: Network Interface error. {err}");
                    let response =
                        Response::SetInterfaceResult(Err(ServerError::InvalidInterface));
                    let _ = ctx.server_response_tx.try_send(response);
                    return;
                },
            };

            ctx.network_interface = Some(network_interface);
        },
        Request::ChangePassword(_) => {
            todo!()
        },
        Request::SaveConfig => {
            log::info!("Commands: Saving config requested.");
            let result = ctx.config.save_to_file();
            let response = match result {
                Ok(_) => Response::SaveConfigResult(Ok(())),
                Err(_) => {
                    Response::SaveConfigResult(Err(ServerError::FailedToSaveConfig))
                },
            };
            let _ = ctx.server_response_tx.try_send(response);
        },
        Request::Reboot => {
            log::info!("Commands: Reboot requested.");
            let response = Response::RebootResult(Ok(()));
            let _ = ctx.server_response_tx.try_send(response);
            ctx.shutdown_flag.store(true, Ordering::Release);

            // Waiting for sending `Reboot` response
            thread::sleep(Duration::from_millis(100));

            commands::exit_reboot();
        },
    }
}
