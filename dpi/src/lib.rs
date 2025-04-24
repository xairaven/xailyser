// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

use crate::metadata::{FrameMetadata, NetworkFrame};
use crate::protocols::Protocols;
use crate::wrapper::OwnedPacket;

pub fn process(packet: pcap::Packet, unparsed_needed: bool) -> NetworkFrame {
    let mut metadata = FrameMetadata::from_header(packet.header);

    let is_fully_parsed = Protocols::parse(packet.data, &mut metadata);

    // If fully parsed or not needed raw bytes for saving to pcap - first branch
    // If not parsed fully and unparsed needed for saving to pcap - second branch
    if is_fully_parsed || !unparsed_needed {
        NetworkFrame::Metadata(metadata)
    } else {
        let raw_packet = OwnedPacket::from(packet);
        NetworkFrame::RawPacket(raw_packet)
    }
}

pub mod metadata;
pub mod protocols;
pub mod wrapper;
