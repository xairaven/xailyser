use crate::config::Config;
use crate::websocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::thread;

const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub fn start(config: Config) {
    let address = SocketAddr::new(ADDRESS, config.port);
    let server = TcpListener::bind(address).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });

    let mut handles = Vec::new();
    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                let handle = thread::spawn(move || {
                    websocket::thread::start(stream);
                });
                handles.push(handle);
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
}
