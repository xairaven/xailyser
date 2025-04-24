use crate::protocols::Protocols;
use crate::wrapper::{OwnedPacket, PacketHeader};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkFrame {
    Metadata(FrameMetadata),
    RawPacket(OwnedPacket),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub header: PacketHeader,
    pub layers: Vec<Protocols>,
}

impl FrameMetadata {
    pub fn from_header(header: &pcap::PacketHeader) -> Self {
        Self {
            header: PacketHeader::from(header),
            layers: vec![],
        }
    }
}
