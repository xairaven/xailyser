use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use thiserror::Error;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::http::{HeaderValue, StatusCode};
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::protocol::CloseFrame;
use tungstenite::WebSocket;
use xailyser_common::auth;

pub struct ConnectionThread {
    shutdown_flag: Arc<AtomicBool>,
}

type WSStream = WebSocket<TcpStream>;

impl ConnectionThread {
    pub fn new(shutdown_flag: Arc<AtomicBool>) -> Self {
        Self { shutdown_flag }
    }

    pub fn start(&self, tcp_stream: TcpStream, password: String) {
        let ws_stream = match self.connect(tcp_stream, password) {
            Ok(value) => {
                log::info!("Websocket connection established.");
                value
            },
            Err(err) => {
                log::info!("{}", err);
                return;
            },
        };

        self.handle_messages(ws_stream);
    }

    fn connect(
        &self, tcp_stream: TcpStream, server_password: String,
    ) -> Result<WSStream, NetError> {
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
            .map_err(|_| NetError::InvalidPasswordHeader)?;
        let check_authentication = |req: &Request, response: Response| {
            if let Some(given_password) = req.headers().get(auth::AUTH_HEADER) {
                if given_password.eq(&server_password_header) {
                    Ok(response)
                } else {
                    let response = Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Some(auth::errors::WRONG_PASSWORD_ERROR.to_string()))
                        .unwrap_or_default();
                    Err(response)
                }
            } else {
                let response = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Some(auth::errors::HEADER_NOT_FOUND_ERROR.to_string()))
                    .unwrap_or_default();
                Err(response)
            }
        };
        tungstenite::accept_hdr(tcp_stream, check_authentication)
            .map_err(|_| NetError::AuthFailed)
    }

    fn handle_messages(&self, mut stream: WSStream) {
        while !self.shutdown_flag.load(Ordering::Acquire) {
            let msg = match stream.read() {
                Ok(value) => value,
                Err(err) => match err {
                    tungstenite::Error::ConnectionClosed
                    | tungstenite::Error::AlreadyClosed => {
                        log::warn!("Connection closed without alerting about it.");
                        return;
                    },
                    tungstenite::Error::Io(err)
                        if err.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        thread::sleep(std::time::Duration::from_millis(50));
                        continue;
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
}

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Authentication failed.")]
    AuthFailed,

    #[error("Invalid password header.")]
    InvalidPasswordHeader,
}
