use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedPacket {
    pub header: PacketHeader,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketHeader {
    pub tv_sec: i32,
    pub tv_usec: i32,
    pub caplen: u32,
    pub len: u32,
}

impl<'a> From<pcap::Packet<'a>> for OwnedPacket {
    fn from(packet: pcap::Packet<'a>) -> Self {
        let header = packet.header;
        OwnedPacket {
            header: PacketHeader {
                tv_sec: header.ts.tv_sec,
                tv_usec: header.ts.tv_usec,
                caplen: header.caplen,
                len: header.len,
            },
            data: packet.data.to_vec(),
        }
    }
}

pub fn save_pcap<P: AsRef<Path>>(
    path: P, packets: &[OwnedPacket], link_type: pcap::Linktype,
) -> Result<(), pcap::Error> {
    let mut file = pcap::Capture::dead(link_type)?.savefile(path)?;

    for packet in packets {
        let header = pcap::PacketHeader {
            ts: libc::timeval {
                tv_sec: packet.header.tv_sec,
                tv_usec: packet.header.tv_usec,
            },
            caplen: packet.header.caplen,
            len: packet.header.len,
        };

        file.write(&pcap::Packet::new(&header, &packet.data));
    }
    file.flush()?;

    Ok(())
}
