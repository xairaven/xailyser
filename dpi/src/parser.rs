use crate::dto::frame::{FrameMetadata, FrameType, OwnedFrame};
use crate::protocols::{ProtocolData, ProtocolId};
use nom::IResult;
use nom::Parser;
use nom::bytes::take;
use nom::number::be_u8;

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
            let result = traversal(root_protocol, &packet, &mut metadata, 0);
            return match result {
                ProcessResult::Complete => Some(FrameType::Metadata(metadata.into())),
                ProcessResult::Incomplete => match self.raw_needed {
                    true => Some(FrameType::Raw(OwnedFrame::from(packet))),
                    false => Some(FrameType::Metadata(metadata.into())),
                },
                ProcessResult::Failed => match self.raw_needed {
                    true => Some(FrameType::Raw(OwnedFrame::from(packet))),
                    false => Some(FrameType::Header(metadata.header)),
                },
            };
        }

        None
    }
}

fn traversal(
    id: &ProtocolId, bytes: &[u8], metadata: &mut FrameMetadata, depth: usize,
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
                return match depth.checked_add(1) {
                    Some(new_depth) => traversal(&best, rest, metadata, new_depth),
                    None => ProcessResult::Failed,
                };
            }

            let children = match id.children() {
                Some(value) => value,
                None => {
                    return ProcessResult::Incomplete;
                },
            };

            for id in children {
                let result = match depth.checked_add(1) {
                    Some(new_depth) => traversal(&id, rest, metadata, new_depth),
                    None => return ProcessResult::Failed,
                };

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

pub fn wire_format(input: &[u8]) -> IResult<&[u8], String> {
    let mut labels = Vec::new();
    let mut rest_buffer = input;
    while !rest_buffer.is_empty() {
        let (rest, len_byte) = be_u8().parse(rest_buffer)?;
        // Null-terminator
        if len_byte == 0 {
            debug_assert!(rest.is_empty());
            rest_buffer = rest;
            break;
        }

        // Unexpected length
        if len_byte as usize > rest.len() {
            return Err(ParserError::ErrorVerify.to_nom(input));
        }

        // Creating label
        let (rest, label): (&[u8], &[u8]) = take(len_byte).parse(rest)?;
        let label = String::from_utf8(label.to_vec())
            .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
        labels.push(label);

        rest_buffer = rest;
    }

    Ok((rest_buffer, labels.join(".")))
}

pub fn cast_to_bool(bit: u8) -> Result<bool, ParserError> {
    match bit {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ParserError::ErrorVerify),
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

#[cfg(test)]
pub(crate) mod tests {
    use crate::dto::frame::FrameMetadata;
    use crate::parser::ProcessResult;
    use crate::protocols::ProtocolId;

    pub enum FrameType {
        Metadata(FrameMetadata),
        Header(()),
        Raw(()),
    }

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
                let result = super::traversal(root_protocol, &packet, &mut metadata, 0);
                return match result {
                    ProcessResult::Complete => Some(FrameType::Metadata(metadata)),
                    ProcessResult::Incomplete => match self.raw_needed {
                        true => Some(FrameType::Raw(())),
                        false => Some(FrameType::Metadata(metadata)),
                    },
                    ProcessResult::Failed => match self.raw_needed {
                        true => Some(FrameType::Raw(())),
                        false => Some(FrameType::Header(())),
                    },
                };
            }

            None
        }
    }
}
