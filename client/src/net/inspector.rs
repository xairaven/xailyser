use dpi::protocols::arp::ArpDto;
use dpi::protocols::dhcpv4::DHCPv4Dto;
use dpi::protocols::dhcpv6::DHCPv6Dto;
use dpi::protocols::dns::DnsDto;
use dpi::protocols::ethernet::EthernetDto;
use dpi::protocols::http::HttpDto;
use dpi::protocols::icmpv4::ICMPv4Dto;
use dpi::protocols::icmpv6::ICMPv6Dto;
use dpi::protocols::ipv4::IPv4Dto;
use dpi::protocols::ipv6::IPv6Dto;
use dpi::protocols::tcp::TcpDto;
use dpi::protocols::udp::UdpDto;

#[derive(Default)]
pub struct InspectorStorage {
    pub arp: Vec<ArpDto>,
    pub dhcpv4: Vec<DHCPv4Dto>,
    pub dhcpv6: Vec<DHCPv6Dto>,
    pub dns: Vec<DnsDto>,
    pub ethernet: Vec<EthernetDto>,
    pub http: Vec<HttpDto>,
    pub icmpv4: Vec<ICMPv4Dto>,
    pub icmpv6: Vec<ICMPv6Dto>,
    pub ipv4: Vec<IPv4Dto>,
    pub ipv6: Vec<IPv6Dto>,
    pub tcp: Vec<TcpDto>,
    pub udp: Vec<UdpDto>,
}
