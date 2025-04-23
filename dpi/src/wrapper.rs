use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedPacket {
    pub header: PacketHeader,
    pub data: Vec<u8>,
}

impl<'a> From<pcap::Packet<'a>> for OwnedPacket {
    fn from(packet: pcap::Packet<'a>) -> Self {
        OwnedPacket {
            header: PacketHeader::from(packet.header),
            data: packet.data.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketHeader {
    pub tv_sec: i32,
    pub tv_usec: i32,
    pub caplen: u32,
    pub len: u32,
}

impl From<&pcap::PacketHeader> for PacketHeader {
    fn from(header: &pcap::PacketHeader) -> Self {
        Self {
            tv_sec: header.ts.tv_sec,
            tv_usec: header.ts.tv_usec,
            caplen: header.caplen,
            len: header.len,
        }
    }
}

impl From<&PacketHeader> for pcap::PacketHeader {
    fn from(header: &PacketHeader) -> Self {
        Self {
            ts: libc::timeval {
                tv_sec: header.tv_sec,
                tv_usec: header.tv_usec,
            },
            caplen: header.caplen,
            len: header.len,
        }
    }
}

pub fn save_pcap<P: AsRef<Path>>(
    path: P, packets: &[OwnedPacket], link_type: pcap::Linktype,
) -> Result<(), pcap::Error> {
    let mut file = pcap::Capture::dead(link_type)?.savefile(path)?;

    for packet in packets {
        let header = pcap::PacketHeader::from(&packet.header);
        file.write(&pcap::Packet::new(&header, &packet.data));
    }
    file.flush()?;

    Ok(())
}
