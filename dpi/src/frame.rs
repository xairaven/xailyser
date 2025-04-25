use crate::protocols::ProtocolData;
use crate::wrapper::{FrameHeader, OwnedFrame};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FrameType {
    Metadata(FrameMetadata),
    Raw(OwnedFrame),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub header: FrameHeader,
    pub layers: Vec<ProtocolData>,
}

impl FrameMetadata {
    pub fn from_header(header: &pcap::PacketHeader) -> Self {
        Self {
            header: FrameHeader::from(header),
            layers: vec![],
        }
    }
}
