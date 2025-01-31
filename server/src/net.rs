use std::net::TcpStream;
use thiserror::Error;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::http::{HeaderValue, StatusCode};
use tungstenite::WebSocket;
use xailyser_common::auth;

type WSStream = WebSocket<TcpStream>;

pub fn connect(
    tcp_stream: TcpStream, server_password: String,
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

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Authentication failed.")]
    AuthFailed,

    #[error("Invalid password header.")]
    InvalidPasswordHeader,
}
