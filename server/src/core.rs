use crate::config::Config;
use crate::ws::ConnectionThread;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub fn start(config: Config) {
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

    let address = SocketAddr::new(ADDRESS, config.port);
    let server = TcpListener::bind(address).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });
    server.set_nonblocking(true).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });
    log::info!("Listening on {}", address);

    let mut handles = Vec::new();
    loop {
        if shutdown_flag.load(Ordering::Acquire) {
            log::info!("Shutting down. Stop listening...");
            break;
        }

        match server.accept() {
            Ok((tcp_stream, _)) => {
                let encrypted_password = config.password.clone();
                let shutdown_flag = Arc::clone(&shutdown_flag);
                let handle = thread::spawn(move || {
                    ConnectionThread::new(shutdown_flag)
                        .start(tcp_stream, encrypted_password);
                });
                handles.push(handle);
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

    for handle in handles {
        if let Err(err) = handle.join() {
            eprintln!("Failed to join handle: {:?}", err);
        }
    }
    log::info!("Shutdown complete");
}
