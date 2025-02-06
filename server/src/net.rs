use crate::context::Context;
use std::sync::atomic::Ordering;

pub struct PacketSniffer {
    runtime_ctx: Context,
}

// TODO
impl PacketSniffer {
    pub fn new(context: Context) -> Self {
        Self {
            runtime_ctx: context,
        }
    }

    pub fn start(&self) {
        loop {
            if self.runtime_ctx.shutdown_flag.load(Ordering::Acquire) {
                log::info!("Shutting down net-capturing thread.");
                break;
            }
        }
    }
}

pub mod interface;
