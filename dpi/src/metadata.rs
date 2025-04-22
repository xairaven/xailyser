use crate::wrapper::OwnedPacket;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkFrame {
    Parsed(FrameMetadata),
    RawPacket(OwnedPacket),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameMetadata {}
