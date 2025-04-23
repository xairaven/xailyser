// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

use crate::wrapper::{OwnedPacket, PacketHeader};

pub fn process(packet: pcap::Packet, unparsed_needed: bool) -> metadata::NetworkFrame {
    // TODO: Parsing

    // Sending just metadata instead of full bytes vector
    if unparsed_needed {
        let raw_packet = OwnedPacket::from(packet);
        metadata::NetworkFrame::RawPacket(raw_packet)
    } else {
        let raw_metadata = PacketHeader::from(packet.header);
        metadata::NetworkFrame::RawMetadata(raw_metadata)
    }
}

pub mod metadata;
pub mod wrapper;
