use std::net::TcpStream;

pub fn start(stream: TcpStream) {
    let mut websocket = match tungstenite::accept(stream) {
        Ok(value) => value,
        Err(err) => {
            log::error!("{}", err);
            return;
        },
    };

    loop {
        let msg = match websocket.read() {
            Ok(value) => value,
            Err(err) => {
                log::error!("{}", err);
                continue;
            },
        };

        if msg.is_binary() || msg.is_text() {
            websocket.send(msg).unwrap_or_else(|err| {
                log::error!("{}", err);
            });
        }
    }
}
