use crossbeam::channel::{Receiver, Sender};
use http::Uri;
use std::net::{SocketAddr, TcpStream};
use std::thread;
use thiserror::Error;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Bytes, ClientRequestBuilder, Message, WebSocket};
use xailyser_common::auth::AUTH_HEADER;
use xailyser_common::cryptography::encrypt_password;
use xailyser_common::messages::{ClientRequest, ServerResponse, CONNECTION_TIMEOUT};

type WsStream = WebSocket<MaybeTlsStream<TcpStream>>;

pub fn connect(address: SocketAddr, password: &str) -> Result<WsStream, WsError> {
    let uri: Uri = format!("ws://{}:{}/socket", address.ip(), address.port())
        .parse()
        .map_err(|_| WsError::FailedParseUri)?;
    let hashed_password = encrypt_password(password);
    let request =
        ClientRequestBuilder::new(uri).with_header(AUTH_HEADER, hashed_password);

    let mut stream = match tungstenite::connect(request) {
        Ok((stream, _)) => stream,
        Err(err) => return Err(WsError::ConnectionFailed(err)),
    };
    match stream.get_mut() {
        MaybeTlsStream::Plain(stream) => stream
            .set_read_timeout(Some(CONNECTION_TIMEOUT))
            .map_err(WsError::BadReadTimeoutDuration),
        _ => return Err(WsError::UnknownStreamType),
    }?;
    log::info!("WS-Stream: Connected to {}.", address);

    Ok(stream)
}

pub fn send_receive_messages(
    mut stream: WsStream, ws_tx: Sender<ServerResponse>, ui_rx: Receiver<ClientRequest>,
) {
    loop {
        if receive_messages(&mut stream, &ws_tx).is_err() {
            return;
        }
        send_messages(&mut stream, &ui_rx);
    }
}

fn receive_messages(
    stream: &mut WsStream, ws_tx: &Sender<ServerResponse>,
) -> Result<(), tungstenite::Error> {
    let msg = match stream.read() {
        Ok(value) => value,
        Err(err) => {
            return match err {
                tungstenite::Error::ConnectionClosed
                | tungstenite::Error::AlreadyClosed => {
                    log::warn!("WS-Stream: Connection closed without alerting about it.");
                    Err(err)
                },
                tungstenite::Error::Io(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        || err.kind() == std::io::ErrorKind::TimedOut =>
                {
                    thread::sleep(CONNECTION_TIMEOUT);
                    Ok(())
                },
                tungstenite::Error::Io(err) => {
                    log::warn!("WS-Stream: {}", err);
                    Err(tungstenite::Error::Io(err))
                },
                _ => {
                    log::error!("WS-Stream: {}", err);
                    Ok(())
                },
            }
        },
    };

    if msg.is_close() {
        log::info!("WS-Stream: Server closed connection.");
        return Err(tungstenite::Error::ConnectionClosed);
    }

    if msg.is_ping() {
        if let Err(err) = stream.send(Message::Pong(Bytes::new())) {
            log::error!("WS-Stream: Can't send message. Error: {}", err);
        }
    }

    if msg.is_empty() || msg.is_binary() {
        log::warn!("WS-Stream: Received empty or binary message.");
    }

    if msg.is_text() {
        let deserialized: Result<ServerResponse, serde_json::Error> =
            serde_json::from_str(&msg.to_string());
        if let Ok(message) = deserialized {
            if let Err(err) = ws_tx.try_send(message) {
                log::error!("WS Channel: Can't send message. Error: {}", err);
            }
        } else {
            log::warn!("Serde: can't deserialize message! {:#?}", msg);
        }
    }

    Ok(())
}

fn send_messages(stream: &mut WsStream, ui_rx: &Receiver<ClientRequest>) {
    if let Ok(message) = ui_rx.try_recv() {
        if let Ok(serialized) = serde_json::to_string(&message) {
            if let Err(err) = stream.send(Message::text(serialized)) {
                log::error!("WS-Stream: Can't send message. Error: {}", err);
            } else {
                log::info!("WS-Stream (Client -> Server): Sent reboot command.");
            }
        } else {
            log::error!("Serde: Can't serialize message! {:#?}", message);
        }
    }
}

#[derive(Debug, Error)]
pub enum WsError {
    #[error("Failed to connect")]
    ConnectionFailed(tungstenite::Error),

    #[error("Failed to parse Uri. Verify IP address & port")]
    FailedParseUri,

    #[error("Bad read timeout duration")]
    BadReadTimeoutDuration(std::io::Error),

    #[error("Unknown TLS stream type")]
    UnknownStreamType,
}

impl WsError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            WsError::ConnectionFailed(err) => Some(err.to_string()),
            WsError::BadReadTimeoutDuration(err) => Some(err.to_string()),
            _ => None,
        }
    }
}
