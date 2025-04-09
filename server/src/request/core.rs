use crate::channels::Channels;
use crate::context;
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
            send_response(channels, response);
        },
        Request::RequestActiveInterface => {
            log::info!("Commands: Active interface requested.");
            let name = context::lock(context, |ctx| {
                ctx.network_interface
                    .as_ref()
                    .map(interface::get_network_interface_name)
            });

            let response = Response::InterfaceActive(name);
            send_response(channels, response);
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

            context::lock(context, |ctx| {
                ctx.change_network_interface(network_interface);
            });

            log::info!("Commands: Set new interface!");
            let response = Response::SetInterfaceResult(Ok(interface_name));
            send_response(channels, response);
        },
        Request::ChangePassword(password) => {
            log::info!("Commands: Changing password requested.");
            context::lock(context, |ctx| {
                ctx.change_password(password);
            });

            send_response(channels, Response::ChangePasswordConfirmation);
        },
        Request::SaveConfig => {
            log::info!("Commands: Saving config requested.");
            context::lock(context, |ctx| {
                let response = match ctx.config.save_to_file() {
                    Ok(_) => Response::SaveConfigResult(Ok(())),
                    Err(_) => {
                        Response::SaveConfigResult(Err(ServerError::FailedToSaveConfig))
                    },
                };
                send_response(channels, response);
            });
        },
        Request::Reboot => {
            log::info!("Commands: Reboot requested.");
            shutdown_flag.store(true, Ordering::Release);
            commands::exit_reboot();
        },
    }
}

fn send_response(channels: &Channels, response: Response) {
    if channels.server_response_tx.try_send(response).is_err() {
        log::warn!("Failed to send response.");
    }
}
