use crate::context;
use crate::context::Context;
use crate::net::interface;
use crate::request::commands;
use common::messages::{Request, Response, ServerError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub fn process(
    request: Request, context: &Arc<Mutex<Context>>, shutdown_flag: &Arc<AtomicBool>,
) -> Option<Response> {
    match request {
        Request::RequestInterfaces => {
            let response = commands::interfaces();
            Some(response)
        },
        Request::RequestActiveInterface => {
            let name = context::lock(context, |ctx| {
                ctx.network_interface
                    .as_ref()
                    .map(interface::get_network_interface_name)
            });

            Some(Response::InterfaceActive(name))
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
                        return Some(response);
                    },
                };

            context::lock(context, |ctx| {
                ctx.change_network_interface(network_interface);
            });

            let response = Response::SetInterfaceResult(Ok(interface_name));
            Some(response)
        },
        Request::ChangePassword(password) => {
            context::lock(context, |ctx| {
                ctx.change_password(password);
            });

            Some(Response::ChangePasswordConfirmation)
        },
        Request::SaveConfig => {
            let response = context::lock(context, |ctx| -> Response {
                match ctx.config.save_to_file() {
                    Ok(_) => Response::SaveConfigResult(Ok(())),
                    Err(_) => {
                        Response::SaveConfigResult(Err(ServerError::FailedToSaveConfig))
                    },
                }
            });
            Some(response)
        },
        Request::Reboot => {
            shutdown_flag.store(true, Ordering::Release);
            commands::exit_reboot();

            None
        },
    }
}
