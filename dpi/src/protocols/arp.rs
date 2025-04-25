use crate::frame::FrameMetadata;
use crate::protocols::ProtocolId;
use crate::{ParseResult, ParseableProtocol};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Arp {
    id: ProtocolId,
}

impl Default for Arp {
    fn default() -> Self {
        Self {
            id: ProtocolId::ARP,
        }
    }
}

impl ParseableProtocol<'_> for Arp {
    fn id(&self) -> &ProtocolId {
        &self.id
    }

    fn parse<'a>(
        &self, bytes: &'a [u8], metadata: &mut FrameMetadata,
    ) -> ParseResult<'a> {
        todo!()
    }
}
