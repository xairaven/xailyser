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
    pub fn into_message(self) -> Result<Message, serde_json::Error> {
        match self {
            UiClientRequest::Request(request) => {
                let serialized = serde_json::to_string(&request)?;
                Ok(Message::text(serialized))
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
