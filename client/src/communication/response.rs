use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use chrono::Local;
use common::messages::Response;

pub fn process(ctx: &mut Context, response: Response) {
    match response {
        Response::Data(_data) => {
            todo!()
        },
        Response::ServerSettings(settings) => {
            ctx.settings_server.interfaces_available = settings.interfaces_available;
            ctx.settings_server.interface_active = settings.interface_active;
            ctx.settings_server.interface_config = settings.interface_config;

            ctx.settings_server.last_updated = Some(Local::now());
        },
        Response::SetInterfaceResult(result) => {
            let modal = match result {
                Ok(interface) => MessageModal::info(&format!(
                    "Interface set: {interface}! Don't forget to save the config, if needed."
                )),
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::ChangePasswordConfirmation => {
            let modal = MessageModal::info(
                "Successfully changed password! Don't forget to save the config, if needed.",
            );
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::SaveConfigResult(result) => {
            let modal = match result {
                Ok(_) => MessageModal::info("Successfully saved the config!"),
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::Error(err) => {
            let modal = MessageModal::error(&err.to_string());
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::SyncSuccessful => {
            ctx.heartbeat.update();
        },
    }
}
