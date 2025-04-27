use crate::frame::FrameMetadata;
use crate::protocols::{ProtocolData, ProtocolId};
use nom::IResult;

// IPv4 Protocol
// RFC 791: https://datatracker.ietf.org/doc/html/rfc791

pub fn parse<'a>(bytes: &'a [u8], _: &FrameMetadata) -> IResult<&'a [u8], ProtocolData> {
    todo!()
}

pub fn best_children(metadata: &FrameMetadata) -> Option<ProtocolId> {
    todo!()
}

pub mod address;
