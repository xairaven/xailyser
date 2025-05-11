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
        self.arp.len()
            + self.dhcpv4.len()
            + self.dhcpv6.len()
            + self.dns.len()
            + self.ethernet.len()
            + self.http.len()
            + self.icmpv4.len()
            + self.icmpv6.len()
            + self.tcp.len()
            + self.udp.len()
    }

    pub fn clear(&mut self) {
        self.arp.clear();
        self.dhcpv4.clear();
        self.dhcpv6.clear();
        self.dns.clear();
        self.ethernet.clear();
        self.http.clear();
        self.icmpv4.clear();
        self.icmpv6.clear();
        self.tcp.clear();
        self.udp.clear();
    }
}
