use crate::frame::FrameMetadata;
use crate::parser::ParserError;
use crate::protocols::ip::protocol::IpNextLevelProtocol;
use crate::protocols::{ProtocolData, ProtocolId, ip};
use nom::Parser;
use nom::number::{be_u8, be_u16};
use nom::{IResult, bits};
use serde::{Deserialize, Serialize};
use std::net::Ipv6Addr;

// IPv6 Protocol
// RFC 8200: https://datatracker.ietf.org/doc/html/rfc8200

pub const VERSION_LENGTH_BITS: usize = 4;
pub const TRAFFIC_CLASS_LENGTH_BITS: usize = 8;
pub const FLOW_LABEL_LENGTH_BITS: usize = 20;
pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    // Version (4 bits), Traffic Class (8 bits), Flow Label (20 bits)
    let (rest, (version, traffic_class, flow_label)): (&[u8], (u8, u8, u32)) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>((
            bits::complete::take(VERSION_LENGTH_BITS),
            bits::complete::take(TRAFFIC_CLASS_LENGTH_BITS),
            bits::complete::take(FLOW_LABEL_LENGTH_BITS),
        ))(bytes)?;
    if version != 6 {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    // Payload Length (2 bytes)
    let (rest, payload_length) = be_u16().parse(rest)?;

    // Next Header (1 byte)
    let (rest, next_header) = be_u8().parse(rest)?;
    let next_header = IpNextLevelProtocol::try_from(next_header)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // Hop Limit (1 byte)
    let (rest, hop_limit) = be_u8().parse(rest)?;

    // Source Address
    let (rest, address_source) = ip::address::v6_parse(rest)?;
    // Destination Address
    let (rest, address_destination) = ip::address::v6_parse(rest)?;

    // Cutting ethernet padding
    let payload = rest
        .get(..payload_length as usize)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;

    let protocol = IPv6 {
        version,
        traffic_class,
        flow_label,
        payload_length,
        next_header,
        hop_limit,
        address_source,
        address_destination,
    };

    Ok((payload, ProtocolData::IPv6(protocol)))
}

