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
            let result = Self::traversal(root_protocol, &packet, &mut metadata, 0);
            return match result {
                ProcessResult::Complete => Some(FrameType::Metadata(metadata)),
                ProcessResult::Incomplete => match self.raw_needed {
                    true => Some(FrameType::Raw(OwnedFrame::from(packet))),
                    false => Some(FrameType::Metadata(metadata)),
                },
                ProcessResult::Failed => match self.raw_needed {
                    true => Some(FrameType::Raw(OwnedFrame::from(packet))),
                    false => Some(FrameType::Header(metadata.header)),
                },
            };
        }

        None
    }

    fn traversal(
        id: &ProtocolId, bytes: &[u8], metadata: &mut FrameMetadata, depth: usize
    ) -> ProcessResult {
        const MAX_DEPTH: usize = 16;
        if depth > MAX_DEPTH {
            return ProcessResult::Failed;
        }
        
        let result = id.parse()(bytes);

        match result {
            Ok(([], layer)) => {
                metadata.layers.push(layer);
                ProcessResult::Complete
            },
            Ok((rest, layer)) => {
                metadata.layers.push(layer);

                if let Some(best) = id.best_children(metadata) {
                    return Self::traversal(&best, rest, metadata, depth + 1);
                }

                let children = match id.children() {
                    Some(value) => value,
                    None => {
                        return ProcessResult::Incomplete;
                    },
                };

                for id in children {
                    let result = Self::traversal(&id, rest, metadata, depth + 1);

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

    pub fn cast_to_bool(bit: u8) -> Result<bool, ParserError> {
        match bit {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ParserError::ErrorVerify),
        }
    }
}
pub type ParseFn = fn(&[u8]) -> IResult<&[u8], ProtocolData>;
pub type PortFn = fn(u16, u16) -> bool;

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
