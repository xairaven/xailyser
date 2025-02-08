use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use xailyser_common::messages::Response;

pub fn process(ctx: &mut Context, response: Response) {
    match response {
        Response::InterfacesList(list) => {
            let modal = MessageModal::info("Successfully got interfaces list!");
            let _ = ctx.modals_tx.try_send(Box::new(modal));
            ctx.interfaces_available = list;
        },
        Response::InterfaceActive(name) => {
            ctx.interface_active = name;
        },
        Response::SetInterfaceResult(result) => {
            let modal = match result {
                Ok(interface) => {
                    MessageModal::info(&format!("Interface set: {interface}! Please restart the server & save config for the changes to take effect."))
                },
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Response::ChangePasswordResult(result) => {
            let modal = match result {
                Ok(_) => MessageModal::info("Successfully changed password! Please restart the server for the changes to take effect."),
                Err(err) => MessageModal::error(&err.to_string()),
            };
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
