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
            let result = id.protocol().process(&packet, &mut metadata);
            match result {
                ProcessResult::Complete => {
                    return Some(FrameType::Metadata(metadata));
                },
                ProcessResult::Incomplete => {
                    return if !self.raw_needed {
                        Some(FrameType::Metadata(metadata))
                    } else {
                        Some(FrameType::Raw(OwnedFrame::from(packet)))
                    };
                },
                ProcessResult::Failed => continue,
            }
        }

        // If there are not roots... impossible
        if !self.raw_needed {
            None
        } else {
            Some(FrameType::Raw(OwnedFrame::from(packet)))
        }
    }
}

pub trait ParseableProtocol<'a> {
    fn id(&self) -> &ProtocolId;
    fn parse(&self, bytes: &'a [u8], metadata: &mut FrameMetadata) -> ParseResult<'a>;
    fn process(&self, bytes: &'a [u8], metadata: &mut FrameMetadata) -> ProcessResult {
        let result = self.parse(bytes, metadata);

        match result {
            ParseResult::Success => ProcessResult::Complete,
            ParseResult::SuccessIncomplete(bytes) => {
                let children = match self.id().children() {
                    Some(value) => value,
                    None => {
                        return ProcessResult::Incomplete;
                    },
                };

                for id in children {
                    let result = id.protocol().process(bytes, metadata);

                    match result {
                        ProcessResult::Complete | ProcessResult::Incomplete => {
                            return result;
                        },
                        ProcessResult::Failed => continue,
                    }
                }

                ProcessResult::Incomplete
            },
            ParseResult::Failed => ProcessResult::Failed,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProcessResult {
    // Fully parsed
    Complete,

    // Some protocols parsed (we are going into the deep), but some in the deepness are not
    Incomplete,

    // Not matched
    Failed,
}

pub enum ParseResult<'a> {
    // Fully parsed, to the last byte
    Success,

    // Exact protocol parsed successfully, but there are something inside
    SuccessIncomplete(&'a [u8]),

    // Not matched
    Failed,
}

pub mod frame;
pub mod protocols;
pub mod wrapper;
