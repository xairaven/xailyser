use crate::channels::Channels;
use crate::context::Context;
use crate::net::interface;
use crate::request::commands;
use common::messages::{Request, Response, ServerError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub fn process(
    request: Request, context: &Arc<Mutex<Context>>, channels: &Channels,
    shutdown_flag: &Arc<AtomicBool>,
) {
    match request {
        Request::RequestInterfaces => {
            log::info!("Commands: Interfaces requested.");
            let response = commands::interfaces();
            let _ = channels.server_response_tx.try_send(response);
        },
        Request::RequestActiveInterface => {
            log::info!("Commands: Active interface requested.");
            let interface = match context.lock() {
                Ok(guard) => guard.network_interface.clone(),
                Err(err) => {
                    log::error!("{}", err);
                    std::process::exit(1);
                },
            };
            let name: Option<String> =
                interface.map(|iface| interface::get_network_interface_name(&iface));
            let response = Response::InterfaceActive(name);
            let _ = channels.server_response_tx.try_send(response);
        },
        Request::SetInterface(interface_name) => {
            let network_interface =
                match interface::get_network_interface(&interface_name) {
                    Ok(interface) => interface,
                    Err(err) => {
                        log::error!("Commands: Network Interface error. {err}");
                        let response = Response::SetInterfaceResult(Err(
                            ServerError::InvalidInterface,
                        ));
                        let _ = channels.server_response_tx.try_send(response);
                        return;
                    },
                };

            let mut context_guard = match context.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    log::error!("{}", err);
                    std::process::exit(1);
                },
            };
            context_guard.change_network_interface(network_interface);

            log::info!("Commands: Set new interface!");
            let response = Response::SetInterfaceResult(Ok(interface_name));
            let _ = channels.server_response_tx.try_send(response);
        },
        Request::ChangePassword(password) => {
            log::info!("Commands: Changing password requested.");
            let mut context_guard = match context.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    log::error!("{}", err);
                    std::process::exit(1);
                },
            };
            context_guard.change_password(password);

            let _ = channels
                .server_response_tx
                .try_send(Response::ChangePasswordConfirmation);
        },
        Request::SaveConfig => {
            log::info!("Commands: Saving config requested.");
            let context_guard = match context.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    log::error!("{}", err);
                    std::process::exit(1);
                },
            };

            let response = match context_guard.config.save_to_file() {
                Ok(_) => Response::SaveConfigResult(Ok(())),
                Err(_) => {
                    Response::SaveConfigResult(Err(ServerError::FailedToSaveConfig))
                },
            };
            let _ = channels.server_response_tx.try_send(response);
        },
        Request::Reboot => {
            log::info!("Commands: Reboot requested.");
            shutdown_flag.store(true, Ordering::Release);
            commands::exit_reboot();
        },
    }
}
