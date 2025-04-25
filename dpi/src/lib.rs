// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

use crate::frame::{FrameMetadata, FrameType};
use crate::protocols::ProtocolId;
use crate::wrapper::OwnedFrame;

pub struct ProtocolParser {
    raw_needed: bool,
    roots: Vec<ProtocolId>,
}

impl ProtocolParser {
    pub fn new(raw_needed: bool) -> Self {
        Self {
            raw_needed,
            roots: ProtocolId::roots(),
        }
    }

    pub fn process(&self, packet: pcap::Packet) -> Option<FrameType> {
        let mut metadata = FrameMetadata::from_header(packet.header);

        for id in &self.roots {
            let result = id.parse()(&packet, &mut metadata);
            match result {
                ParseResult::Complete => {
                    return Some(FrameType::Metadata(metadata));
                },
                ParseResult::Incomplete => {
                    return if !self.raw_needed {
                        Some(FrameType::Metadata(metadata))
                    } else {
                        Some(FrameType::Raw(OwnedFrame::from(packet)))
                    };
                },
                ParseResult::Failed => continue,
            }
        }

        if !self.raw_needed {
            None
        } else {
            Some(FrameType::Raw(OwnedFrame::from(packet)))
        }
    }
}

pub type ParserFn = fn(&[u8], &mut FrameMetadata) -> ParseResult;
pub trait ParseableProtocol {
    fn id(&self) -> &ProtocolId;
    fn parse(bytes: &[u8], metadata: &mut FrameMetadata) -> ParseResult;
}

#[derive(Clone, Debug)]
pub enum ParseResult {
    Complete,
    Incomplete,
    Failed,
}

pub mod frame;
pub mod protocols;
pub mod wrapper;
