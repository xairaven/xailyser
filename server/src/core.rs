use crate::config::Config;
use crate::net;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::thread;

const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub fn start(config: Config) {
    let address = SocketAddr::new(ADDRESS, config.port);
    let server = TcpListener::bind(address).unwrap_or_else(|err| {
        log::error!("{}", err);
        std::process::exit(1);
    });
    log::info!("Listening on {}", address);

    let mut handles = Vec::new();
    for stream in server.incoming() {
        match stream {
            Ok(tcp_stream) => {
                let handle = thread::spawn(move || {
                    let ws_stream = match tungstenite::accept(tcp_stream) {
                        Ok(value) => {
                            log::info!("WebSocket connection established!");
                            value
                        },
                        Err(err) => {
                            log::error!("{}", err);
                            return;
                        },
                    };
                    net::handle_messages(ws_stream);
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
