use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Error, WebSocket};

pub fn start(mut websocket: WebSocket<MaybeTlsStream<TcpStream>>) {
    loop {
        let msg = match websocket.read() {
            Ok(value) => value,
            Err(err) => match err {
                Error::ConnectionClosed | Error::AlreadyClosed => {
                    log::warn!("Connection closed without alerting about it.");
                    return;
                },
                Error::Io(err) => {
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
            todo!()
        }
    }
}
