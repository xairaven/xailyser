use crate::context;
use crate::context::Context;
use crate::net::interface::InterfaceError;
use pnet::datalink::DataLinkReceiver;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub struct PacketSniffer {
    context: Arc<Mutex<Context>>,
    shutdown_flag: Arc<AtomicBool>,
}

// TODO
impl PacketSniffer {
    pub fn new(context: Arc<Mutex<Context>>, shutdown_flag: Arc<AtomicBool>) -> Self {
        Self {
            context,
            shutdown_flag,
        }
    }

    pub fn start(&self) -> Result<(), NetworkError> {
        let interface = context::lock(&self.context, |ctx| ctx.network_interface.clone())
            .ok_or(NetworkError::NoInterface)?;

        let datalink_rx = match interface::get_datalink_channel(&interface) {
            Ok(channel) => channel,
            Err(err) => {
                return Err(NetworkError::InterfaceError(err));
            },
        };

        self.listen(datalink_rx)?;

        Ok(())
    }

    pub fn listen(
        &self, mut channel_rx: Box<dyn DataLinkReceiver>,
    ) -> Result<(), NetworkError> {
        loop {
            if self.shutdown_flag.load(Ordering::Acquire) {
                log::info!("Shutting down net-capturing thread.");
                break;
            }

            match channel_rx.next() {
                Ok(_packet) => {
                    // TODO: To handle packet.
                },
                Err(err) => {
                    return Err(NetworkError::IoError(err));
                },
            }
        }

        Ok(())
    }
}

pub mod interface;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Interface is absent.")]
    NoInterface,

    #[error("Interface error.")]
    InterfaceError(#[from] InterfaceError),

    #[error("Input-output error.")]
    IoError(std::io::Error),
}
