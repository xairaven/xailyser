use crate::config::Config;
use crate::context;
use crate::context::Context;
use crate::net::PacketSniffer;
use crate::tcp::TcpHandler;
use common::channel::BroadcastChannel;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn start(config: Config) {
    let context = Arc::new(Mutex::new(match Context::new(config) {
        Ok(ctx) => ctx,
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        },
    }));
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let frame_channel = BroadcastChannel::<dpi::metadata::NetworkFrame>::new();
    let frame_channel = Arc::new(Mutex::new(frame_channel));

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
        let is_some_interface =
            context::lock(&context, |ctx| ctx.network_interface.is_some());

        if !is_some_interface {
            None
        } else {
            let context = Arc::clone(&context);
            let shutdown_flag = Arc::clone(&shutdown_flag);
            let frame_channel = Arc::clone(&frame_channel);
            Some(
                thread::Builder::new()
                    .name("Network-Sniffing-Thread".to_owned())
                    .spawn(move || {
                        log::info!("Packet sniffing thread started.");
                        let result = PacketSniffer::new(
                            frame_channel,
                            context,
                            shutdown_flag.clone(),
                        )
                        .start();

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
            move || {
                log::info!("TCP Listening thread started.");
                if let Err(err) =
                    TcpHandler::new(frame_channel, context, Arc::clone(&shutdown_flag))
                        .start()
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
