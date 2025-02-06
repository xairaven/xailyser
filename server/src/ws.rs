use crate::context::Context;
use bytes::Bytes;
use crossbeam::channel::{Receiver, Sender};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use thiserror::Error;
use tungstenite::handshake::server;
use tungstenite::http::{HeaderValue, StatusCode};
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::protocol::CloseFrame;
use tungstenite::{Message, Utf8Bytes, WebSocket};
use xailyser_common::auth;
use xailyser_common::messages::{Request, Response, ServerError, CONNECTION_TIMEOUT};

pub struct WsHandler {
    shutdown_flag: Arc<AtomicBool>,
}

type WSStream = WebSocket<TcpStream>;

impl WsHandler {
    pub fn new(shutdown_flag: Arc<AtomicBool>) -> Self {
        Self { shutdown_flag }
    }

    pub fn start(&self, tcp_stream: TcpStream, context: Context) -> Result<(), WsError> {
        let ws_stream = match self.connect(tcp_stream, context.config.password) {
            Ok(value) => {
                log::info!("Websocket connection established.");
                value
            },
            Err(err) => return Err(err),
        };

        self.send_receive_messages(
            ws_stream,
            context.client_request_tx,
            context.server_response_rx,
        );
        Ok(())
    }

    fn connect(
        &self, tcp_stream: TcpStream, server_password: String,
    ) -> Result<WSStream, WsError> {
        if let Ok(peer_addr) = &tcp_stream.peer_addr() {
            log::info!(
                "Received a new handshake from {}:{}",
                peer_addr.ip(),
                peer_addr.port()
            );
        } else {
            log::info!("Received a new handshake!");
        }

        let server_password_header = HeaderValue::from_str(server_password.as_str())
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

    fn send_receive_messages(
        &self, mut stream: WSStream, client_request_tx: Sender<Request>,
        server_response_rx: Receiver<Response>,
    ) {
        while !self.shutdown_flag.load(Ordering::Acquire) {
            if self
                .receive_messages(&mut stream, &client_request_tx)
                .is_err()
            {
                return;
            }
            self.send_messages(&mut stream, &server_response_rx);
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

    fn receive_messages(
        &self, stream: &mut WSStream, client_request_tx: &Sender<Request>,
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
                }
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
                if let Err(err) = client_request_tx.try_send(message) {
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

    fn send_messages(
        &self, stream: &mut WSStream, server_response_rx: &Receiver<Response>,
    ) {
        if let Ok(message) = server_response_rx.try_recv() {
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
