use crate::frame::FrameMetadata;
use crate::protocols::ProtocolId;
use crate::{ParseResult, ParseableProtocol};
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

impl ParseableProtocol<'_> for Ethernet {
    fn id(&self) -> &ProtocolId {
        &self.id
    }

    fn parse<'a>(
        &self, bytes: &'a [u8], metadata: &mut FrameMetadata,
    ) -> ParseResult<'a> {
        todo!()
    }
}