pub fn best_children(metadata: &FrameMetadata) -> Option<ProtocolId> {
    // Checking IP inner protocol type
    let ipv6 = match metadata.layers.last() {
        Some(ProtocolData::IPv6(value)) => value,
        _ => return None,
    };
    match ipv6.next_header {
        IpNextLevelProtocol::Ipv6Icmp => Some(ProtocolId::ICMPv6),
        IpNextLevelProtocol::IPv6 => Some(ProtocolId::IPv6),
        IpNextLevelProtocol::TCP => Some(ProtocolId::TCP),
        IpNextLevelProtocol::UDP => Some(ProtocolId::UDP),
        _ => None,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IPv6 {
    pub version: u8,
    pub traffic_class: u8,
    pub flow_label: u32,
    pub payload_length: u16,
    pub next_header: IpNextLevelProtocol,
    pub hop_limit: u8,
    pub address_source: Ipv6Addr,
    pub address_destination: Ipv6Addr,
}

#[cfg(test)]
mod tests {
    use crate::frame::FrameType;
    use crate::parser::ProtocolParser;
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv4::IPv4;
    use crate::protocols::ipv6::IPv6;
    use crate::protocols::tcp::TCP;
    use crate::protocols::{ProtocolData, tcp};
    use crate::wrapper::FrameHeader;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    #[test]
    fn test_ipv6_tcp_http() {
        let hex_actual = "22 1A 95 D6 7A 23 86 93 23 D3 37 8E 86 DD 60 0D 68 4A 00 7D 06 40 FC 00 00 02 00 00 00 02 00 00 00 00 00 00 00 01 FC 00 00 02 00 00 00 01 00 00 00 00 00 00 00 01 A9 A0 1F 90 02 1B 63 8D BA 31 1E 8E 80 18 00 CF C9 2E 00 00 01 01 08 0A 80 1D A5 22 80 1D A5 22 47 45 54 20 2F 68 65 6C 6C 6F 2E 74 78 74 20 48 54 54 50 2F 31 2E 31 0D 0A 55 73 65 72 2D 41 67 65 6E 74 3A 20 63 75 72 6C 2F 37 2E 33 38 2E 30 0D 0A 48 6F 73 74 3A 20 5B 66 63 30 30 3A 32 3A 30 3A 31 3A 3A 31 5D 3A 38 30 38 30 0D 0A 41 63 63 65 70 74 3A 20 2A 2F 2A 0D 0A 0D 0A".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 179,
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
                _ => panic!(),
            },
            None => panic!(),
        };

        let actual_ethernet = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            destination_mac: MacAddress::try_from("22:1A:95:D6:7A:23").unwrap(),
            source_mac: MacAddress::try_from("86:93:23:D3:37:8E").unwrap(),
            ether_type: EtherType::Ipv6,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_ipv6 = match metadata.layers[1].clone() {
            ProtocolData::IPv6(value) => value,
            _ => panic!(),
        };

        let expected_ipv6 = IPv6 {
            version: 6,
            traffic_class: 0x00,
            flow_label: 0xd684a,
            payload_length: 125,
            next_header: IpNextLevelProtocol::TCP,
            hop_limit: 64,
            address_source: Ipv6Addr::from_str("fc00:2:0:2::1").unwrap(),
            address_destination: Ipv6Addr::from_str("fc00:2:0:1::1").unwrap(),
        };

        assert_eq!(actual_ipv6, expected_ipv6);

        let actual_tcp = match metadata.layers[2].clone() {
            ProtocolData::TCP(value) => value,
            _ => panic!(),
        };

        let expected_tcp = TCP {
            port_source: 43424,
            port_destination: 8080,
            sequence_number: 0x021b638d,
            acknowledgement_number: 0xba311e8e,
            data_offset: 32,
            reserved: 0,
            flags: tcp::Flags {
                congestion_window_reduced: false,
                ecn_echo: false,
                urgent: false,
                acknowledgment: true,
                push: true,
                reset: false,
                syn: false,
                fin: false,
            },
            window: 207,
            checksum: 0xc92e,
            urgent_pointer: 0,
            options: vec![
                tcp::OptionData::NoOperation,
                tcp::OptionData::NoOperation,
                tcp::OptionData::Timestamps(2149426466, 2149426466),
            ],
        };

        assert_eq!(actual_tcp, expected_tcp);
    }

    #[test]
    fn test_ipv6_tcp() {
        let hex_actual = "22 1A 95 D6 7A 23 86 93 23 D3 37 8E 86 DD 60 0D 68 4A 00 20 06 40 FC 00 00 02 00 00 00 02 00 00 00 00 00 00 00 01 FC 00 00 02 00 00 00 01 00 00 00 00 00 00 00 01 A9 A0 1F 90 02 1B 63 EB BA 31 1F 86 80 10 00 D8 2A 66 00 00 01 01 08 0A 80 1D A5 25 80 1D A5 25".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 86,
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
                _ => panic!(),
            },
            None => panic!(),
        };

        let actual_ethernet = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            destination_mac: MacAddress::try_from("22:1A:95:D6:7A:23").unwrap(),
            source_mac: MacAddress::try_from("86:93:23:D3:37:8E").unwrap(),
            ether_type: EtherType::Ipv6,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_ipv6 = match metadata.layers[1].clone() {
            ProtocolData::IPv6(value) => value,
            _ => panic!(),
        };

        let expected_ipv6 = IPv6 {
            version: 6,
            traffic_class: 0x00,
            flow_label: 0xd684a,
            payload_length: 32,
            next_header: IpNextLevelProtocol::TCP,
            hop_limit: 64,
            address_source: Ipv6Addr::from_str("fc00:2:0:2::1").unwrap(),
            address_destination: Ipv6Addr::from_str("fc00:2:0:1::1").unwrap(),
        };

        assert_eq!(actual_ipv6, expected_ipv6);

        let actual_tcp = match metadata.layers[2].clone() {
            ProtocolData::TCP(value) => value,
            _ => panic!(),
        };

        let expected_tcp = TCP {
            port_source: 43424,
            port_destination: 8080,
            sequence_number: 0x021b63eb,
            acknowledgement_number: 0xba311f86,
            data_offset: 32,
            reserved: 0,
            flags: tcp::Flags {
                congestion_window_reduced: false,
                ecn_echo: false,
                urgent: false,
                acknowledgment: true,
                push: false,
                reset: false,
                syn: false,
                fin: false,
            },
            window: 216,
            checksum: 0x2a66,
            urgent_pointer: 0,
            options: vec![
                tcp::OptionData::NoOperation,
                tcp::OptionData::NoOperation,
                tcp::OptionData::Timestamps(2149426469, 2149426469),
            ],
        };

        assert_eq!(actual_tcp, expected_tcp);
    }

    #[test]
    fn test_ipv4_ipv6_tcp() {
        let hex_actual = "01 00 01 00 00 00 1A 43 20 00 01 00 08 00 45 00 00 81 2A 50 00 00 10 29 46 CB 8B 12 19 21 51 83 43 83 60 04 40 E8 00 45 06 3F 20 01 06 38 09 02 00 01 02 01 02 FF FE E2 75 96 20 02 51 83 43 83 00 00 00 00 00 00 51 83 43 83 00 15 04 02 E5 37 A5 73 62 6B F3 08 50 18 81 60 98 72 00 00 33 33 31 20 47 75 65 73 74 20 6C 6F 67 69 6E 20 6F 6B 2C 20 74 79 70 65 20 79 6F 75 72 20 6E 61 6D 65 20 61 73 20 70 61 73 73 77 6F 72 64 2E 0D 0A".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 143,
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
                _ => panic!(),
            },
            None => panic!(),
        };

        let actual_ethernet = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            destination_mac: MacAddress::try_from("01:00:01:00:00:00").unwrap(),
            source_mac: MacAddress::try_from("1A:43:20:00:01:00").unwrap(),
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
            total_length: 129,
            identification: 0x2a50,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 16,
            protocol_inner: IpNextLevelProtocol::IPv6,
            checksum: 0x46cb,
            address_source: Ipv4Addr::from_str("139.18.25.33").unwrap(),
            address_destination: Ipv4Addr::from_str("81.131.67.131").unwrap(),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_ipv6 = match metadata.layers[2].clone() {
            ProtocolData::IPv6(value) => value,
            _ => panic!(),
        };

        let expected_ipv6 = IPv6 {
            version: 6,
            traffic_class: 0x00,
            flow_label: 0x440e8,
            payload_length: 69,
            next_header: IpNextLevelProtocol::TCP,
            hop_limit: 63,
            address_source: Ipv6Addr::from_str("2001:638:902:1:201:2ff:fee2:7596")
                .unwrap(),
            address_destination: Ipv6Addr::from_str("2002:5183:4383::5183:4383").unwrap(),
        };

        assert_eq!(actual_ipv6, expected_ipv6);

        let actual_tcp = match metadata.layers[3].clone() {
            ProtocolData::TCP(value) => value,
            _ => panic!(),
        };

        let expected_tcp = TCP {
            port_source: 21,
            port_destination: 1026,
            sequence_number: 0xe537a573,
            acknowledgement_number: 0x626bf308,
            data_offset: 20,
            reserved: 0,
            flags: tcp::Flags {
                congestion_window_reduced: false,
                ecn_echo: false,
                urgent: false,
                acknowledgment: true,
                push: true,
                reset: false,
                syn: false,
                fin: false,
            },
            window: 33120,
            checksum: 0x9872,
            urgent_pointer: 0,
            options: vec![],
        };

        assert_eq!(actual_tcp, expected_tcp);
    }
}
