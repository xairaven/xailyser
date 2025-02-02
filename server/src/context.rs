use crate::config::Config;
use crossbeam::channel::{unbounded, Receiver, Sender};
use xailyser_common::messages::{ClientRequest, ServerResponse};

#[derive(Clone)]
pub struct Context {
    pub password: String,
    pub port: u16,

    pub server_response_tx: Sender<ServerResponse>,
    pub server_response_rx: Receiver<ServerResponse>,
    pub client_request_tx: Sender<ClientRequest>,
    pub client_request_rx: Receiver<ClientRequest>,
}

impl Context {
    pub fn new(config: &Config) -> Self {
        let (server_response_tx, server_response_rx) = unbounded::<ServerResponse>();
        let (client_request_tx, client_request_rx) = unbounded::<ClientRequest>();

        Self {
            password: config.password.clone(),
            port: config.port,

            server_response_tx,
            server_response_rx,
            client_request_tx,
            client_request_rx,
        }
    }
}
