use crate::config::Config;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use xailyser_common::messages::{Request, Response};

#[derive(Clone)]
pub struct Context {
    pub config: Config,

    pub shutdown_flag: Arc<AtomicBool>,

    pub server_response_tx: Sender<Response>,
    pub server_response_rx: Receiver<Response>,
    pub client_request_tx: Sender<Request>,
    pub client_request_rx: Receiver<Request>,
}

impl Context {
    pub fn new(config: Config) -> Self {
        let (server_response_tx, server_response_rx) = unbounded::<Response>();
        let (client_request_tx, client_request_rx) = unbounded::<Request>();

        Self {
            config,

            shutdown_flag: Arc::new(AtomicBool::new(false)),

            server_response_tx,
            server_response_rx,
            client_request_tx,
            client_request_rx,
        }
    }
}
