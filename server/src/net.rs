use crate::context::Context;
use crate::net::interface::InterfaceError;
use pnet::datalink::DataLinkReceiver;
use std::sync::atomic::Ordering;
use thiserror::Error;

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

    pub fn start(&self) -> Result<(), NetworkError> {
        let interface = match self.runtime_ctx.network_interface.clone() {
            None => return Err(NetworkError::NoInterface),
            Some(value) => value,
        };

        let datalink_rx = interface::get_datalink_channel(&interface);
        let datalink_rx = match datalink_rx {
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
            if self.runtime_ctx.shutdown_flag.load(Ordering::Acquire) {
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
