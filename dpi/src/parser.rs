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
            root: ProtocolId::root(link_type),
        }
    }

    pub fn process(&self, packet: pcap::Packet) -> Option<FrameType> {
        let mut metadata = FrameMetadata::from_header(packet.header);

        if let Some(root_protocol) = &self.root {
            let result = Self::traversal(root_protocol, &packet, &mut metadata);
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
                    return Self::traversal(&best, rest, metadata);
                }

                let children = match id.children() {
                    Some(value) => value,
                    None => {
                        return ProcessResult::Incomplete;
                    },
                };

                for id in children {
                    let result = Self::traversal(&id, rest, metadata);

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
}
pub type ParseFn =
    for<'a, 'b> fn(&'a [u8], &'b FrameMetadata) -> IResult<&'a [u8], ProtocolData>;

pub enum ParserError {
    ErrorVerify,
    FailureVerify,
}

impl ParserError {
    pub fn to_nom<T>(&self, input: T) -> nom::Err<nom::error::Error<T>> {
        match self {
            Self::ErrorVerify => nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )),
            Self::FailureVerify => nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )),
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
