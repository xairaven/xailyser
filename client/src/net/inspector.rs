use crate::ws::data::{Locator, PortDto};
use dpi::protocols::arp::ArpDto;
use dpi::protocols::dhcpv4::DHCPv4Dto;
use dpi::protocols::dhcpv6::DHCPv6Dto;
use dpi::protocols::dns::DnsDto;
use dpi::protocols::http::HttpDto;
use dpi::protocols::icmpv4::ICMPv4Dto;
use dpi::protocols::icmpv6::ICMPv6Dto;
use dpi::protocols::ipv4::IPv4Dto;
use dpi::protocols::ipv6::IPv6Dto;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Default)]
pub struct InspectorStorage {
    pub arp: Vec<ArpDto>,
    pub dhcpv4: Vec<DHCPv4Dto>,
    pub dhcpv6: Vec<DHCPv6Dto>,
    pub dns: Vec<DnsDto>,
    pub ethernet: Vec<Locator>,
    pub http: Vec<(HttpDto, Locator)>,
    pub icmpv4: Vec<(ICMPv4Dto, Locator)>,
    pub icmpv6: Vec<(ICMPv6Dto, Locator)>,
    pub ipv4: Vec<(IPv4Dto, Locator)>,
    pub ipv6: Vec<(IPv6Dto, Locator)>,
    pub tcp: Vec<(PortDto, Locator)>,
    pub udp: Vec<(PortDto, Locator)>,
}

impl InspectorStorage {
    pub fn len(&self) -> usize {
        let mut sum: usize = 0;
        for protocol in ProtocolsRegistered::iter() {
            sum += self.records_captured(&protocol);
        }
        sum
    }

    pub fn clear(&mut self) {
        for protocol in ProtocolsRegistered::iter() {
            self.clear_by_protocol(&protocol);
        }
    }

    pub fn records_captured(&self, protocol: &ProtocolsRegistered) -> usize {
        match protocol {
            ProtocolsRegistered::Arp => self.arp.len(),
            ProtocolsRegistered::DHCPv4 => self.dhcpv4.len(),
            ProtocolsRegistered::DHCPv6 => self.dhcpv6.len(),
            ProtocolsRegistered::Dns => self.dns.len(),
            ProtocolsRegistered::Ethernet => self.ethernet.len(),
            ProtocolsRegistered::Http => self.http.len(),
            ProtocolsRegistered::ICMPv4 => self.icmpv4.len(),
            ProtocolsRegistered::ICMPv6 => self.icmpv6.len(),
            ProtocolsRegistered::IPv4 => self.ipv4.len(),
            ProtocolsRegistered::IPv6 => self.ipv6.len(),
            ProtocolsRegistered::Tcp => self.tcp.len(),
            ProtocolsRegistered::Udp => self.udp.len(),
        }
    }

    fn clear_by_protocol(&mut self, protocol: &ProtocolsRegistered) {
        match protocol {
            ProtocolsRegistered::Arp => self.arp.clear(),
            ProtocolsRegistered::DHCPv4 => self.dhcpv4.clear(),
            ProtocolsRegistered::DHCPv6 => self.dhcpv6.clear(),
            ProtocolsRegistered::Dns => self.dns.clear(),
            ProtocolsRegistered::Ethernet => self.ethernet.clear(),
            ProtocolsRegistered::Http => self.http.clear(),
            ProtocolsRegistered::ICMPv4 => self.icmpv4.clear(),
            ProtocolsRegistered::ICMPv6 => self.icmpv6.clear(),
            ProtocolsRegistered::IPv4 => self.ipv4.clear(),
            ProtocolsRegistered::IPv6 => self.ipv6.clear(),
            ProtocolsRegistered::Tcp => self.tcp.clear(),
            ProtocolsRegistered::Udp => self.udp.clear(),
        }
    }
}

#[derive(Debug, Clone, Display, EnumIter)]
pub enum ProtocolsRegistered {
    #[strum(to_string = "ARP")]
    Arp,

    DHCPv4,
    DHCPv6,

    #[strum(to_string = "DNS")]
    Dns,

    Ethernet,

    #[strum(to_string = "HTTP")]
    Http,

    ICMPv4,
    ICMPv6,
    IPv4,
    IPv6,

    #[strum(to_string = "TCP")]
    Tcp,
    #[strum(to_string = "UDP")]
    Udp,
}
