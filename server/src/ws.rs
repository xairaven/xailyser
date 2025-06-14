use crate::context::Context;
use crate::{context, request};
use bytes::Bytes;
use common::auth;
use common::compression::{compress, decompress};
use common::messages::{CONNECTION_TIMEOUT, Request, Response, ServerError};
use crossbeam::channel::{Receiver, RecvTimeoutError};
use dpi::dto::frame::FrameType;
use std::collections::VecDeque;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    compression: bool,
    context: Arc<Mutex<Context>>,
    frame_receiver: Receiver<FrameType>,
    response_queue: VecDeque<Response>,
    shutdown_flag: Arc<AtomicBool>,

    _connection_guard: WsConnectionGuard,
}

type WSStream = WebSocket<TcpStream>;
const BATCH_SIZE: usize = 100;

impl WsHandler {
    pub fn start(&mut self, tcp_stream: TcpStream) -> Result<(), WsError> {
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
        let server_compression_header = context::lock(&self.context, |ctx| {
            HeaderValue::from_str(&ctx.compression.to_string())
                .map_err(|_| WsError::InvalidCompressionHeader)
        })?;

        let check_authentication = |req: &server::Request, response: server::Response| {
            let password_header = req.headers().get(auth::AUTH_HEADER);
            let compression_header = req.headers().get(auth::COMPRESSION_HEADER);

            match password_header {
                Some(given_password) if given_password == server_password_header => {},
                Some(_) => {
                    return Err(server::Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Some(auth::errors::WRONG_PASSWORD.to_string()))
                        .unwrap_or_default());
                },
                None => {
                    return Err(server::Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Some(auth::errors::PASSWORD_HEADER_NOT_FOUND.to_string()))
                        .unwrap_or_default());
                },
            };

