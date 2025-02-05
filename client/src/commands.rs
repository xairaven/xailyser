use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::protocol::CloseFrame;
use tungstenite::{Bytes, Message};
use xailyser_common::messages::ClientRequest;

#[derive(Debug)]
pub enum UiCommand {
    ClientRequest(ClientRequest),
    CloseConnection,
    Ping,
}

impl UiCommand {
    pub fn into_message(self) -> Result<Message, serde_json::Error> {
        match self {
            UiCommand::ClientRequest(request) => {
                let serialized = serde_json::to_string(&request)?;
                Ok(Message::text(serialized))
            },
            UiCommand::CloseConnection => {
                let message = Message::Close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: Default::default(),
                }));
                Ok(message)
            },
            UiCommand::Ping => Ok(Message::Ping(Bytes::new())),
        }
    }
}
