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
            ProtocolData::Ethernet(value) => ProtocolDto::Ethernet(value.into()),
            ProtocolData::Arp(value) => ProtocolDto::Arp(value.into()),
            ProtocolData::DHCPv4(value) => ProtocolDto::DHCPv4(value.into()),
            ProtocolData::DHCPv6(value) => ProtocolDto::DHCPv6(value.into()),
            ProtocolData::DNS(value) => ProtocolDto::DNS(value.into()),
            ProtocolData::HTTP(value) => ProtocolDto::HTTP(value.into()),
            ProtocolData::IPv4(value) => ProtocolDto::IPv4(value.into()),
            ProtocolData::IPv6(value) => ProtocolDto::IPv6(value.into()),
            ProtocolData::ICMPv4(value) => ProtocolDto::ICMPv4(value.into()),
            ProtocolData::ICMPv6(value) => ProtocolDto::ICMPv6(value.into()),
            ProtocolData::TCP(value) => ProtocolDto::TCP(value.into()),
            ProtocolData::UDP(value) => ProtocolDto::UDP(value.into()),
        }
    }
}
