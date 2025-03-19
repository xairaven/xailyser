use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use chrono::Local;
use xailyser_common::messages::Response;

pub fn process(ctx: &mut Context, response: Response) {
    match response {
        Response::InterfacesList(list) => {
            let modal = MessageModal::info("Successfully got interfaces list!");
            let _ = ctx.modals_tx.try_send(Box::new(modal));
            ctx.interfaces_available = list;
            ctx.interfaces_last_updated = Some(Local::now());
        },
        Response::InterfaceActive(name) => {
            ctx.interface_active = name;
            ctx.interfaces_last_updated = Some(Local::now());
        },
        Response::SetInterfaceResult(result) => {
            let modal = match result {
                Ok(interface) => MessageModal::info(&format!(
                    "Interface set: {interface}! Please save config & restart server for the changes to take effect."
                )),
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::ChangePasswordConfirmation => {
            let modal = MessageModal::info(
                "Successfully changed password! Please save config & restart server for the changes to take effect.",
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
    }
}
