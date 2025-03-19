use crate::config::Config;
use crate::net::interface;
use crate::net::interface::InterfaceError;
use crossbeam::channel::{Receiver, Sender, unbounded};
use pnet::datalink::NetworkInterface;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use thiserror::Error;
use xailyser_common::cryptography::encrypt_password;
use xailyser_common::messages::{Request, Response};

#[derive(Clone)]
pub struct Context {
    pub config: Config,
    pub encrypted_password: String,

    pub network_interface: Option<NetworkInterface>,

    pub shutdown_flag: Arc<AtomicBool>,

    pub server_response_tx: Sender<Response>,
    pub server_response_rx: Receiver<Response>,
    pub client_request_tx: Sender<Request>,
    pub client_request_rx: Receiver<Request>,
}

impl Context {
    pub fn new(config: Config) -> Result<Self, ContextError> {
        let (server_response_tx, server_response_rx) = unbounded::<Response>();
        let (client_request_tx, client_request_rx) = unbounded::<Request>();

        let encrypted_password = encrypt_password(&config.password);

        let interface: Option<NetworkInterface> = match &config.interface {
            None => None,
            Some(interface_name) => {
                let network_interface = interface::get_network_interface(interface_name);
                let network_interface = match network_interface {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(ContextError::InterfaceError(err));
                    },
                };
                Some(network_interface)
            },
        };

        Ok(Self {
            config,

            encrypted_password,

            network_interface: interface,
            shutdown_flag: Arc::new(AtomicBool::new(false)),

            server_response_tx,
            server_response_rx,
            client_request_tx,
            client_request_rx,
        })
    }
}

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Interface error.")]
    InterfaceError(InterfaceError),
}
