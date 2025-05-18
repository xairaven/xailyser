use crate::dto::metadata::FrameMetadataDto;
use crate::protocols::ProtocolData;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FrameType {
    Metadata(FrameMetadataDto),
    Header(FrameHeader),
    Raw(OwnedFrame),
}

#[derive(Clone, Debug)]
pub struct FrameMetadata {
    pub header: FrameHeader,
    pub layers: Vec<ProtocolData>,
}

impl FrameMetadata {
    pub fn from_header(header: &pcap::PacketHeader) -> Self {
        Self {
            header: FrameHeader::from(header),
            layers: vec![],
        }
    }
}

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
    pub tv_sec: i64,
    pub tv_usec: i64,
    pub caplen: u32,
    pub len: u32,
}

impl From<&pcap::PacketHeader> for FrameHeader {
    fn from(header: &pcap::PacketHeader) -> Self {
        #[cfg(target_family = "unix")]
        let result = Self {
            tv_sec: header.ts.tv_sec,
            tv_usec: header.ts.tv_usec,
            caplen: header.caplen,
            len: header.len,
        };

        #[cfg(target_os = "windows")]
        let result = Self {
            tv_sec: header.ts.tv_sec as i64,
            tv_usec: header.ts.tv_usec as i64,
            caplen: header.caplen,
            len: header.len,
        };

        result
    }
}

impl From<&FrameHeader> for pcap::PacketHeader {
    fn from(header: &FrameHeader) -> Self {
        #[cfg(target_family = "unix")]
        let result = Self {
            ts: libc::timeval {
                tv_sec: header.tv_sec,
                tv_usec: header.tv_usec,
            },
            caplen: header.caplen,
            len: header.len,
        };

        #[cfg(target_os = "windows")]
        let result = Self {
            ts: libc::timeval {
                tv_sec: header.tv_sec as i32,
                tv_usec: header.tv_usec as i32,
            },
            caplen: header.caplen,
            len: header.len,
        };

        result
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
