use crate::communication::request::UiClientRequest;
use common::auth::AUTH_HEADER;
use common::cryptography::encrypt_password;
use common::messages::{CONNECTION_TIMEOUT, Response};
use crossbeam::channel::{Receiver, Sender};
use http::Uri;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use thiserror::Error;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Bytes, ClientRequestBuilder, Message, WebSocket};

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
    mut stream: WsStream, server_response_tx: Sender<Response>,
    ui_client_requests_rx: Receiver<UiClientRequest>, shutdown_flag: Arc<AtomicBool>,
) {
    while !shutdown_flag.load(Ordering::Acquire) {
        if receive_messages(&mut stream, &server_response_tx).is_err() {
            return;
        }
        send_messages(&mut stream, &ui_client_requests_rx);
    }
}

fn receive_messages(
    stream: &mut WsStream, server_response_tx: &Sender<Response>,
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
            };
        },
    };

    if msg.is_close() {
        log::info!("WS-Stream: Server closed connection.");
        return Err(tungstenite::Error::ConnectionClosed);
    }

    // Heartbeat
    if msg.is_pong() {
        log::debug!("WS-Stream: Received a Pong message.");
        if let Err(err) = server_response_tx.try_send(Response::SyncSuccessful) {
            log::error!("WS Channel: Can't send message. Error: {}", err);
        }
        return Ok(());
    }

    // Server don't send ping messages
    if msg.is_ping() {
        if let Err(err) = stream.send(Message::Pong(Bytes::new())) {
            log::error!("WS-Stream: Can't send message. Error: {}", err);
        }
    }

    if msg.is_empty() || msg.is_binary() {
        log::warn!("WS-Stream: Received empty or binary message.");
    }

    if msg.is_text() {
        let deserialized: Result<Response, serde_json::Error> =
            serde_json::from_str(&msg.to_string());
        if let Ok(message) = deserialized {
            if let Err(err) = server_response_tx.try_send(message) {
                log::error!("WS Channel: Can't send message. Error: {}", err);
            }
        } else {
            log::warn!("Serde: can't deserialize message! {:#?}", msg);
        }
    }

    Ok(())
}

fn send_messages(
    stream: &mut WsStream, ui_client_requests_rx: &Receiver<UiClientRequest>,
) {
    if let Ok(command) = ui_client_requests_rx.try_recv() {
        let message = match command.into_message() {
            Ok(value) => value,
            Err(_) => {
                log::error!("Serde: Can't serialize message!");
                return;
            },
        };

        if let Err(err) = stream.send(message) {
            log::error!("WS-Stream: Can't send message. Error: {}", err);
        } else {
            log::debug!("WS-Stream (Client -> Server): Sent command.");
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
            WsError::ConnectionFailed(err) => match err {
                tungstenite::Error::ConnectionClosed
                | tungstenite::Error::AlreadyClosed
                | tungstenite::Error::Io(_)
                | tungstenite::Error::Tls(_)
                | tungstenite::Error::Capacity(_)
                | tungstenite::Error::Protocol(_)
                | tungstenite::Error::WriteBufferFull(_)
                | tungstenite::Error::Utf8
                | tungstenite::Error::AttackAttempt
                | tungstenite::Error::HttpFormat(_) => Some(err.to_string()),
                tungstenite::Error::Url(_) => {
                    Some("Bad url (or server is not working)".to_string())
                },
                tungstenite::Error::Http(_) => {
                    Some("Unauthorized, or bad headers.".to_string())
                },
            },
            WsError::BadReadTimeoutDuration(err) => Some(err.to_string()),
            _ => None,
        }
    }
}
