use crate::context::Context;
use bytes::Bytes;
use std::net::TcpStream;
use std::sync::atomic::Ordering;
use std::thread;
use thiserror::Error;
use tungstenite::handshake::server;
use tungstenite::http::{HeaderValue, StatusCode};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::{Message, Utf8Bytes, WebSocket};
use xailyser_common::auth;
use xailyser_common::messages::{CONNECTION_TIMEOUT, Request, Response, ServerError};

pub struct WsHandler {
    runtime_ctx: Context,
}

type WSStream = WebSocket<TcpStream>;

impl WsHandler {
    pub fn new(context: Context) -> Self {
        Self {
            runtime_ctx: context,
        }
    }

    pub fn start(&self, tcp_stream: TcpStream) -> Result<(), WsError> {
        let ws_stream = match self.connect(tcp_stream) {
            Ok(value) => {
                log::info!("Websocket connection established.");
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
                "Received a new handshake from {}:{}",
                peer_addr.ip(),
                peer_addr.port()
            );
        } else {
            log::info!("Received a new handshake!");
        }

        let server_password_header =
            HeaderValue::from_str(&self.runtime_ctx.encrypted_password)
                .map_err(|_| WsError::InvalidPasswordHeader)?;
        let check_authentication = |req: &server::Request, response: server::Response| {
            if let Some(given_password) = req.headers().get(auth::AUTH_HEADER) {
                if given_password.eq(&server_password_header) {
                    Ok(response)
                } else {
                    let response = server::Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Some(auth::errors::WRONG_PASSWORD_ERROR.to_string()))
                        .unwrap_or_default();
                    Err(response)
                }
            } else {
                let response = server::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Some(auth::errors::HEADER_NOT_FOUND_ERROR.to_string()))
                    .unwrap_or_default();
                Err(response)
            }
        };
        tungstenite::accept_hdr(tcp_stream, check_authentication)
            .map_err(|_| WsError::AuthFailed)
    }

    fn send_receive_messages(&self, mut stream: WSStream) {
        while !self.runtime_ctx.shutdown_flag.load(Ordering::Acquire) {
            if self.receive_messages(&mut stream).is_err() {
                return;
            }
            self.send_messages(&mut stream);
        }

        if let Ok(address) = stream.get_ref().peer_addr() {
            log::info!(
                "Closing connection ({}:{}), server is in shutdown process...",
                address.ip(),
                address.port()
            );
        } else {
            log::info!("Closing connection. IP & Port undefined.");
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
                        log::warn!("Connection closed without alerting about it.");
                        Err(err)
                    },
                    tungstenite::Error::Io(err)
                        if err.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        thread::sleep(CONNECTION_TIMEOUT);
                        Ok(())
                    },
                    tungstenite::Error::Io(err) => {
                        log::warn!("{}", err);
                        Err(tungstenite::Error::Io(err))
                    },
                    _ => {
                        log::error!("{}", err);
                        Ok(())
                    },
                };
            },
        };

        if msg.is_close() {
            log::info!("Client closed connection.");
            return Err(tungstenite::Error::ConnectionClosed);
        }

        if msg.is_ping() {
            let _ = stream.send(Message::Pong(Bytes::new()));
        }

        if msg.is_empty() || msg.is_binary() {
            log::warn!("Received empty or binary message.");
            let message = Response::Error(ServerError::InvalidMessageFormat);
            if let Ok(text) = serde_json::to_string(&message) {
                if let Err(err) = stream.send(Message::Text(Utf8Bytes::from(text))) {
                    log::error!("Failed to send error to client. Cause: {}", err);
                }
            }
        }

        if msg.is_text() {
            let deserialized: Result<Request, serde_json::Error> =
                serde_json::from_str(&msg.to_string());
            if let Ok(message) = deserialized {
                log::info!(
                    "Received message from client: {:#?}. IP: {}",
                    message,
                    stream.get_ref().peer_addr()?
                );
                if let Err(err) = self.runtime_ctx.client_request_tx.try_send(message) {
                    log::error!("Failed to pass command to the Processor: {}", err);
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
        if let Ok(message) = self.runtime_ctx.server_response_rx.try_recv() {
            if let Ok(serialized) = serde_json::to_string(&message) {
                let _ = stream.send(Message::text(serialized));
            } else {
                log::error!("Can't serialize message! {:#?}", message);
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
