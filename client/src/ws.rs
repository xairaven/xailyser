use crossbeam::channel::{Receiver, Sender};
use http::Uri;
use std::net::{SocketAddr, TcpStream};
use thiserror::Error;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Bytes, ClientRequestBuilder, Message, WebSocket};
use xailyser_common::auth::AUTH_HEADER;
use xailyser_common::cryptography::encrypt_password;
use xailyser_common::messages::{ClientRequest, ServerResponse};

type WSStream = WebSocket<MaybeTlsStream<TcpStream>>;

pub fn connect(address: SocketAddr, password: &str) -> Result<WSStream, NetError> {
    let uri: Uri = format!("ws://{}:{}/socket", address.ip(), address.port())
        .parse()
        .map_err(|_| NetError::FailedParseUri)?;
    let hashed_password = encrypt_password(password);
    let request =
        ClientRequestBuilder::new(uri).with_header(AUTH_HEADER, hashed_password);

    let (stream, response) = match tungstenite::connect(request) {
        Ok((stream, response)) => (stream, response),
        Err(err) => return Err(NetError::ConnectionFailed(err)),
    };
    log::info!("Connected! Status: {}", response.status());

    Ok(stream)
}

pub fn send_receive_messages(
    mut stream: WSStream, ws_tx: Sender<ServerResponse>, ui_rx: Receiver<ClientRequest>,
) {
    loop {
        if receive_messages(&mut stream, &ws_tx).is_err() {
            return;
        }
        send_messages(&mut stream, &ui_rx);
    }
}

fn receive_messages(
    stream: &mut WSStream, ws_tx: &Sender<ServerResponse>,
) -> Result<(), tungstenite::Error> {
    let msg = match stream.read() {
        Ok(value) => value,
        Err(err) => {
            return match err {
                tungstenite::Error::ConnectionClosed
                | tungstenite::Error::AlreadyClosed => {
                    log::warn!("Connection closed without alerting about it.");
                    Err(err)
                },
                tungstenite::Error::Io(err) => {
                    log::warn!("{}", err);
                    Err(tungstenite::Error::Io(err))
                },
                _ => {
                    log::error!("{}", err);
                    Ok(())
                },
            }
        },
    };

    if msg.is_close() {
        log::info!("Server closed connection.");
        return Err(tungstenite::Error::ConnectionClosed);
    }

    if msg.is_ping() {
        let _ = stream.send(Message::Pong(Bytes::new()));
    }

    if msg.is_empty() || msg.is_binary() {
        log::warn!("Received empty or binary message. Please, check server stability");
    }

    if msg.is_text() {
        let deserialized: Result<ServerResponse, serde_json::Error> =
            serde_json::from_str(&msg.to_string());
        if let Ok(message) = deserialized {
            let _ = ws_tx.try_send(message);
        } else {
            log::warn!("Can't deserialize message. Please, check server stability");
        }
    }

    Ok(())
}

fn send_messages(stream: &mut WSStream, ui_rx: &Receiver<ClientRequest>) {
    if let Ok(message) = ui_rx.try_recv() {
        if let Ok(serialized) = serde_json::to_string(&message) {
            let _ = stream.send(Message::text(serialized));
        } else {
            log::error!("Can't serialize message! {:#?}", message);
        }
    }
}

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Failed to connect")]
    ConnectionFailed(tungstenite::Error),

    #[error("Failed to parse Uri. Verify IP address & port")]
    FailedParseUri,
}

impl NetError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            NetError::ConnectionFailed(err) => Some(err.to_string()),
            _ => None,
        }
    }
}
