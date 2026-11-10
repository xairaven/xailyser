use crate::dto::frame::{FrameHeader, FrameMetadata};
use crate::protocols::{
    ProtocolData, arp, dhcpv4, dhcpv6, dns, ethernet, http, icmpv4, icmpv6, ipv4, ipv6,
    tcp, udp,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameMetadataDto {
    pub header: FrameHeader,
    pub layers: Vec<ProtocolDto>,
}

impl From<FrameMetadata> for FrameMetadataDto {
    fn from(value: FrameMetadata) -> Self {
        Self {
            header: value.header,
            layers: value.layers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProtocolDto {
    Ethernet(ethernet::EthernetDto),

    Arp(arp::ArpDto),

    DHCPv4(dhcpv4::DHCPv4Dto),
    DHCPv6(dhcpv6::DHCPv6Dto),
    DNS(dns::DnsDto),
    HTTP(http::HttpDto),

    IPv4(ipv4::IPv4Dto),
    IPv6(ipv6::IPv6Dto),

    ICMPv4(icmpv4::ICMPv4Dto),
    ICMPv6(icmpv6::ICMPv6Dto),

    TCP(tcp::TcpDto),
    UDP(udp::UdpDto),
}

impl From<ProtocolData> for ProtocolDto {
    fn from(value: ProtocolData) -> Self {
        match value {
            ProtocolData::Ethernet(value) => Self::Ethernet(value.into()),
            ProtocolData::Arp(value) => Self::Arp(value.into()),
            ProtocolData::DHCPv4(value) => Self::DHCPv4(value.into()),
            ProtocolData::DHCPv6(value) => Self::DHCPv6(value.into()),
            ProtocolData::DNS(value) => Self::DNS(value.into()),
            ProtocolData::HTTP(value) => Self::HTTP(value.into()),
            ProtocolData::IPv4(value) => Self::IPv4(value.into()),
            ProtocolData::IPv6(value) => Self::IPv6(value.into()),
            ProtocolData::ICMPv4(value) => Self::ICMPv4(value.into()),
            ProtocolData::ICMPv6(value) => Self::ICMPv6(value.into()),
            ProtocolData::TCP(value) => Self::TCP(value.into()),
            ProtocolData::UDP(value) => Self::UDP(value.into()),
        }
    }
}
