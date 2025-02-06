use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use xailyser_common::messages::Response;

pub fn process(ctx: &mut Context) {
    match ctx.server_response_rx.try_recv() {
        Ok(Response::InterfacesList(list)) => {
            let modal = MessageModal::info("Successfully got interfaces list!");
            let _ = ctx.modals_tx.try_send(Box::new(modal));
            ctx.interfaces_available = list;
        },
        Ok(Response::SetInterfaceResult(result)) => {
            let modal = match result {
                Ok(_) => MessageModal::info("Successfully set interface!"),
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Ok(Response::ChangePasswordResult(result)) => {
            let modal = match result {
                Ok(_) => MessageModal::info("Successfully changed password!"),
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Ok(Response::RebootResult(result)) => {
            let modal = match result {
                Ok(_) => MessageModal::info("Successfully rebooted server!"),
                Err(err) => MessageModal::error(&err.to_string()),
            };
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Ok(Response::Error(err)) => {
            let modal = MessageModal::error(&err.to_string());
            let _ = ctx.modals_tx.try_send(Box::new(modal));
        },
        Err(_) => {},
    }
}
