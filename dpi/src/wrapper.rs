use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedFrame {
    pub header: FrameHeader,
    pub data: Vec<u8>,
}

impl<'a> From<pcap::Packet<'a>> for OwnedFrame {
    fn from(packet: pcap::Packet<'a>) -> Self {
        OwnedFrame {
            header: FrameHeader::from(packet.header),
            data: packet.data.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameHeader {
    pub tv_sec: i32,
    pub tv_usec: i32,
    pub caplen: u32,
    pub len: u32,
}

impl From<&pcap::PacketHeader> for FrameHeader {
    fn from(header: &pcap::PacketHeader) -> Self {
        Self {
            tv_sec: header.ts.tv_sec,
            tv_usec: header.ts.tv_usec,
            caplen: header.caplen,
            len: header.len,
        }
    }
}

impl From<&FrameHeader> for pcap::PacketHeader {
    fn from(header: &FrameHeader) -> Self {
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
    path: P, frames: &VecDeque<OwnedFrame>, link_type: pcap::Linktype,
) -> Result<(), pcap::Error> {
    let mut file = pcap::Capture::dead(link_type)?.savefile(path)?;

    for frame in frames {
        let header = pcap::PacketHeader::from(&frame.header);
        file.write(&pcap::Packet::new(&header, &frame.data));
    }
    file.flush()?;

    Ok(())
}
