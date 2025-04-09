use common::messages::{Request, Response};
use crossbeam::channel::{Receiver, Sender, unbounded};

#[derive(Clone)]
pub struct Channels {
    pub server_response_tx: Sender<Response>,
    pub server_response_rx: Receiver<Response>,
    pub client_request_tx: Sender<Request>,
    pub client_request_rx: Receiver<Request>,
}

impl Default for Channels {
    fn default() -> Self {
        let (server_response_tx, server_response_rx) = unbounded::<Response>();
        let (client_request_tx, client_request_rx) = unbounded::<Request>();

        Self {
            server_response_tx,
            server_response_rx,
            client_request_tx,
            client_request_rx,
        }
    }
}
