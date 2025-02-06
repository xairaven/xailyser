use crate::context::Context;
use crate::ws::WsHandler;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;
use thiserror::Error;
use xailyser_common::messages::CONNECTION_TIMEOUT;

const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const APPROXIMATE_MAX_CONNECTIONS: usize = 5;

pub struct TcpHandler {
    runtime_ctx: Context,
}

impl TcpHandler {
    pub fn new(context: Context) -> Self {
        Self {
            runtime_ctx: context,
        }
    }

    pub fn start(&self) -> Result<(), TcpError> {
        if self.runtime_ctx.shutdown_flag.load(Ordering::Acquire) {
            return Ok(());
        }

        let address = SocketAddr::new(LOCALHOST, self.runtime_ctx.config.port);
        let server = TcpListener::bind(address).map_err(TcpError::ListenerBindError)?;
        server
            .set_nonblocking(true)
            .map_err(TcpError::FailedSetNonBlocking)?;

        log::info!("Listening on {}", address);
        self.listen(server);

        Ok(())
    }

    pub fn listen(&self, listener: TcpListener) {
        let mut ws_handles: Vec<JoinHandle<()>> =
            Vec::with_capacity(APPROXIMATE_MAX_CONNECTIONS);
        loop {
            if self.runtime_ctx.shutdown_flag.load(Ordering::Acquire) {
                log::info!("Shutting down TCP listening thread.");
                break;
            }

            match listener.accept() {
                Ok((tcp_stream, _)) => {
                    let runtime_ctx = self.runtime_ctx.clone();
                    let handle = thread::spawn(move || {
                        log::info!("TCP connection attempt found. Started WS thread.");
                        let result = WsHandler::new(runtime_ctx).start(tcp_stream);

                        if let Err(err) = result {
                            log::error!("{}. Terminated connection.", err);
                        }
                    });

                    ws_handles.push(handle);
                },
                Err(ref err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        || err.kind() == std::io::ErrorKind::TimedOut =>
                {
                    thread::sleep(CONNECTION_TIMEOUT);
                    continue;
                },
                Err(err) => {
                    log::error!("Connection failed! {}", err);
                },
            }
        }

        for handle in ws_handles {
            if let Err(err) = handle.join() {
                log::error!("Failed to join WS connection thread handle: {:?}", err);
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum TcpError {
    #[error("Failed to bind address.")]
    ListenerBindError(std::io::Error),

    #[error("Failed to set nonblocking mode on the server.")]
    FailedSetNonBlocking(std::io::Error),
}
