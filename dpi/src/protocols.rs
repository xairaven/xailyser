use crate::ParseFn;
use crate::frame::FrameMetadata;
use crate::protocols::arp::Arp;
use crate::protocols::ethernet::Ethernet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProtocolId {
    Ethernet,

    Arp,

    IPv4,
    // IPv6,
    //
    // ICMP,
    // ICMPv6,
    //
    // TCP,
    // UDP,
    //
    // DNS,
    // FTP,
    // HTTP,
    // HTTPS,
    // IMAP,
    // POP3,
    // SMTP,
    // SSH,
}

impl ProtocolId {
    pub fn roots() -> Vec<Self> {
        vec![Self::Ethernet]
    }

    pub fn parse(&self) -> ParseFn {
        match self {
            ProtocolId::Ethernet => ethernet::parse,
            ProtocolId::Arp => arp::parse,
            ProtocolId::IPv4 => ipv4::parse,
        }
    }

    pub fn best_children(&self, metadata: &FrameMetadata) -> Option<Self> {
        match self {
            ProtocolId::Ethernet => ethernet::best_children(metadata),
            ProtocolId::Arp => None,
            ProtocolId::IPv4 => ipv4::best_children(metadata),
        }
    }

    pub fn children(&self) -> Option<Vec<Self>> {
        match self {
            ProtocolId::Ethernet => Some(vec![
                Self::Arp,
                Self::IPv4,
                // Self::ICMP,
                // Self::ICMPv6,
                // Self::IPv6,
            ]),
            ProtocolId::Arp => None,
            ProtocolId::IPv4 => Some(vec![todo!()]),
            // ProtocolId::IPv4 => Some(vec![Self::ICMP, Self::TCP, Self::UDP]),
            // ProtocolId::IPv6 => Some(vec![Self::ICMPv6, Self::TCP, Self::UDP]),
            // ProtocolId::ICMP => None,
            // ProtocolId::ICMPv6 => None,
            //
            // ProtocolId::TCP => Some(vec![Self::HTTP, Self::HTTPS, Self::DNS]),
            // ProtocolId::UDP => Some(vec![Self::DNS]),
            //
            // ProtocolId::DNS => None,
            // ProtocolId::FTP => None,
            // ProtocolId::HTTP => None,
            // ProtocolId::HTTPS => None,
            // ProtocolId::IMAP => None,
            // ProtocolId::POP3 => None,
            // ProtocolId::SMTP => None,
            // ProtocolId::SSH => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProtocolData {
    Ethernet(Ethernet),

    Arp(Arp),
}

pub mod arp;
pub mod ethernet;
pub mod ipv4;
