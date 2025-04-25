use crate::ParseResult;
use crate::frame::FrameMetadata;
use crate::protocols::ProtocolId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ethernet {
    id: ProtocolId,
}

impl Default for Ethernet {
    fn default() -> Self {
        Self {
            id: ProtocolId::Ethernet,
        }
    }
}

pub fn parse<'a>(bytes: &'a [u8], metadata: &mut FrameMetadata) -> ParseResult<'a> {
    todo!()
}
