use crate::context::{Context, ServerSettings};
use crate::ui::modals::message::MessageModal;
use chrono::Local;
use common::messages::{Response, ServerError};

pub fn process(ctx: &mut Context, response: Response) {
    match response {
        Response::ChangePasswordConfirmation => {
            let modal = MessageModal::info(&t!("Response.PasswordChange.Success"));
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::Data(_data) => {
            todo!()
        },
        Response::Error(err) => {
            let modal = MessageModal::error(&localize_server_errors(&err));
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::SaveConfigResult(result) => {
            let modal = match result {
                Ok(_) => MessageModal::info(&t!("Response.SaveConfig.Success")),
                Err(err) => MessageModal::error(&localize_server_errors(&err)),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::ServerSettings(dto) => {
            ctx.settings_server = ServerSettings {
                compression_active: dto.compression_active,
                compression_config: dto.compression_config,

                interfaces_available: dto.interfaces_available,
                interface_active: dto.interface_active,
                interface_config: dto.interface_config,

                last_updated: Some(Local::now()),
            };
        },
        Response::SetCompressionResult(result) => {
            let modal = match result {
                Ok(compression) => {
                    let text: String = if compression {
                        t!("Response.SetCompression.Success.On").to_string()
                    } else {
                        t!("Response.SetCompression.Success.Off").to_string()
                    };
                    MessageModal::info(&text)
                },
                Err(err) => MessageModal::error(&localize_server_errors(&err)),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::SetInterfaceResult(result) => {
            let modal = match result {
                Ok(interface) => MessageModal::info(&t!(
                    "Response.SetInterface.Success",
                    "interface" = interface
                )),
                Err(err) => MessageModal::error(&localize_server_errors(&err)),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::SyncSuccessful => {
            ctx.heartbeat.update();
        },
    }
}

fn localize_server_errors(err: &ServerError) -> String {
    match err {
        ServerError::FailedToChangePassword => {
            t!("Response.Error.PasswordChange").to_string()
        },
        ServerError::FailedToGetInterfaces => {
            t!("Response.Error.InterfacesGet").to_string()
        },
        ServerError::FailedToSaveConfig => t!("Response.Error.ConfigSave").to_string(),
        ServerError::InvalidMessageFormat => {
            t!("Response.Error.InvalidMessageFormat").to_string()
        },
        ServerError::InvalidInterface => {
            t!("Response.Error.InvalidInterface").to_string()
        },
    }
}
