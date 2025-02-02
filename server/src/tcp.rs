use crate::context::Context;
use crate::ws::WsHandler;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use thiserror::Error;

const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const APPROXIMATE_MAX_CONNECTIONS: usize = 5;
pub const WOULD_BLOCK_SLEEP_DELAY: Duration = Duration::from_millis(10);

pub struct TcpHandler {
    runtime_context: Context,

    shutdown_flag: Arc<AtomicBool>,
}

impl TcpHandler {
    pub fn new(runtime_context: Context, shutdown_flag: Arc<AtomicBool>) -> Self {
        Self {
            runtime_context,
            shutdown_flag,
        }
    }

    pub fn start(&mut self) -> Result<(), TcpError> {
        if self.shutdown_flag.load(Ordering::Acquire) {
            return Ok(());
        }

        let address = SocketAddr::new(LOCALHOST, self.runtime_context.port);
        let server = TcpListener::bind(address).map_err(TcpError::ListenerBindError)?;
        server
            .set_nonblocking(true)
            .map_err(TcpError::FailedSetNonBlocking)?;

        log::info!("Listening on {}", address);
        self.listen(server);

        Ok(())
    }

    pub fn listen(&mut self, listener: TcpListener) {
        let mut ws_handles: Vec<JoinHandle<()>> =
            Vec::with_capacity(APPROXIMATE_MAX_CONNECTIONS);
        loop {
            if self.shutdown_flag.load(Ordering::Acquire) {
                log::info!("Shutting down TCP listening thread.");
                break;
            }

            match listener.accept() {
                Ok((tcp_stream, _)) => {
                    let shutdown_flag_copy = Arc::clone(&self.shutdown_flag);
                    let runtime_context = self.runtime_context.clone();
                    let handle = thread::spawn(move || {
                        log::info!("TCP connection attempt found. Started WS thread.");
                        let result = WsHandler::new(shutdown_flag_copy)
                            .start(tcp_stream, runtime_context);

                        if let Err(err) = result {
                            log::error!("{}. Terminated connection.", err);
                        }
                    });

                    ws_handles.push(handle);
                },
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(WOULD_BLOCK_SLEEP_DELAY);
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
