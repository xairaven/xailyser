use crate::dto::frame::FrameMetadata;
use crate::parser::{ParseFn, PortFn};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

/// Guide: How to Add a Protocol
/// 1. Add it to the `ProtocolId` enum.
/// 2. If the protocol is a root protocol, add a link to it in the `ProtocolId::root` method according to the linktype.
/// 3. Add a parsing method with the signature `ParseFn` to the `ProtocolId::parse` method. The parsing method itself should be placed in your module, e.g., `protocols::custom_protocol`.
/// 4. If there is a way to determine the most suitable nested protocol, create a `best_children` method in your module, following the pattern of existing methods. Link your new method in `ProtocolId::best_children`.
/// 5. In `ProtocolId::children`, specify whether there are any nested protocols.
///
/// That's it! After that, write tests and verify that parsing works correctly.

// FEATURE: FTP, TLS/SSL, IMAP, POP3, SMTP, SSH, SIP, RTP
#[derive(Clone, Debug, EnumIter, Display, Serialize, Deserialize, PartialEq, Copy)]
pub enum ProtocolId {
    Arp,
    DHCPv4,
    DHCPv6,
    DNS,
    Ethernet,
    HTTP,
    ICMPv4,
    ICMPv6,
    IPv4,
    IPv6,
    TCP,
    UDP,
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
            Self::Ethernet => ethernet::parse,
            Self::Arp => arp::parse,
            Self::DHCPv4 => dhcpv4::parse,
            Self::DHCPv6 => dhcpv6::parse,
            Self::DNS => dns::parse,
            Self::HTTP => http::parse,
            Self::ICMPv4 => icmpv4::parse,
            Self::ICMPv6 => icmpv6::parse,
            Self::IPv4 => ipv4::parse,
            Self::IPv6 => ipv6::parse,
            Self::TCP => tcp::parse,
            Self::UDP => udp::parse,
        }
    }

    pub fn check_ports(&self) -> Option<PortFn> {
        match self {
            Self::Ethernet => None,
            Self::Arp => None,
            Self::IPv4 => None,
            Self::IPv6 => None,
            Self::ICMPv4 => None,
            Self::ICMPv6 => None,
            Self::TCP => None,
            Self::UDP => None,
            Self::DHCPv4 => Some(dhcpv4::is_protocol_default),
            Self::DHCPv6 => Some(dhcpv6::is_protocol_default),
            Self::DNS => Some(dns::is_protocol_default),
            Self::HTTP => Some(http::is_protocol_default),
        }
    }

    pub fn best_children(&self, metadata: &FrameMetadata) -> Option<Self> {
        match self {
            Self::Ethernet => ethernet::best_children(metadata),
            Self::Arp => None,
            Self::DHCPv4 => None,
            Self::DHCPv6 => None,
            Self::DNS => None,
            Self::HTTP => None,
            Self::ICMPv4 => None,
            Self::ICMPv6 => None,
            Self::IPv4 => ipv4::best_children(metadata),
            Self::IPv6 => ipv6::best_children(metadata),
            Self::TCP => tcp::best_children(metadata),
            Self::UDP => udp::best_children(metadata),
        }
    }

    pub fn children(&self) -> Option<Vec<Self>> {
        match self {
            Self::Ethernet => Some(vec![Self::Arp, Self::IPv4, Self::IPv6]),
            Self::Arp => None,

            Self::IPv4 => Some(vec![Self::ICMPv4, Self::TCP, Self::UDP]),
            Self::IPv6 => Some(vec![Self::TCP, Self::UDP, Self::ICMPv6, Self::IPv6]),
            Self::ICMPv4 => None,
            Self::ICMPv6 => None,

            // TODO: TCP, UDP: Add HTTPS
            Self::TCP => Some(vec![Self::HTTP, Self::DNS, Self::DHCPv4, Self::DHCPv6]),
            Self::UDP => Some(vec![Self::HTTP, Self::DNS, Self::DHCPv4, Self::DHCPv6]),

            Self::DHCPv4 => None,
            Self::DHCPv6 => None,
            Self::DNS => None,
            Self::HTTP => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProtocolData {
    Ethernet(ethernet::Ethernet),

    Arp(arp::Arp),

    DHCPv4(dhcpv4::DHCPv4),
    DHCPv6(dhcpv6::DHCPv6),
    DNS(dns::DNS),
    HTTP(http::HTTP),

    IPv4(ipv4::IPv4),
    IPv6(ipv6::IPv6),

    ICMPv4(icmpv4::ICMPv4),
    ICMPv6(icmpv6::ICMPv6),

    TCP(tcp::TCP),
    UDP(udp::UDP),
}

pub mod arp;
pub mod dhcpv4;
pub mod dhcpv6;
pub mod dns;
pub mod ethernet;
pub mod http;
pub mod icmpv4;
pub mod icmpv6;
pub mod ip {
    pub mod address;
    pub mod protocol;
}
pub mod ipv4;
pub mod ipv6;
pub mod tcp;
pub mod udp;
