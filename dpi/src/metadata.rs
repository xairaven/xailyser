use crate::wrapper::{OwnedPacket, PacketHeader};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkFrame {
    Parsed(FrameMetadata),
    RawPacket(OwnedPacket),
    RawMetadata(PacketHeader),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameMetadata {}
