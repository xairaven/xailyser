use std::net::TcpStream;
use tungstenite::WebSocket;

type WSStream = WebSocket<TcpStream>;

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
            todo!()
        }
    }
}