            match compression_header {
                Some(given_compression)
                    if given_compression == server_compression_header =>
                {
                    Ok(response)
                },
                Some(_) => Err(server::Response::builder()
                    .status(StatusCode::PRECONDITION_FAILED)
                    .body(Some(auth::errors::WRONG_COMPRESSION.to_string()))
                    .unwrap_or_default()),
                None => Err(server::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Some(auth::errors::COMPRESSION_HEADER_NOT_FOUND.to_string()))
                    .unwrap_or_default()),
            }
        };

        let stream = tungstenite::accept_hdr(tcp_stream, check_authentication)
            .map_err(|err| WsError::AuthFailed(err.to_string()))?;

        stream
            .get_ref()
            .set_nonblocking(true)
            .map_err(|_| WsError::FailedSetNonBlockingStream)?;

        Ok(stream)
    }

    fn send_receive_messages(&mut self, mut stream: WSStream) {
        while !self.shutdown_flag.load(Ordering::Acquire) {
            match self.frame_receiver.recv_timeout(CONNECTION_TIMEOUT) {
                Ok(first) => {
                    self.response_queue.push_back(Response::Data(first));
                    log::debug!(
                        "WS-{}. Pushing data from frame receiver to queue.",
                        self.id
                    );

                    // Catching others without waiting
                    for _ in 1..=BATCH_SIZE {
                        match self.frame_receiver.try_recv() {
                            Ok(frame) => {
                                self.response_queue.push_back(Response::Data(frame));
                                log::debug!(
                                    "WS-{}. Pushing data from frame receiver to queue.",
                                    self.id
                                );
                            },
                            _ => break,
                        }
                    }
                    self.send_messages(&mut stream);
                },
                Err(err) if err == RecvTimeoutError::Disconnected => {
                    log::error!(
                        "WS-{}. Broadcast channel sender disconnected. {}",
                        self.id,
                        err
                    );
                    break;
                },
                _ => {},
            }
            if let Err(err) = self.receive_messages(&mut stream) {
                log::debug!(
                    "WS-{}. Got error while receiving messages: {}",
                    self.id,
                    err
                );
                return;
            }
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

    fn receive_messages(
        &mut self, stream: &mut WSStream,
    ) -> Result<(), Box<tungstenite::Error>> {
        log::debug!("WS-{}. Reading next message...", self.id);
        let msg = match stream.read() {
            Ok(msg) => msg,
            Err(err) => return self.handle_read_error(err),
        };
        log::debug!("WS-{}. Message successfully read.", self.id);

        if msg.is_close() {
            log::info!("WS-{}. Client closed connection.", self.id);
            return Err(Box::new(tungstenite::Error::ConnectionClosed));
        }

        // Heartbeat system
        if msg.is_ping() {
            let _ = stream.send(Message::Pong(Bytes::new()));
            log::debug!("WS-{}. Got ping!", self.id);
            return Ok(());
        }

        match self.compression {
            true => self.handle_binary_compressed(msg, stream)?,
            false => self.handle_text_uncompressed(msg, stream)?,
        }

        Ok(())
    }

    fn send_messages(&mut self, stream: &mut WSStream) {
        while let Some(response) = self.response_queue.pop_front() {
            log::debug!("WS-{}. Response from queue popped out.", self.id);
            if let Ok(serialized) = serde_json::to_string(&response) {
                if self.compression {
                    match compress(&serialized) {
                        Ok(bytes) => {
                            log::debug!(
                                "WS-{}. Will send compressed message now..",
                                self.id
                            );
                            let _ = stream.send(Message::Binary(Bytes::from(bytes)));
                            log::debug!("WS-{}. Message successfully sent.", self.id);
                        },
                        Err(_) => {
                            log::error!(
                                "WS-{}. Can't compress message! {:#?}",
                                self.id,
                                response
                            );
                        },
                    }
                } else {
                    log::debug!("WS-{}. Will send uncompressed message now..", self.id);
                    let _ = stream.send(Message::text(serialized));
                    log::debug!("WS-{}. Message successfully sent.", self.id);
                }
            } else {
                log::error!("WS-{}. Can't serialize message! {:#?}", self.id, response);
            }
        }
    }

    fn handle_read_error(
        &self, err: tungstenite::Error,
    ) -> Result<(), Box<tungstenite::Error>> {
        use tungstenite::Error::*;
        match err {
            ConnectionClosed => {
                log::info!("WS-{}. Connection closed.", self.id);
                Err(Box::new(err))
            },
            AlreadyClosed => {
                log::warn!("WS-{}. Connection closed without alerting.", self.id);
                Err(Box::new(err))
            },
            Io(io_err) if io_err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(CONNECTION_TIMEOUT);
                log::debug!("WS-{}. Connection timeout. Sleeping...", self.id);
                Ok(())
            },
            Io(io_err) => {
                log::warn!("WS-{}. {}", self.id, io_err);
                Err(Box::new(Io(io_err)))
            },
            _ => {
                log::error!("WS-{}. {}. {:#?}", self.id, err, err);
                Ok(())
            },
        }
    }

    fn handle_binary_compressed(
        &mut self, msg: Message, stream: &mut WSStream,
    ) -> Result<(), Box<tungstenite::Error>> {
        if msg.is_empty() || msg.is_text() {
            log::warn!("WS-{}. Received empty or non-compressed message.", self.id);
            self.send_error_response(stream)?;
            return Ok(());
        }

        if msg.is_binary() {
            let decompressed = decompress(&msg.into_data())
                .map_err(|err| Box::new(tungstenite::Error::from(err)))?;
            self.process_message(&decompressed, stream)?;
        }

        Ok(())
    }

    fn handle_text_uncompressed(
        &mut self, msg: Message, stream: &mut WSStream,
    ) -> Result<(), Box<tungstenite::Error>> {
        if msg.is_empty() || msg.is_binary() {
            log::warn!("WS-{}. Received empty or binary message.", self.id);
            self.send_error_response(stream)?;
            return Ok(());
        }

        if msg.is_text() {
            self.process_message(&msg.to_string(), stream)?;
        }

        Ok(())
    }

    fn process_message(
        &mut self, text: &str, stream: &mut WSStream,
    ) -> Result<(), Box<tungstenite::Error>> {
        match serde_json::from_str::<Request>(text) {
            Ok(message) => {
                log::info!(
                    "WS-{}. Received message from client: {:#?}. IP: {}",
                    self.id,
                    message,
                    stream
                        .get_ref()
                        .peer_addr()
                        .map_err(|err| Box::new(tungstenite::Error::from(err)))?
                );

                if let Some(response) =
                    request::core::process(message, &self.context, &self.shutdown_flag)
                {
                    self.response_queue.push_back(response);
                    log::debug!(
                        "WS-{}. Pushed back processed request to queue.",
                        self.id
                    );
                }
            },
            Err(_) => {
                self.send_error_response(stream)?;
            },
        }

        Ok(())
    }

    fn send_error_response(
        &self, stream: &mut WSStream,
    ) -> Result<(), Box<tungstenite::Error>> {
        let message = Response::Error(ServerError::InvalidMessageFormat);
        if let Ok(text) = serde_json::to_string(&message) {
            if self.compression {
                let compressed = compress(&text)
                    .map_err(|err| Box::new(tungstenite::Error::from(err)))?;
                log::debug!("WS-{}. Trying to send error response.. (bytes)", self.id);
                let _ = stream.send(Message::Binary(Bytes::from(compressed)));
                log::debug!("WS-{}. Error response successfully sent! (bytes)", self.id);
            } else {
                log::debug!("WS-{}. Trying to send error response.. (text)", self.id);
                let _ = stream.send(Message::Text(Utf8Bytes::from(text)));
                log::debug!("WS-{}. Error response successfully sent! (text)", self.id);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum WsError {
    #[error("Authentication failed")]
    AuthFailed(String),

    #[error("Invalid compression header")]
    InvalidCompressionHeader,

    #[error("Invalid password header")]
    InvalidPasswordHeader,

    #[error("Failed to set non-blocking stream")]
    FailedSetNonBlockingStream,
}

pub struct WsHandlerBuilder {
    pub id: u16,
    pub frame_receiver: Receiver<FrameType>,
    pub context: Arc<Mutex<Context>>,
    pub shutdown_flag: Arc<AtomicBool>,
    pub ws_active_counter: Arc<AtomicUsize>,
}

impl WsHandlerBuilder {
    pub fn build(self) -> WsHandler {
        let compression = context::lock(&self.context, |context| context.compression);
        let connection_guard = WsConnectionGuard::new(self.ws_active_counter);

        WsHandler {
            id: self.id,
            compression,
            context: self.context,
            frame_receiver: self.frame_receiver,
            response_queue: VecDeque::new(),
            shutdown_flag: self.shutdown_flag,

            _connection_guard: connection_guard,
        }
    }
}

pub struct WsConnectionGuard {
    counter: Arc<AtomicUsize>,
}

impl WsConnectionGuard {
    pub fn new(counter: Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::Release);
        Self { counter }
    }
}

impl Drop for WsConnectionGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Release);
    }
}
