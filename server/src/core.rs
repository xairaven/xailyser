use crate::config::Config;
use crate::context::Context;
use crate::net::PacketSniffer;
use crate::request;
use crate::tcp::TcpHandler;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub const ORDERING_SLEEP_DELAY: Duration = Duration::from_millis(100);

pub fn start(config: Config) {
    let mut context = match Context::new(config) {
        Ok(value) => value,
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        },
    };

    let shutdown_flag_copy = Arc::clone(&context.shutdown_flag);
    if ctrlc::set_handler({
        move || {
            shutdown_flag_copy.store(true, Ordering::Release);
        }
    })
    .is_err()
    {
        log::error!("Error setting Ctrl-C handler. Shutting down.");
        std::process::exit(1);
    }

    let packet_sniffer_handle: Option<JoinHandle<()>> =
        if context.network_interface.is_some() {
            let runtime_context = context.clone();
            let handle = thread::spawn(move || {
                log::info!("Packet sniffing thread started.");
                let shutdown_flag = Arc::clone(&runtime_context.shutdown_flag);
                let result = PacketSniffer::new(runtime_context).start();

                if let Err(err) = result {
                    log::error!("Network Error: {}", err);
                    shutdown_flag.store(true, Ordering::Release);
                }
            });
            Some(handle)
        } else {
            None
        };

    let runtime_context = context.clone();
    let tcp_thread_handle = thread::spawn(move || {
        log::info!("TCP Listening thread started.");
        let shutdown_flag = Arc::clone(&runtime_context.shutdown_flag);
        let result = TcpHandler::new(runtime_context).start();

        if let Err(err) = result {
            log::error!("TCP Error: {}", err);
            shutdown_flag.store(true, Ordering::Release);
        }
    });

    while !context.shutdown_flag.load(Ordering::Acquire) {
        if let Ok(request) = context.client_request_rx.try_recv() {
            request::core::process(&mut context, request);
        } else {
            thread::sleep(ORDERING_SLEEP_DELAY);
        }
    }

    // Joining packet sniffing thread
    if let Some(handle) = packet_sniffer_handle {
        if handle.join().is_err() {
            log::error!("Failed to join packet sniffing thread!");
        }
    }

    // Joining TCP Thread
    if tcp_thread_handle.join().is_err() {
        log::error!("Failed to join TCP listening thread!");
    }

    log::info!("Shutdown complete");
}
