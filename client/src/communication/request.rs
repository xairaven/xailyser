use common::compression::compress;
use common::messages::Request;
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::{Bytes, Message};

#[derive(Debug)]
pub enum UiClientRequest {
    Request(Request),
    CloseConnection,
    Ping,
}

impl UiClientRequest {
    pub fn into_message(self, is_compression_enabled: bool) -> Result<Message, String> {
        match self {
            UiClientRequest::Request(request) => {
                let serialized =
                    serde_json::to_string(&request).map_err(|err| err.to_string())?;

                if is_compression_enabled {
                    let compressed =
                        compress(&serialized).map_err(|err| err.to_string())?;
                    Ok(Message::Binary(Bytes::from(compressed)))
                } else {
                    Ok(Message::text(serialized))
                }
            },
            UiClientRequest::CloseConnection => {
                let message = Message::Close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: Default::default(),
                }));
                Ok(message)
            },
            UiClientRequest::Ping => Ok(Message::Ping(Bytes::new())),
        }
    }
}
