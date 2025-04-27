// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

use crate::frame::{FrameMetadata, FrameType};
use crate::protocols::{ProtocolData, ProtocolId};
use crate::wrapper::OwnedFrame;
use nom::IResult;

pub struct ProtocolParser {
    raw_needed: bool,
    root: Option<ProtocolId>,
}

impl ProtocolParser {
    pub fn new(link_type: &pcap::Linktype, raw_needed: bool) -> Self {
        Self {
            raw_needed,
            root: Self::get_root_protocol(link_type),
        }
    }

    pub fn process(&self, packet: pcap::Packet) -> Option<FrameType> {
        let mut metadata = FrameMetadata::from_header(packet.header);

        if let Some(root_protocol) = &self.root {
            let result = traversal(root_protocol, &packet, &mut metadata);
            return match result {
                ProcessResult::Complete => Some(FrameType::Metadata(metadata)),
                ProcessResult::Incomplete => {
                    if !self.raw_needed {
                        Some(FrameType::Metadata(metadata))
                    } else {
                        Some(FrameType::Raw(OwnedFrame::from(packet)))
                    }
                },
                ProcessResult::Failed => {
                    if !self.raw_needed {
                        None
                    } else {
                        Some(FrameType::Raw(OwnedFrame::from(packet)))
                    }
                },
            };
        }

        None
    }

    fn get_root_protocol(link_type: &pcap::Linktype) -> Option<ProtocolId> {
        match link_type {
            pcap::Linktype(1) => Some(ProtocolId::Ethernet),
            _ => None,
        }
    }
}

fn traversal(
    id: &ProtocolId, bytes: &[u8], metadata: &mut FrameMetadata,
) -> ProcessResult {
    let result = id.parse()(bytes, metadata);

    match result {
        Ok(([], layer)) => {
            metadata.layers.push(layer);
            ProcessResult::Complete
        },
        Ok((rest, layer)) => {
            metadata.layers.push(layer);

            if let Some(best) = id.best_children(metadata) {
                return traversal(&best, rest, metadata);
            }

            let children = match id.children() {
                Some(value) => value,
                None => {
                    return ProcessResult::Incomplete;
                },
            };

            for id in children {
                let result = traversal(&id, rest, metadata);

                match result {
                    ProcessResult::Complete | ProcessResult::Incomplete => {
                        return result;
                    },
                    ProcessResult::Failed => continue,
                }
            }

            ProcessResult::Incomplete
        },
        Err(_) => ProcessResult::Failed,
    }
}
pub type ParseFn =
    for<'a, 'b> fn(&'a [u8], &'b FrameMetadata) -> IResult<&'a [u8], ProtocolData>;

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
    Success(ProtocolData),

    // Exact protocol parsed successfully, but there are something inside
    SuccessIncomplete(ProtocolData, &'a [u8]),

    // Not matched
    Failed,
}

pub mod error;
pub mod frame;
pub mod protocols;
pub mod wrapper;
