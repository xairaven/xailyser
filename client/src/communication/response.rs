use crate::context::{Context, ServerSettings};
use crate::ui::modals::message::MessageModal;
use chrono::Local;
use common::messages::Response;

pub fn process(ctx: &mut Context, response: Response) {
    match response {
        Response::ChangePasswordConfirmation => {
            let modal = MessageModal::info(
                "Successfully changed password! Don't forget to save the config, if needed.",
            );
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::Data(_data) => {
            todo!()
        },
        Response::Error(err) => {
            let modal = MessageModal::error(&err.to_string());
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::SaveConfigResult(result) => {
            let modal = match result {
                Ok(_) => MessageModal::info("Successfully saved the config!"),
                Err(err) => MessageModal::error(&err.to_string()),
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
                    let is_enabled = if compression { "enabled" } else { "disabled" };
                    MessageModal::info(&format!(
                        "Compression {is_enabled}! Changes will take effect after saving config and reboot."
                    ))
                },
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::SetInterfaceResult(result) => {
            let modal = match result {
                Ok(interface) => MessageModal::info(&format!(
                    "Interface set: {interface}! Changes will take effect after saving config and reboot."
                )),
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::SyncSuccessful => {
            ctx.heartbeat.update();
        },
    }
}
