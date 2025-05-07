use crate::dto::frame::FrameMetadata;
use crate::parser::ParserError;
use crate::protocols::ip::protocol::IpNextLevelProtocol;
use crate::protocols::{ProtocolData, ProtocolId, ip};
use nom::Parser;
use nom::number::{be_u8, be_u16};
use nom::{IResult, bits, sequence};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

// IPv4 Protocol
// RFC 791: https://datatracker.ietf.org/doc/html/rfc791
pub const PROTOCOL_VERSION_LENGTH_BITS: usize = 4;
pub const IHL_LENGTH_BITS: usize = 4;
pub const DSCP_LENGTH_BITS: usize = 6;
pub const ECN_LENGTH_BITS: usize = 2;
pub const FLAGS_LENGTH_BITS: usize = 3;
pub const FRAGMENT_OFFSET_LENGTH_BITS: usize = 13;
pub const PACKET_NECESSARY_LENGTH_BYTES: usize = 20;
pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    // Version (4 bits), Internet Header Length (4 bits)
    let (rest, (version, ihl)): (&[u8], (u8, u16)) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>(sequence::pair(
            bits::complete::take(PROTOCOL_VERSION_LENGTH_BITS),
            bits::complete::take(IHL_LENGTH_BITS),
        ))(bytes)?;
    if version != 4 {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }
    // IHL is stored in 32bit words. So, we are doing IHL * 32 / 8 (bits in bytes)
    let ihl = ihl
        .checked_mul(4)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;

    // Differentiated Services Code Point (6 bits), Explicit Congestion Notification (2 bits)
    let (rest, (dscp, ecn)): (&[u8], (u8, u8)) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>(sequence::pair(
            bits::complete::take(DSCP_LENGTH_BITS),
            bits::complete::take(ECN_LENGTH_BITS),
        ))(rest)?;

    // Total Length
    let (rest, total_len) = be_u16().parse(rest)?;

    // Totally parsed = 4 bytes. So, we can cut ethernet padding there.
    let packet = rest
        .get(
            ..total_len
                .checked_sub(4)
                .ok_or(ParserError::ErrorVerify.to_nom(bytes))? as usize,
        )
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;
    let boundary = ihl
        .checked_sub(4)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))? as usize;
    let rest = packet
        .get(..boundary)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;
    let payload = packet
        .get(boundary..)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;

    // Identification - 2 bytes
    let (rest, identification) = be_u16().parse(rest)?;

    // Flags, Fragment offset - 16 bits.
    let (rest, (flags, fragment_offset)): (&[u8], (u8, u16)) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>(sequence::pair(
            bits::complete::take(FLAGS_LENGTH_BITS),
            bits::complete::take(FRAGMENT_OFFSET_LENGTH_BITS),
        ))(rest)?;

    // Time To Live
    let (rest, ttl) = be_u8().parse(rest)?;

    // Protocol field
    let (rest, inner_protocol) = be_u8().parse(rest)?;
    let protocol_inner = IpNextLevelProtocol::try_from(inner_protocol)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // Checksum
    let (rest, checksum) = be_u16().parse(rest)?;

    // Source Address
    let (rest, address_source) = ip::address::v4_parse(rest)?;
    // Destination Address
    let (rest, address_destination) = ip::address::v4_parse(rest)?;

    if rest.len()
        != (ihl as usize)
            .checked_sub(PACKET_NECESSARY_LENGTH_BYTES)
            .ok_or(ParserError::ErrorVerify.to_nom(bytes))?
    {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    let protocol = IPv4 {
        version,
        internet_header_length: ihl,
        differentiated_services_code_point: dscp,
        explicit_congestion_notification: ecn,
        total_length: total_len,
        identification,
        flags,
        fragment_offset,
        time_to_live: ttl,
        protocol_inner,
        checksum,
        address_source,
        address_destination,
    };

    Ok((payload, ProtocolData::IPv4(protocol)))
}

pub fn best_children(metadata: &FrameMetadata) -> Option<ProtocolId> {
    // Checking IP inner protocol type
    let ipv4 = match metadata.layers.last() {
        Some(ProtocolData::IPv4(value)) => value,
        _ => return None,
    };
    match ipv4.protocol_inner {
        IpNextLevelProtocol::ICMP => Some(ProtocolId::ICMPv4),
        IpNextLevelProtocol::IPv6 => Some(ProtocolId::IPv6),
        IpNextLevelProtocol::TCP => Some(ProtocolId::TCP),
        IpNextLevelProtocol::UDP => Some(ProtocolId::UDP),
        _ => None,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IPv4 {
    pub version: u8,
    pub internet_header_length: u16,
    pub differentiated_services_code_point: u8,
    pub explicit_congestion_notification: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub time_to_live: u8,
    pub protocol_inner: IpNextLevelProtocol,
    pub checksum: u16,
    pub address_source: Ipv4Addr,
    pub address_destination: Ipv4Addr,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::frame::{FrameHeader, FrameType};
    use crate::parser::ProtocolParser;
    use crate::protocols::ProtocolData;
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;

    #[test]
    fn test_ipv4_with_ethernet_padding() {
        let hex_actual = "40 61 86 9A F1 F5 00 1A 8C 15 F9 80 08 00 45 00 00 28 2B AE 00 00 34 06 79 55 48 0E D5 93 C0 A8 03 83 01 BB CB B8 EE BA 6C 0B 18 D9 CD D6 50 10 01 B4 BD 69 00 00 00 00 00 00 00 00".replace(" ", "");
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
                _ => panic!(),
            },
            None => panic!(),
        };

        let actual_ethernet = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            destination_mac: MacAddress::try_from("40:61:86:9A:F1:F5").unwrap(),
            source_mac: MacAddress::try_from("00:1A:8C:15:F9:80").unwrap(),
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
            total_length: 40,
            identification: 0x2bae,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 52,
            protocol_inner: IpNextLevelProtocol::TCP,
            checksum: 0x7955,
            address_source: Ipv4Addr::new(72, 14, 213, 147),
            address_destination: Ipv4Addr::new(192, 168, 3, 131),
        };

        assert_eq!(actual_ipv4, expected_ipv4);
    }

    #[test]
    fn test_ipv4_without_ethernet_padding() {
        let hex_actual = "40 61 86 9A F1 F5 00 1A 8C 15 F9 80 08 00 45 00 00 63 2B B1 00 00 34 06 79 17 48 0E D5 93 C0 A8 03 83 01 BB CB B8 EE BA 71 7C 18 D9 CD D6 50 18 01 B4 3D CA 00 00 17 03 01 00 36 B5 2A 58 A3 3D BD EC F3 7C C9 C4 43 B9 5D 94 C9 3D 9D E5 75 11 47 6E 2E A0 E0 8B 1B 64 44 BE D8 06 FE 5B 00 69 B3 12 D0 D9 37 87 87 F4 1C 42 E3 00 16 EE 14 CA 69".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 113,
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
            destination_mac: MacAddress::try_from("40:61:86:9A:F1:F5").unwrap(),
            source_mac: MacAddress::try_from("00:1A:8C:15:F9:80").unwrap(),
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
            total_length: 99,
            identification: 0x2bb1,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 52,
            protocol_inner: IpNextLevelProtocol::TCP,
            checksum: 0x7917,
            address_source: Ipv4Addr::new(72, 14, 213, 147),
            address_destination: Ipv4Addr::new(192, 168, 3, 131),
        };

        assert_eq!(actual_ipv4, expected_ipv4);
    }
}
