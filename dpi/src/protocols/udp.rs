use crate::frame::FrameMetadata;
use crate::protocols::{ProtocolData, ProtocolId, dns};
use nom::IResult;
use nom::Parser;
use nom::number::be_u16;
use serde::{Deserialize, Serialize};

// UDP Protocol
// RFC 768: https://datatracker.ietf.org/doc/html/rfc768

pub fn parse<'a>(bytes: &'a [u8], _: &FrameMetadata) -> IResult<&'a [u8], ProtocolData> {
    // Source port. 2 bytes
    let (rest, port_source) = be_u16().parse(bytes)?;
    // Destination port. 2 bytes
    let (rest, port_destination) = be_u16().parse(rest)?;
    // Length. 2 bytes
    let (rest, length) = be_u16().parse(rest)?;
    // Checksum. 2 bytes
    let (rest, checksum) = be_u16().parse(rest)?;

    let payload = rest;
    let protocol = UDP {
        port_source,
        port_destination,
        length,
        checksum,
    };

    Ok((payload, ProtocolData::UDP(protocol)))
}

pub fn best_children(metadata: &FrameMetadata) -> Option<ProtocolId> {
    // Checking ports
    let layer = match metadata.layers.last() {
        Some(ProtocolData::UDP(value)) => value,
        _ => return None,
    };
    let port_source = layer.port_source;
    let port_destination = layer.port_destination;

    if port_source == dns::PORT_DNS || port_destination == dns::PORT_DNS {
        Some(ProtocolId::DNS)
    } else {
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UDP {
    pub port_source: u16,
    pub port_destination: u16,
    pub length: u16,
    pub checksum: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::FrameType;
    use crate::parser::ProtocolParser;
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv4::IPv4;
    use crate::wrapper::FrameHeader;
    use std::net::Ipv4Addr;

    #[test]
    fn test_udp_without_ethernet_padding() {
        let hex_actual = "01 00 5E 00 00 FC 40 61 86 9A F1 F5 08 00 45 00 00 32 6A 3D 00 00 01 11 AA 56 C0 A8 03 83 E0 00 00 FC D5 48 14 EB 00 1E 20 88 76 F2 00 00 00 01 00 00 00 00 00 00 04 77 70 61 64 00 00 01 00 01".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 64,
            len: 0,
        };

        let parser = ProtocolParser::new(&pcap::Linktype(1), false);
        let packet = pcap::Packet {
            header: &pcap::PacketHeader::from(&header),
            data: &frame,
        };
        let result = parser.process(packet);
        let metadata = match result {
            Some(value) => match value {
                FrameType::Metadata(value) => value,
                FrameType::Raw(_) => panic!(),
            },
            None => panic!(),
        };

        let actual_ethernet = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            destination_mac: MacAddress::try_from("01:00:5E:00:00:FC").unwrap(),
            source_mac: MacAddress::try_from("40:61:86:9A:F1:F5").unwrap(),
            ether_type: EtherType::Ipv4,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_ipv4 = match metadata.layers[1].clone() {
            ProtocolData::IPv4(value) => value,
            _ => panic!(),
        };

        let expected_ipv4 = IPv4 {
            version: 4,
            internet_header_length: 20,
            differentiated_services_code_point: 0,
            explicit_congestion_notification: 0,
            total_length: 50,
            identification: 0x6a3d,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 1,
            protocol_inner: IpNextLevelProtocol::UDP,
            checksum: 0xaa56,
            address_source: Ipv4Addr::new(192, 168, 3, 131),
            address_destination: Ipv4Addr::new(224, 0, 0, 252),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_udp = match metadata.layers[2].clone() {
            ProtocolData::UDP(value) => value,
            _ => panic!(),
        };

        let expected_udp = UDP {
            port_source: 54600,
            port_destination: 5355,
            length: 30,
            checksum: 0x2088,
        };

        assert_eq!(actual_udp, expected_udp);
    }

    #[test]
    fn test_udp_with_ethernet_padding() {
        let hex_actual = "00 1E 68 51 4F A9 00 1A 8C 10 AD 30 08 00 45 00 00 2D 41 23 00 00 71 11 F7 55 58 C6 0D 6F AC 10 FF 01 53 78 C7 27 00 19 BB 26 B4 DC 02 B9 57 01 B1 11 45 7F BB A7 6C 79 6D 64 7E 00".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 60,
            len: 0,
        };

        let parser = ProtocolParser::new(&pcap::Linktype(1), false);
        let packet = pcap::Packet {
            header: &pcap::PacketHeader::from(&header),
            data: &frame,
        };
        let result = parser.process(packet);
        let metadata = match result {
            Some(value) => match value {
                FrameType::Metadata(value) => value,
                FrameType::Raw(_) => panic!(),
            },
            None => panic!(),
        };

        let actual_ethernet = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            destination_mac: MacAddress::try_from("00:1E:68:51:4F:A9").unwrap(),
            source_mac: MacAddress::try_from("00:1A:8C:10:AD:30").unwrap(),
            ether_type: EtherType::Ipv4,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_ipv4 = match metadata.layers[1].clone() {
            ProtocolData::IPv4(value) => value,
            _ => panic!(),
        };

        let expected_ipv4 = IPv4 {
            version: 4,
            internet_header_length: 20,
            differentiated_services_code_point: 0,
            explicit_congestion_notification: 0,
            total_length: 45,
            identification: 0x4123,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 113,
            protocol_inner: IpNextLevelProtocol::UDP,
            checksum: 0xf755,
            address_source: Ipv4Addr::new(88, 198, 13, 111),
            address_destination: Ipv4Addr::new(172, 16, 255, 1),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_udp = match metadata.layers[2].clone() {
            ProtocolData::UDP(value) => value,
            _ => panic!(),
        };

        let expected_udp = UDP {
            port_source: 21368,
            port_destination: 50983,
            length: 25,
            checksum: 0xbb26,
        };

        assert_eq!(actual_udp, expected_udp);
    }
}
