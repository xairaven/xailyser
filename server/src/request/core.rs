use crate::context::Context;
use crate::request::commands;
use std::sync::atomic::Ordering;
use xailyser_common::messages::Request;

pub fn process(ctx: &mut Context, request: Request) {
    match request {
        Request::RequestInterfaces => {
            log::info!("Commands: Interfaces requested.");
            let response = commands::interfaces();
            let _ = ctx.server_response_tx.try_send(response);
        },
        Request::SetInterface(_) => {
            todo!()
        },
        Request::ChangePassword(_) => {
            todo!()
        },
        Request::Reboot => {
            log::info!("Commands: Reboot requested.");
            let response = commands::spawn_new_process();
            let _ = ctx.server_response_tx.try_send(response);
            ctx.shutdown_flag.store(true, Ordering::Release);
        },
    }
}
