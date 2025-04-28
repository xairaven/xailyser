use crate::ParseFn;
use crate::frame::FrameMetadata;
use crate::protocols::arp::Arp;
use crate::protocols::ethernet::Ethernet;
use crate::protocols::ipv4::IPv4;
use serde::{Deserialize, Serialize};

/// Guide: How to Add a Protocol
/// 1. Add it to the `ProtocolId` enum.
/// 2. If the protocol is a root protocol, add a link to it in the `ProtocolId::root` method according to the linktype.
/// 3. Add a parsing method with the signature `ParseFn` to the `ProtocolId::parse` method. The parsing method itself should be placed in your module, e.g., `protocols::custom_protocol`.
/// 4. If there is a way to determine the most suitable nested protocol, create a `best_children` method in your module, following the pattern of existing methods. Link your new method in `ProtocolId::best_children`.
/// 5. In `ProtocolId::children`, specify whether there are any nested protocols.
///
/// That's it! After that, write tests and verify that parsing works correctly.
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
    pub fn root(link_type: &pcap::Linktype) -> Option<Self> {
        match link_type {
            pcap::Linktype(1) => Some(Self::Ethernet),
            _ => None,
        }
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

            // TODO: ...
            ProtocolId::IPv4 => None,
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

    IPv4(IPv4),
}

pub mod arp;
pub mod ethernet;
pub mod ipv4;
