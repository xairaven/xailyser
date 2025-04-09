use crate::channels::Channels;
use crate::config::Config;
use crate::context::Context;
use crate::net::PacketSniffer;
use crate::request;
use crate::tcp::TcpHandler;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn start(config: Config) {
    let context = Arc::new(Mutex::new(match Context::new(config) {
        Ok(ctx) => ctx,
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        },
    }));
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let channels = Channels::default();

    if let Err(err) = ctrlc::set_handler({
        let shutdown_flag = Arc::clone(&shutdown_flag);
        move || {
            shutdown_flag.store(true, Ordering::Release);
        }
    }) {
        log::error!("Error setting Ctrl-C handler: {}", err);
        std::process::exit(1);
    }

    let packet_sniffer_handle = {
        let is_some_interface = match context.lock() {
            Ok(guard) => guard.network_interface.is_some(),
            Err(err) => {
                log::error!("{}", err);
                std::process::exit(1);
            },
        };

        if !is_some_interface {
            None
        } else {
            let context = Arc::clone(&context);
            let shutdown_flag = Arc::clone(&shutdown_flag);
            Some(
                thread::Builder::new()
                    .name("Network-Sniffing-Thread".to_owned())
                    .spawn(move || {
                        log::info!("Packet sniffing thread started.");
                        let result =
                            PacketSniffer::new(context, shutdown_flag.clone()).start();

                        if let Err(err) = result {
                            log::error!("Network Error: {}", err);
                            shutdown_flag.store(true, Ordering::Release);
                        }
                    })
                    .unwrap_or_else(|err| {
                        log::error!("Failed to spawn sniffing thread: {}", err);
                        std::process::exit(1);
                    }),
            )
        }
    };

    let tcp_thread_handle = thread::Builder::new()
        .name("TCP-Thread".to_owned())
        .spawn({
            let context = Arc::clone(&context);
            let shutdown_flag = Arc::clone(&shutdown_flag);
            let channels = channels.clone();
            move || {
                log::info!("TCP Listening thread started.");
                if let Err(err) =
                    TcpHandler::new(channels, context, Arc::clone(&shutdown_flag)).start()
                {
                    log::error!("TCP Error: {}", err);
                    shutdown_flag.store(true, Ordering::Release);
                }
            }
        })
        .unwrap_or_else(|err| {
            log::error!("Failed to spawn TCP thread: {}", err);
            std::process::exit(1);
        });

    // Client request handling (Client -> WS Thread -> This thread)
    while !shutdown_flag.load(Ordering::Acquire) {
        if let Ok(request) = channels.client_request_rx.try_recv() {
            request::core::process(request, &context, &channels, &shutdown_flag);
        } else {
            const SLEEP_DELAY: Duration = Duration::from_millis(100);
            thread::sleep(SLEEP_DELAY);
        }
    }

    // Joining threads
    if let Some(handle) = packet_sniffer_handle {
        if handle.join().is_err() {
            log::error!("Failed to join packet sniffing thread!");
        }
    }
    if tcp_thread_handle.join().is_err() {
        log::error!("Failed to join TCP listening thread!");
    }

    log::info!("Shutdown complete");
}
