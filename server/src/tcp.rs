use crate::context;
use crate::context::Context;
use crate::ws::{WsError, WsHandlerBuilder};
use common::channel::BroadcastChannel;
use common::messages::CONNECTION_TIMEOUT;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use thiserror::Error;

const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub struct TcpHandler {
    frame_channel: Arc<Mutex<BroadcastChannel<dpi::metadata::NetworkFrame>>>,
    context: Arc<Mutex<Context>>,
    shutdown_flag: Arc<AtomicBool>,
    ws_active_counter: Arc<AtomicUsize>,
    ws_threads_counter: u16,
}

impl TcpHandler {
    pub fn start(&mut self) -> Result<(), TcpError> {
        if self.shutdown_flag.load(Ordering::Acquire) {
            return Ok(());
        }

        let port = context::lock(&self.context, |ctx| ctx.config.port);
        let address = SocketAddr::new(LOCALHOST, port);
        let server = TcpListener::bind(address).map_err(TcpError::ListenerBindError)?;
        server
            .set_nonblocking(true)
            .map_err(TcpError::FailedSetNonBlocking)?;

        log::info!("Listening on {}", address);
        self.listen(server);

        Ok(())
    }

    pub fn listen(&mut self, listener: TcpListener) {
        const APPROXIMATE_MAX_CONNECTIONS: usize = 5;
        let mut ws_handles: Vec<JoinHandle<()>> =
            Vec::with_capacity(APPROXIMATE_MAX_CONNECTIONS);
        loop {
            if self.shutdown_flag.load(Ordering::Acquire) {
                log::info!("Shutting down TCP listening thread.");
                break;
            }

            match listener.accept() {
                Ok((tcp_stream, _)) => {
                    let thread_counter = self.ws_threads_counter;
                    let handle = thread::Builder::new()
                        .name(format!("WS-Connection-{}", thread_counter))
                        .spawn({
                            let context = Arc::clone(&self.context);
                            let shutdown_flag = Arc::clone(&self.shutdown_flag);
                            let frame_receiver = match self.frame_channel.lock() {
                                Ok(mut guard) => guard.add_receiver(),
                                Err(err) => {
                                    log::error!("Broadcast channel lock failed: {}", err);
                                    std::process::exit(1);
                                },
                            };
                            let ws_active_counter = Arc::clone(&self.ws_active_counter);

                            move || {
                                log::info!(
                                    "TCP connection attempt found. Started WS thread."
                                );
                                let mut ws_handler = WsHandlerBuilder {
                                    id: thread_counter,
                                    frame_receiver,
                                    context,
                                    shutdown_flag,
                                    ws_active_counter,
                                }
                                .build();

                                if let Err(err) = ws_handler.start(tcp_stream) {
                                    match &err {
                                        WsError::AuthFailed(error_detailed) => {
                                            log::error!(
                                                "WS-{}. {}. {}",
                                                thread_counter,
                                                err,
                                                error_detailed
                                            )
                                        },
                                        _ => log::error!(
                                            "WS-{}. {}. Terminated connection.",
                                            thread_counter,
                                            err
                                        ),
                                    }
                                }
                            }
                        })
                        .unwrap_or_else(|err| {
                            log::error!(
                                "Failed to spawn web-socket thread (N: {}): {}",
                                self.ws_threads_counter,
                                err
                            );
                            std::process::exit(1);
                        });
                    self.ws_threads_counter += 1;
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

pub struct TcpHandlerBuilder {
    pub frame_channel: Arc<Mutex<BroadcastChannel<dpi::metadata::NetworkFrame>>>,
    pub context: Arc<Mutex<Context>>,
    pub shutdown_flag: Arc<AtomicBool>,
    pub ws_active_counter: Arc<AtomicUsize>,
}

impl TcpHandlerBuilder {
    pub fn build(self) -> TcpHandler {
        TcpHandler {
            frame_channel: self.frame_channel,
            context: self.context,
            shutdown_flag: self.shutdown_flag,
            ws_active_counter: self.ws_active_counter,

            ws_threads_counter: 0,
        }
    }
}
