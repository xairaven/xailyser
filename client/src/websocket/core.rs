use crate::context::Context;
use crate::websocket;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use thiserror::Error;

pub fn connect(context: Arc<Mutex<Context>>) -> Result<JoinHandle<()>, NetError> {
    let address = context
        .try_lock()
        .map_err(|_| NetError::ContextLocked)?
        .deref()
        .address
        .ok_or(NetError::SocketAddrNotInitialized)?;

    let request = format!("ws://{}:{}/socket", address.ip(), address.port());
    let (socket, response) = match tungstenite::connect(request) {
        Ok((socket, response)) => (socket, response),
        Err(err) => return Err(NetError::ConnectionFailed(err, address)),
    };
    log::info!("Connected! Status: {}", response.status());

    let handle = thread::spawn(move || {
        websocket::thread::start(socket, context);
    });

    Ok(handle)
}

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Failed to connect.")]
    ConnectionFailed(tungstenite::Error, SocketAddr),

    #[error("Cannot get data because of locking.")]
    ContextLocked,

    #[error("Address & port not initialized.")]
    SocketAddrNotInitialized,
}

impl NetError {
    pub fn additional_info(&self) -> Option<String> {
        match self {
            NetError::ConnectionFailed(err, socket) => {
                let message = format!(
                    "Connection failed to {}:{}. Details: {}",
                    socket.ip(),
                    socket.port(),
                    err
                );
                Some(message)
            },
            _ => None,
        }
    }
}
