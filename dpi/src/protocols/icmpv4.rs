use crate::protocols::ProtocolData;
use nom::IResult;
use nom::number::{be_u8, be_u16};
use nom::{Finish, Parser};
use serde::{Deserialize, Serialize};

// ICMPv4 Protocol
// RFC 792: https://datatracker.ietf.org/doc/html/rfc792

pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    // Message type. 1 byte
    let (rest, message_type) = be_u8().parse(bytes)?;

    // Code. 1 byte
    let (rest, code) = be_u8().parse(rest)?;

    // Checksum. 2 bytes
    let (rest, checksum) = be_u16().parse(rest)?;

    // Data, depending on type & code.
    let data = rest.to_vec();

    let protocol = ICMPv4 {
        message_type,
        code,
        checksum,
        data,
    };

    Finish::finish(Ok((rest, ProtocolData::ICMPv4(protocol))))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ICMPv4 {
    pub message_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ICMPv4Dto {
    pub message_type: u8,
    pub code: u8,
}

impl From<ICMPv4> for ICMPv4Dto {
    fn from(value: ICMPv4) -> Self {
        Self {
            message_type: value.message_type,
            code: value.code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::frame::FrameHeader;
    use crate::parser::tests::FrameType;
    use crate::parser::tests::ProtocolParser;
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv4::IPv4;
    use std::net::Ipv4Addr;

    #[test]
    fn test_icmpv4() {
        let hex_actual = "00 1A 8C 10 AD 30 00 1E 68 51 4F A9 08 00 45 00 00 3C 7E 74 00 00 20 01 EB DF AC 10 FF 01 43 D7 41 84 08 00 40 08 00 01 0F 55 41 42 43 44 45 46 47 48 49 4A 4B 4C 4D 4E 4F 50 51 52 53 54 55 56 57 41 42 43 44 45 46 47 48 49".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 74,
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
            destination_mac: MacAddress::try_from("00:1A:8C:10:AD:30").unwrap(),
            source_mac: MacAddress::try_from("00:1E:68:51:4F:A9").unwrap(),
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
            total_length: 60,
            identification: 0x7e74,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 32,
            protocol_inner: IpNextLevelProtocol::ICMP,
            checksum: 0xebdf,
            address_source: Ipv4Addr::new(172, 16, 255, 1),
            address_destination: Ipv4Addr::new(67, 215, 65, 132),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_icmp = match metadata.layers[2].clone() {
            ProtocolData::ICMPv4(value) => value,
            _ => panic!(),
        };

        let expected_icmp = ICMPv4 {
            message_type: 8,
            code: 0,
            checksum: 0x4008,
            data: vec![
                0x00, 0x01, 0x0F, 0x55, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
                0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54,
                0x55, 0x56, 0x57, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
            ],
        };

        assert_eq!(actual_icmp, expected_icmp);
    }
}
