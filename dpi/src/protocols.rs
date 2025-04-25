use crate::ParseFn;
use crate::protocols::arp::Arp;
use crate::protocols::ethernet::Ethernet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProtocolId {
    Ethernet,

    ARP,

    IPv4,
    IPv6,

    ICMP,
    ICMPv6,

    TCP,
    UDP,

    DNS,
    FTP,
    HTTP,
    HTTPS,
    IMAP,
    POP3,
    SMTP,
    SSH,
}

impl ProtocolId {
    pub fn roots() -> Vec<Self> {
        vec![Self::Ethernet]
    }

    pub fn parse(&self) -> ParseFn {
        match self {
            ProtocolId::Ethernet => ethernet::parse,
            ProtocolId::ARP => arp::parse,
            _ => todo!(),
        }
    }

    pub fn children(&self) -> Option<Vec<Self>> {
        match self {
            ProtocolId::Ethernet => Some(vec![
                Self::ARP,
                Self::ICMP,
                Self::ICMPv6,
                Self::IPv4,
                Self::IPv6,
            ]),
            ProtocolId::ARP => None,

            ProtocolId::IPv4 => Some(vec![Self::ICMP, Self::TCP, Self::UDP]),
            ProtocolId::IPv6 => Some(vec![Self::ICMPv6, Self::TCP, Self::UDP]),
            ProtocolId::ICMP => None,
            ProtocolId::ICMPv6 => None,

            ProtocolId::TCP => Some(vec![Self::HTTP, Self::HTTPS, Self::DNS]),
            ProtocolId::UDP => Some(vec![Self::DNS]),

            ProtocolId::DNS => None,
            ProtocolId::FTP => None,
            ProtocolId::HTTP => None,
            ProtocolId::HTTPS => None,
            ProtocolId::IMAP => None,
            ProtocolId::POP3 => None,
            ProtocolId::SMTP => None,
            ProtocolId::SSH => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProtocolData {
    Ethernet(Ethernet),

    ARP(Arp),
}

mod arp;
mod ethernet;
