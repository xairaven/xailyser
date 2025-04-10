use crate::channels::Channels;
use crate::context;
use crate::context::Context;
use bytes::Bytes;
use common::auth;
use common::messages::{CONNECTION_TIMEOUT, Request, Response, ServerError};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use thiserror::Error;
use tungstenite::handshake::server;
use tungstenite::http::{HeaderValue, StatusCode};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::{Message, Utf8Bytes, WebSocket};

pub struct WsHandler {
    id: u16,
    channels: Channels,
    context: Arc<Mutex<Context>>,
    shutdown_flag: Arc<AtomicBool>,
}

type WSStream = WebSocket<TcpStream>;

impl WsHandler {
    pub fn new(
        id: u16, channels: Channels, context: Arc<Mutex<Context>>,
        shutdown_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id,
            channels,
            context,
            shutdown_flag,
        }
    }

    pub fn start(&self, tcp_stream: TcpStream) -> Result<(), WsError> {
        let ws_stream = match self.connect(tcp_stream) {
            Ok(value) => {
                log::info!("WS-{}. Websocket connection established.", self.id);
                value
            },
            Err(err) => return Err(err),
        };

        self.send_receive_messages(ws_stream);
        Ok(())
    }

    fn connect(&self, tcp_stream: TcpStream) -> Result<WSStream, WsError> {
        if let Ok(peer_addr) = &tcp_stream.peer_addr() {
            log::info!(
                "WS-{}. Received a new handshake from {}:{}",
                self.id,
                peer_addr.ip(),
                peer_addr.port()
            );
        } else {
            log::info!("WS-{}. Received a new handshake!", self.id);
        }

        let server_password_header = context::lock(&self.context, |ctx| {
            HeaderValue::from_str(&ctx.encrypted_password)
                .map_err(|_| WsError::InvalidPasswordHeader)
        })?;

        let check_authentication =
            |req: &server::Request, response: server::Response| match req
                .headers()
                .get(auth::AUTH_HEADER)
            {
                Some(given_password) if given_password == server_password_header => {
                    Ok(response)
                },
                Some(_) => Err(server::Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Some(auth::errors::WRONG_PASSWORD_ERROR.to_string()))
                    .unwrap_or_default()),
                None => Err(server::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Some(auth::errors::HEADER_NOT_FOUND_ERROR.to_string()))
                    .unwrap_or_default()),
            };
        tungstenite::accept_hdr(tcp_stream, check_authentication)
            .map_err(|_| WsError::AuthFailed)
    }

    fn send_receive_messages(&self, mut stream: WSStream) {
        while !self.shutdown_flag.load(Ordering::Acquire) {
            if self.receive_messages(&mut stream).is_err() {
                return;
            }
            self.send_messages(&mut stream);
        }

        if let Ok(address) = stream.get_ref().peer_addr() {
            log::info!(
                "WS-{}. Closing connection ({}:{}), server is in shutdown process...",
                self.id,
                address.ip(),
                address.port()
            );
        } else {
            log::info!("WS-{}. Closing connection. IP & Port undefined.", self.id);
        }

        let _ = stream.close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: Default::default(),
        }));
    }

    fn receive_messages(&self, stream: &mut WSStream) -> Result<(), tungstenite::Error> {
        let msg = match stream.read() {
            Ok(value) => value,
            Err(err) => {
                return match err {
                    tungstenite::Error::ConnectionClosed
                    | tungstenite::Error::AlreadyClosed => {
                        log::warn!(
                            "WS-{}. Connection closed without alerting about it.",
                            self.id
                        );
                        Err(err)
                    },
                    tungstenite::Error::Io(err)
                        if err.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        thread::sleep(CONNECTION_TIMEOUT);
                        Ok(())
                    },
                    tungstenite::Error::Io(err) => {
                        log::warn!("WS-{}. {}", self.id, err);
                        Err(tungstenite::Error::Io(err))
                    },
                    _ => {
                        log::error!("WS-{}. {}", self.id, err);
                        Ok(())
                    },
                };
            },
        };

        if msg.is_close() {
            log::info!("WS-{}. Client closed connection.", self.id);
            return Err(tungstenite::Error::ConnectionClosed);
        }

        // Heartbeat system
        if msg.is_ping() {
            let _ = stream.send(Message::Pong(Bytes::new()));
            log::debug!("WS-{}. Got ping!", self.id);
            return Ok(());
        }

        if msg.is_empty() || msg.is_binary() {
            log::warn!("WS-{}. Received empty or binary message.", self.id);
            let message = Response::Error(ServerError::InvalidMessageFormat);
            if let Ok(text) = serde_json::to_string(&message) {
                if let Err(err) = stream.send(Message::Text(Utf8Bytes::from(text))) {
                    log::error!(
                        "WS-{}. Failed to send error to client. Cause: {}",
                        self.id,
                        err
                    );
                }
            }
        }

        if msg.is_text() {
            let deserialized: Result<Request, serde_json::Error> =
                serde_json::from_str(&msg.to_string());
            if let Ok(message) = deserialized {
                log::info!(
                    "WS-{}. Received message from client: {:#?}. IP: {}",
                    self.id,
                    message,
                    stream.get_ref().peer_addr()?
                );
                if let Err(err) = self.channels.client_request_tx.try_send(message) {
                    log::error!(
                        "WS-{}. Failed to pass command to the Processor: {}",
                        self.id,
                        err
                    );
                }
            } else {
                let message = Response::Error(ServerError::InvalidMessageFormat);
                if let Ok(text) = serde_json::to_string(&message) {
                    let _ = stream.send(Message::Text(Utf8Bytes::from(text)));
                }
            }
        }

        Ok(())
    }

    fn send_messages(&self, stream: &mut WSStream) {
        if let Ok(message) = self.channels.server_response_rx.try_recv() {
            if let Ok(serialized) = serde_json::to_string(&message) {
                let _ = stream.send(Message::text(serialized));
            } else {
                log::error!("WS-{}. Can't serialize message! {:#?}", self.id, message);
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum WsError {
    #[error("Authentication failed")]
    AuthFailed,

    #[error("Invalid password header")]
    InvalidPasswordHeader,
}
