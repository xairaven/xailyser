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
        Request::ChangePassword(password) => {
            context::lock(context, |ctx| {
                ctx.change_password(password);
            });

            Some(Response::ChangePasswordConfirmation)
        },
        Request::Reboot => {
            shutdown_flag.store(true, Ordering::Release);
            commands::exit_reboot();

            None
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
        Request::ServerSettings => {
            let interfaces_available = match commands::interfaces() {
                Ok(interfaces) => interfaces,
                Err(err) => return Some(Response::Error(err)),
            };
            let (compression, interface_active, interface_config) =
                context::lock(context, |ctx| {
                    let compression = ctx.config.compression;
                    let interface_active = ctx
                        .network_interface
                        .as_ref()
                        .map(interface::get_network_interface_name);
                    let interface_config = ctx.config.interface.clone();

                    (compression, interface_active, interface_config)
                });

            let settings = ServerSettings {
                compression,
                interface_active,
                interface_config,
                interfaces_available,
            };

            Some(Response::ServerSettings(settings))
        },
        Request::SetCompression(is_compression_enabled) => {
            context::lock(context, |ctx| {
                ctx.change_config_compression(is_compression_enabled);
            });

            let response = Response::SetCompressionResult(Ok(is_compression_enabled));
            Some(response)
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
    }
}
