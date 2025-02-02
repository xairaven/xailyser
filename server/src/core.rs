use crate::config::Config;
use crate::context::Context;
use crate::net::PacketSniffer;
use crate::tcp::TcpHandler;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use xailyser_common::messages::ClientRequest;

pub const ORDERING_SLEEP_DELAY: Duration = Duration::from_millis(10);

pub fn start(config: Config) {
    let context = Context::new(&config);

    let shutdown_flag = Arc::new(AtomicBool::new(false));
    if ctrlc::set_handler({
        let shutdown_flag = Arc::clone(&shutdown_flag);
        move || {
            shutdown_flag.store(true, Ordering::Release);
        }
    })
    .is_err()
    {
        log::error!("Error setting Ctrl-C handler. Shutting down.");
        std::process::exit(1);
    }

    let shutdown_flag_copy = Arc::clone(&shutdown_flag);
    let packet_sniffer_handle = thread::spawn(move || {
        log::info!("Packet sniffing thread started.");
        PacketSniffer::new(shutdown_flag_copy).start();
    });

    let shutdown_flag_copy = Arc::clone(&shutdown_flag);
    let runtime_context = context.clone();
    let tcp_thread_handle = thread::spawn(move || {
        log::info!("TCP Listening thread started.");
        let result = TcpHandler::new(runtime_context, shutdown_flag_copy).start();

        if let Err(err) = result {
            log::error!("TCP Error: {}", err);
        }
    });

    while !shutdown_flag.load(Ordering::Acquire) {
        if let Ok(request) = context.client_request_rx.try_recv() {
            match request {
                ClientRequest::RequestInterfaces => {
                    todo!()
                },
                ClientRequest::SetInterface(_) => {
                    todo!()
                },
                ClientRequest::ChangePassword(_) => {
                    todo!()
                },
                ClientRequest::Reboot => {
                    todo!()
                },
            }
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
