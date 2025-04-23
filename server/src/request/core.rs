use crate::context::Context;
use crate::net::interface;
use crate::request::commands;
use common::messages::{Request, Response, ServerError, ServerSettingsDto};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub fn process(
    request: Request, context: &Arc<Mutex<Context>>, shutdown_flag: &Arc<AtomicBool>,
) -> Option<Response> {
    match request {
        Request::ChangePassword(password) => {
            let response = lock_with_response(context, |ctx| {
                ctx.change_password(password);
                Response::SuccessChangePassword
            });

            Some(response)
        },

        Request::Reboot => {
            shutdown_flag.store(true, Ordering::Release);
            commands::exit_reboot();

            None
        },

        Request::SaveConfig => {
            let response =
                lock_with_response(context, |ctx| match ctx.config.save_to_file() {
                    Ok(_) => Response::SuccessSaveConfig,
                    Err(_) => Response::Error(ServerError::FailedToSaveConfig),
                });
            Some(response)
        },

        Request::ServerSettings => {
            let interfaces_available = match commands::interfaces() {
                Ok(interfaces) => interfaces,
                Err(err) => return Some(Response::Error(err)),
            };

            let response = lock_with_response(context, |ctx| {
                let dto = ServerSettingsDto {
                    compression_active: ctx.compression,
                    compression_config: ctx.config.compression,
                    interface_active: ctx
                        .network_interface
                        .as_ref()
                        .map(interface::get_network_interface_name),
                    interface_config: ctx.config.interface.clone(),
                    interfaces_available,
                    send_unparsed_frames_active: ctx.send_unparsed_frames,
                    send_unparsed_frames_config: ctx.config.send_unparsed_frames,
                };

                Response::ServerSettings(dto)
            });
            Some(response)
        },

        Request::SetCompression(is_compression_enabled) => {
            let response = lock_with_response(context, |ctx| {
                ctx.config.compression = is_compression_enabled;
                Response::SuccessSetCompression(is_compression_enabled)
            });

            Some(response)
        },

        Request::SetInterface(interface_name) => {
            let network_interface =
                match interface::get_network_interface(&interface_name) {
                    Ok(interface) => interface,
                    Err(err) => {
                        log::error!("Request Processing: Network Interface error. {err}");
                        let response = Response::Error(ServerError::InvalidInterface);
                        return Some(response);
                    },
                };

            let result = interface::get_capture(network_interface.clone(), 100);
            if let Err(err) = result {
                log::error!("Request Processing: Network Interface error. {err}");
                let response = Response::Error(ServerError::InvalidInterface);
                return Some(response);
            }

            let response = lock_with_response(context, |ctx| {
                ctx.change_config_network_interface(network_interface);
                Response::SuccessSetInterface(interface_name)
            });
            Some(response)
        },

        Request::SetSendUnparsedFrames(is_sending_enabled) => {
            let response = lock_with_response(context, |ctx| {
                ctx.config.send_unparsed_frames = is_sending_enabled;
                Response::SuccessSetSendUnparsedFrames(is_sending_enabled)
            });

            Some(response)
        },
    }
}

fn lock_with_response(
    context: &Arc<Mutex<Context>>, f: impl FnOnce(&mut Context) -> Response,
) -> Response {
    match context.lock() {
        Ok(mut guard) => f(&mut guard),
        Err(err) => {
            log::error!("Request Processing: Error locking mutex on context. {err}");
            Response::Error(ServerError::MutexPoisoned)
        },
    }
}
