use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    AuthenticateRequest { password: String },
    AuthenticateReply(bool),
}
