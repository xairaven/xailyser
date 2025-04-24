use crate::metadata::FrameMetadata;
use crate::protocols::ethernet::Ethernet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Protocols {
    Ethernet(Ethernet),

    ARP,
    ICMP,
    IPv4,
    IPv6,

    UDP,
    TCP,

    DNS,
    HTTP,
    HTTPS,
    FTP,
    SMTP,
    IMAP,
    POP3,
    SSH,
}

impl Protocols {
    pub fn parse(bytes: &[u8], metadata: &mut FrameMetadata) -> bool {
        ethernet::Ethernet::parse(bytes, metadata)
    }
}

pub trait Protocol {
    fn parse(bytes: &[u8], metadata: &mut FrameMetadata) -> bool;
}

mod ethernet;
