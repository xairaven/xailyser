use crate::config::Config;
use crate::net::interface;
use crate::net::interface::InterfaceError;
use common::cryptography::encrypt_password;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub struct Context {
    pub compression: bool,
    pub config: Config,
    pub encrypted_password: String,
    pub link_type: Option<pcap::Linktype>,
    pub network_interface: Option<pcap::Device>,
    pub send_unparsed_frames: bool,
}

impl Context {
    pub fn new(config: Config) -> Result<Self, ContextError> {
        let encrypted_password = encrypt_password(&config.password);

        let interface: Option<pcap::Device> = match &config.interface {
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
            compression: config.compression,
            encrypted_password,
            link_type: None,
            network_interface: interface,
            send_unparsed_frames: config.send_unparsed_frames,

            config,
        })
    }

    pub fn change_config_network_interface(&mut self, interface: pcap::Device) {
        let name = interface::get_network_interface_name(&interface);
        self.config.interface = Some(name);
    }

    pub fn change_password(&mut self, new_password: String) {
        self.encrypted_password = encrypt_password(&new_password);
        self.config.password = new_password;
    }
}

pub fn lock<T>(context: &Arc<Mutex<Context>>, f: impl FnOnce(&mut Context) -> T) -> T {
    match context.lock() {
        Ok(mut guard) => f(&mut guard),
        Err(err) => {
            log::error!("Context lock failed: {}", err);
            std::process::exit(1);
        },
    }
}

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Interface error.")]
    InterfaceError(InterfaceError),
}
