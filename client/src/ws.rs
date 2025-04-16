use crate::communication::request::UiClientRequest;
use common::auth::AUTH_HEADER;
use common::compression::decompress;
use common::cryptography::encrypt_password;
use common::messages::{CONNECTION_TIMEOUT, Request, Response};
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

pub struct WsHandler {
    pub compression: bool,
    pub shutdown_flag: Arc<AtomicBool>,

    pub stream: WsStream,
    pub server_response_tx: Sender<Response>,
    pub ui_client_requests_rx: Receiver<UiClientRequest>,
}

pub fn connect(
    address: SocketAddr, password: &str, compression: bool,
) -> Result<WsStream, WsError> {
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

    // Requesting server settings after connection
    let request = UiClientRequest::Request(Request::ServerSettings);
    match request.into_message(compression) {
        Ok(value) => {
            let result = stream.send(value);
            match result {
                Ok(_) => log::info!("WS-Stream: Sent connection server settings request"),
                Err(_) => {
                    log::error!("WS-Stream: Failed to send connection settings request.")
                },
            }
        },
        Err(_) => {
            log::error!(
                "WS-Stream: Serde. Can't serialize server settings request after connection!"
            );
        },
    };

    Ok(stream)
}

impl WsHandler {
    pub fn send_receive_messages(&mut self) {
        while !self.shutdown_flag.load(Ordering::Acquire) {
            if self.receive_messages().is_err() {
                return;
            }
            self.send_messages();
        }
    }

    fn receive_messages(&mut self) -> Result<(), tungstenite::Error> {
        let msg = match self.stream.read() {
            Ok(value) => value,
            Err(err) => {
                return match err {
                    tungstenite::Error::ConnectionClosed
                    | tungstenite::Error::AlreadyClosed => {
                        log::warn!(
                            "WS-Stream: Connection closed without alerting about it."
                        );
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
            if let Err(err) = self.server_response_tx.try_send(Response::SyncSuccessful) {
                log::error!("WS Channel: Can't send message. Error: {}", err);
            }
            return Ok(());
        }

        // Server don't send ping messages
        if msg.is_ping() {
            if let Err(err) = self.stream.send(Message::Pong(Bytes::new())) {
                log::error!("WS-Stream: Can't send message. Error: {}", err);
            }
        }

        match self.compression {
            true => self.handle_binary_compressed(msg),
            false => self.handle_text_uncompressed(msg),
        }

        Ok(())
    }

    fn handle_binary_compressed(&self, msg: Message) {
        if msg.is_empty() || msg.is_text() {
            log::warn!("WS-Stream: Received empty or non-compressed message.");
        }

        if msg.is_binary() {
            let decompressed = match decompress(&msg.into_data()) {
                Ok(value) => value,
                Err(_) => {
                    log::error!("WS-Stream: Failed to decompress message.");
                    return;
                },
            };
            self.pass_responses(&decompressed);
        }
    }

    fn handle_text_uncompressed(&self, msg: Message) {
        if msg.is_empty() || msg.is_binary() {
            log::warn!("WS-Stream: Received empty or binary message.");
        }

        if msg.is_text() {
            self.pass_responses(&msg.to_string());
        }
    }

    fn pass_responses(&self, text: &str) {
        let deserialized: Result<Response, serde_json::Error> =
            serde_json::from_str(text);
        if let Ok(message) = deserialized {
            if let Err(err) = self.server_response_tx.try_send(message) {
                log::error!("WS Channel: Can't send message. Error: {}", err);
            }
        } else {
            log::warn!("Serde: can't deserialize message! {:#?}", text);
        }
    }

    fn send_messages(&mut self) {
        if let Ok(command) = self.ui_client_requests_rx.try_recv() {
            let message = match command.into_message(self.compression) {
                Ok(value) => value,
                Err(_) => {
                    log::error!("Serde: Can't serialize message!");
                    return;
                },
            };

            if let Err(err) = self.stream.send(message) {
                log::error!("WS-Stream: Can't send message. Error: {}", err);
            } else {
                log::debug!("WS-Stream (Client -> Server): Sent command.");
            }
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
