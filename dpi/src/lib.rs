// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

use crate::wrapper::OwnedPacket;

pub fn process(packet: pcap::Packet) -> metadata::NetworkFrame {
    // TODO: Parsing

    let packet = OwnedPacket::from(packet);
    metadata::NetworkFrame::RawPacket(packet)
}

pub mod metadata;
pub mod wrapper;
