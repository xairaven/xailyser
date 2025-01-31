use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::thread::JoinHandle;
use thiserror::Error;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

type WSStream = WebSocket<MaybeTlsStream<TcpStream>>;

pub fn connect(address: SocketAddr) -> Result<JoinHandle<()>, NetError> {
    let request = format!("ws://{}:{}/socket", address.ip(), address.port());
    let (stream, response) = match tungstenite::connect(request) {
        Ok((stream, response)) => (stream, response),
        Err(err) => return Err(NetError::ConnectionFailed(err)),
    };
    log::info!("Connected! Status: {}", response.status());

    let handle = thread::spawn(move || {
        handle_messages(stream);
    });

    Ok(handle)
}

// TODO
pub fn handle_messages(mut stream: WSStream) {
    loop {
        let msg = match stream.read() {
            Ok(value) => value,
            Err(err) => match err {
                tungstenite::Error::ConnectionClosed
                | tungstenite::Error::AlreadyClosed => {
                    log::warn!("Connection closed without alerting about it.");
                    return;
                },
                tungstenite::Error::Io(err) => {
                    log::warn!("{}", err);
                    return;
                },
                _ => {
                    log::error!("{}", err);
                    continue;
                },
            },
        };

        if msg.is_close() {
            return;
        }

        if msg.is_binary() || msg.is_text() {
            log::info!("Text received: {}", msg);
        }
    }
}

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Failed to connect")]
    ConnectionFailed(tungstenite::Error),
}

impl NetError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            NetError::ConnectionFailed(err) => Some(err.to_string()),
        }
    }
}
