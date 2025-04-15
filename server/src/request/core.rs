use crate::context;
use crate::context::Context;
use crate::net::interface;
use crate::request::commands;
use common::messages::{Request, Response, ServerError, ServerSettings};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub fn process(
    request: Request, context: &Arc<Mutex<Context>>, shutdown_flag: &Arc<AtomicBool>,
) -> Option<Response> {
    match request {
        Request::ServerSettings => {
            let available_interfaces = match commands::interfaces() {
                Ok(interfaces) => interfaces,
                Err(err) => return Some(Response::Error(err)),
            };
            let active_interface = context::lock(context, |ctx| {
                ctx.network_interface
                    .as_ref()
                    .map(interface::get_network_interface_name)
            });
            let config_interface =
                context::lock(context, |ctx| ctx.config.interface.clone());

            let settings = ServerSettings {
                interface_active: active_interface,
                interface_config: config_interface,
                interfaces_available: available_interfaces,
            };

            Some(Response::ServerSettings(settings))
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

            let result = interface::get_capture(network_interface.clone(), 100);
            if let Err(err) = result {
                log::error!("Commands: Network Interface error. {err}");
                let response =
                    Response::SetInterfaceResult(Err(ServerError::InvalidInterface));
                return Some(response);
            }

            context::lock(context, |ctx| {
                ctx.change_config_network_interface(network_interface);
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
