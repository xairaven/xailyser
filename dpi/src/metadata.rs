use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkFrame {
    Parsed(FrameMetadata),
    RawPacket(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameMetadata {}
