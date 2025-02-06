use crate::config::Config;
use crate::context::Context;
use crate::net::PacketSniffer;
use crate::request;
use crate::tcp::TcpHandler;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub const ORDERING_SLEEP_DELAY: Duration = Duration::from_millis(100);

pub fn start(config: Config) {
    let mut context = Context::new(config);

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

    let runtime_context = context.clone();
    let packet_sniffer_handle = thread::spawn(move || {
        log::info!("Packet sniffing thread started.");
        PacketSniffer::new(runtime_context).start();
    });

    let runtime_context = context.clone();
    let tcp_thread_handle = thread::spawn(move || {
        log::info!("TCP Listening thread started.");
        let result = TcpHandler::new(runtime_context).start();

        if let Err(err) = result {
            log::error!("TCP Error: {}", err);
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
    if packet_sniffer_handle.join().is_err() {
        log::error!("Failed to join packet sniffing thread!");
    }
    // Joining TCP Thread
    if tcp_thread_handle.join().is_err() {
        log::error!("Failed to join TCP listening thread!");
    }

    log::info!("Shutdown complete");
}
