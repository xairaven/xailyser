use crate::config::Config;
use crate::context::Context;
use crate::net::NetThreadHandler;
use crate::ws::ConnectionThread;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;

const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub fn start(config: Config) {
    let address = SocketAddr::new(ADDRESS, config.port);
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let context = Arc::new(Mutex::new(Context { config }));

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

    let net_shutdown_flag = Arc::clone(&shutdown_flag);
    let net_context = Arc::clone(&context);
    let net_thread_handle = thread::spawn(move || {
        log::info!("Net capture thread started.");
        let result = NetThreadHandler::new(net_context, net_shutdown_flag).start();

        if let Err(err) = result {
            log::error!("{}. Shutdown server.", err);
            std::process::exit(1);
        }
    });

    let server = TcpListener::bind(address).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });
    server.set_nonblocking(true).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });
    log::info!("Listening on {}", address);

    let mut connection_thread_handles = Vec::new();
    loop {
        if shutdown_flag.load(Ordering::Acquire) {
            log::info!("Shutting down. Stop listening...");
            break;
        }

        match server.accept() {
            Ok((tcp_stream, _)) => {
                let shutdown_flag = Arc::clone(&shutdown_flag);
                let context = Arc::clone(&context);
                let handle = thread::spawn(move || {
                    let result =
                        ConnectionThread::new(context, shutdown_flag).start(tcp_stream);

                    if let Err(err) = result {
                        log::error!("{}. Terminated connection.", err);
                    }
                });
                connection_thread_handles.push(handle);
            },
            Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(50));
                continue;
            },
            Err(err) => {
                log::error!("Connection failed! {}", err);
            },
        }
    }

    for handle in connection_thread_handles {
        if let Err(err) = handle.join() {
            eprintln!("Failed to join connection thread handle: {:?}", err);
        }
    }
    if let Err(err) = net_thread_handle.join() {
        eprintln!("Failed to join net-capture thread handle: {:?}", err);
    }
    log::info!("Shutdown complete");
}
