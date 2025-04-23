use crate::config::Config;
use crate::context;
use crate::context::Context;
use crate::net::PacketSnifferBuilder;
use crate::tcp::TcpHandlerBuilder;
use common::channel::BroadcastChannel;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    let frame_channel = Arc::new(Mutex::new(BroadcastChannel::<
        dpi::metadata::NetworkFrame,
    >::new()));
    let ws_active_counter = Arc::new(AtomicUsize::new(0));

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
            let ws_active_counter = Arc::clone(&ws_active_counter);
            Some(
                thread::Builder::new()
                    .name("Network-Sniffing-Thread".to_owned())
                    .spawn(move || {
                        log::info!("Packet sniffing thread started.");
                        let result = PacketSnifferBuilder {
                            frame_channel,
                            context,
                            shutdown_flag: shutdown_flag.clone(),
                            ws_active_counter,
                        }
                        .build();
                        let mut sniffer = match result {
                            Ok(value) => value,
                            Err(err) => {
                                log::error!("Network Error: {}", err);
                                shutdown_flag.store(true, Ordering::Release);
                                return;
                            },
                        };
                        if let Err(err) = sniffer.listen() {
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
            let ws_active_counter = Arc::clone(&ws_active_counter);
            move || {
                log::info!("TCP Listening thread started.");
                let mut tcp_handler = TcpHandlerBuilder {
                    frame_channel,
                    context,
                    shutdown_flag: shutdown_flag.clone(),
                    ws_active_counter,
                }
                .build();

                if let Err(err) = tcp_handler.start() {
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
