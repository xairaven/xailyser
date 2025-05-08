use crate::protocols::ProtocolData;
use nom::IResult;
use nom::number::{be_u8, be_u16};
use nom::{Finish, Parser};
use serde::{Deserialize, Serialize};

// ICMPv6 Protocol
// RFC 4443: https://datatracker.ietf.org/doc/html/rfc4443

pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    // Message type. 1 byte
    let (rest, message_type) = be_u8().parse(bytes)?;

    // Code. 1 byte
    let (rest, code) = be_u8().parse(rest)?;

    // Checksum. 2 bytes
    let (rest, checksum) = be_u16().parse(rest)?;

    // Data, depending on type & code.
    let data = rest.to_vec();

    let protocol = ICMPv6 {
        message_type,
        code,
        checksum,
        data,
    };

    Finish::finish(Ok((rest, ProtocolData::ICMPv6(protocol))))
}

#[derive(Clone, Debug, PartialEq)]
pub struct ICMPv6 {
    pub message_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ICMPv6Dto {
    pub message_type: u8,
    pub code: u8,
}

impl From<ICMPv6> for ICMPv6Dto {
    fn from(value: ICMPv6) -> Self {
        Self {
            message_type: value.message_type,
            code: value.code,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dto::frame::FrameHeader;
    use crate::parser::tests::FrameType;
    use crate::parser::tests::ProtocolParser;
    use crate::protocols::ProtocolData;
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;
    use crate::protocols::icmpv6::ICMPv6;
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv6::IPv6;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

    #[test]
    fn test_icmpv6_advertisement() {
        let hex_actual = "00 60 97 07 69 EA 00 00 86 05 80 DA 86 DD 60 00 00 00 00 18 3A FF FE 80 00 00 00 00 00 00 02 00 86 FF FE 05 80 DA FE 80 00 00 00 00 00 00 02 60 97 FF FE 07 69 EA 88 00 2A 18 40 00 00 00 FE 80 00 00 00 00 00 00 02 00 86 FF FE 05 80 DA".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 78,
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
            destination_mac: MacAddress::try_from("00:60:97:07:69:EA").unwrap(),
            source_mac: MacAddress::try_from("00:00:86:05:80:DA").unwrap(),
            ether_type: EtherType::Ipv6,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_ipv6 = match metadata.layers[1].clone() {
            ProtocolData::IPv6(value) => value,
            _ => panic!(),
        };

        let expected_ipv6 = IPv6 {
            version: 6,
            traffic_class: 0,
            flow_label: 0,
            payload_length: 24,
            next_header: IpNextLevelProtocol::Ipv6Icmp,
            hop_limit: 255,
            address_source: Ipv6Addr::from_str("fe80::200:86ff:fe05:80da").unwrap(),
            address_destination: Ipv6Addr::from_str("fe80::260:97ff:fe07:69ea").unwrap(),
        };

        assert_eq!(actual_ipv6, expected_ipv6);

        let actual_icmp = match metadata.layers[2].clone() {
            ProtocolData::ICMPv6(value) => value,
            _ => panic!(),
        };

        let expected_icmp = ICMPv6 {
            message_type: 136,
            code: 0,
            checksum: 0x2a18,
            data: vec![
                0x40, 0x00, 0x00, 0x00, 0xFE, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x02, 0x00, 0x86, 0xFF, 0xFE, 0x05, 0x80, 0xDA,
            ],
        };

        assert_eq!(actual_icmp, expected_icmp);
    }

    #[test]
    fn test_icmpv6_ping() {
        let hex_actual = "00 00 86 05 80 DA 00 60 97 07 69 EA 86 DD 60 00 00 00 00 10 3A 40 3F FE 05 07 00 00 00 01 02 60 97 FF FE 07 69 EA 3F FE 05 07 00 00 00 01 02 00 86 FF FE 05 80 DA 81 00 1E 76 7B 20 00 00 19 C9 E7 36 44 E0 0B 00".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 70,
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
            destination_mac: MacAddress::try_from("00:00:86:05:80:DA").unwrap(),
            source_mac: MacAddress::try_from("00:60:97:07:69:EA").unwrap(),
            ether_type: EtherType::Ipv6,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_ipv6 = match metadata.layers[1].clone() {
            ProtocolData::IPv6(value) => value,
            _ => panic!(),
        };

        let expected_ipv6 = IPv6 {
            version: 6,
            traffic_class: 0,
            flow_label: 0,
            payload_length: 16,
            next_header: IpNextLevelProtocol::Ipv6Icmp,
            hop_limit: 64,
            address_source: Ipv6Addr::from_str("3ffe:507:0:1:260:97ff:fe07:69ea")
                .unwrap(),
            address_destination: Ipv6Addr::from_str("3ffe:507:0:1:200:86ff:fe05:80da")
                .unwrap(),
        };

        assert_eq!(actual_ipv6, expected_ipv6);

        let actual_icmp = match metadata.layers[2].clone() {
            ProtocolData::ICMPv6(value) => value,
            _ => panic!(),
        };

        let expected_icmp = ICMPv6 {
            message_type: 129,
            code: 0,
            checksum: 0x1e76,
            data: vec![
                0x7B, 0x20, 0x00, 0x00, 0x19, 0xC9, 0xE7, 0x36, 0x44, 0xE0, 0x0B, 0x00,
            ],
        };

        assert_eq!(actual_icmp, expected_icmp);
    }
}
