use crate::frame::FrameMetadata;
use crate::parser::ParserError;
use crate::protocols::{ProtocolData, ProtocolId, dns};
use nom::number::{be_u16, be_u32};
use nom::{IResult, Parser, bits};
use serde::{Deserialize, Serialize};

// TCP Protocol
// RFC 9293: https://datatracker.ietf.org/doc/html/rfc9293

pub const DATA_OFFSET_LENGTH_BITS: usize = 4;
pub const RESERVED_LENGTH_BITS: usize = 4;
pub const FLAG_LENGTH_BITS: usize = 1;
pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    // Source port. 2 bytes
    let (rest, port_source) = be_u16().parse(bytes)?;
    // Destination port. 2 bytes
    let (rest, port_destination) = be_u16().parse(rest)?;

    // Sequence number, 4 bytes
    let (rest, sequence_number) = be_u32().parse(rest)?;
    // Acknowledgement number, 4 bytes
    let (rest, acknowledgement_number) = be_u32().parse(rest)?;

    // Data Offset, Reserved. Both - 4 bits
    let (rest, (data_offset, reserved)): (&[u8], (u8, u8)) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>(nom::sequence::pair(
            bits::complete::take(DATA_OFFSET_LENGTH_BITS),
            bits::complete::take(RESERVED_LENGTH_BITS),
        ))(rest)?;
    // Data Offset is stored in 32bit words. So, we are doing DOffset * 32 / 8 (bits in bytes)
    let data_offset = data_offset * 4;

    // Already parsed 13 bytes, so doing sub 13.
    let payload = rest
        .get(data_offset as usize - 13..)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;
    let rest = rest
        .get(..data_offset as usize - 13)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;

    // Flags: 8 flags by 1 bit.
    type TcpFlags = (u8, u8, u8, u8, u8, u8, u8, u8);
    let (rest, (cwr, ece, urg, ack, psh, rst, syn, fin)): (&[u8], TcpFlags) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>((
            bits::complete::take(FLAG_LENGTH_BITS),
            bits::complete::take(FLAG_LENGTH_BITS),
            bits::complete::take(FLAG_LENGTH_BITS),
            bits::complete::take(FLAG_LENGTH_BITS),
            bits::complete::take(FLAG_LENGTH_BITS),
            bits::complete::take(FLAG_LENGTH_BITS),
            bits::complete::take(FLAG_LENGTH_BITS),
            bits::complete::take(FLAG_LENGTH_BITS),
        ))(rest)?;
    let flags: [u8; 8] = [cwr, ece, urg, ack, psh, rst, syn, fin];

    // Window: 2 bytes.
    let (rest, window) = be_u16().parse(rest)?;
    // Checksum: 2 bytes.
    let (rest, checksum) = be_u16().parse(rest)?;
    // Urgent pointer: 2 bytes.
    let (rest, urgent_pointer) = be_u16().parse(rest)?;

    // Options - up to 320 bits.
    let options = rest.to_vec();

    let protocol = TCP {
        port_source,
        port_destination,
        sequence_number,
        acknowledgement_number,
        data_offset,
        reserved,
        flags,
        window,
        checksum,
        urgent_pointer,
        options,
    };

    Ok((payload, ProtocolData::TCP(protocol)))
}

pub fn best_children(metadata: &FrameMetadata) -> Option<ProtocolId> {
    // Checking ports
    let layer = match metadata.layers.last() {
        Some(ProtocolData::TCP(value)) => value,
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
pub struct TCP {
    pub port_source: u16,
    pub port_destination: u16,
    pub sequence_number: u32,
    pub acknowledgement_number: u32,
    pub data_offset: u8,
    pub reserved: u8,
    pub flags: [u8; 8],
    pub window: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
    pub options: Vec<u8>,
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
    fn test_tcp_with_options() {
        let hex_actual = "40 61 86 9A F1 F5 00 1A 8C 15 F9 80 08 00 45 00 00 34 94 15 00 00 34 06 11 0F 48 0E D5 66 C0 A8 03 83 00 50 DA 8E B2 61 2D 93 5D 1A BE A5 80 12 16 58 A0 94 00 00 02 04 05 96 01 01 04 02 01 03 03 06".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 66,
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
            total_length: 52,
            identification: 0x9415,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 52,
            protocol_inner: IpNextLevelProtocol::TCP,
            checksum: 0x110f,
            address_source: Ipv4Addr::new(72, 14, 213, 102),
            address_destination: Ipv4Addr::new(192, 168, 3, 131),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_tcp = match metadata.layers[2].clone() {
            ProtocolData::TCP(value) => value,
            _ => panic!(),
        };

        let expected_tcp = TCP {
            port_source: 80,
            port_destination: 55950,
            sequence_number: 2992713107,
            acknowledgement_number: 0x5d1abea5,
            data_offset: 32,
            reserved: 0,
            flags: [0, 0, 0, 1, 0, 0, 1, 0],
            window: 5720,
            checksum: 0xa094,
            urgent_pointer: 0,
            options: vec![
                u8::from_str_radix("02", 16).unwrap(),
                u8::from_str_radix("04", 16).unwrap(),
                u8::from_str_radix("05", 16).unwrap(),
                u8::from_str_radix("96", 16).unwrap(),
                u8::from_str_radix("01", 16).unwrap(),
                u8::from_str_radix("01", 16).unwrap(),
                u8::from_str_radix("04", 16).unwrap(),
                u8::from_str_radix("02", 16).unwrap(),
                u8::from_str_radix("01", 16).unwrap(),
                u8::from_str_radix("03", 16).unwrap(),
                u8::from_str_radix("03", 16).unwrap(),
                u8::from_str_radix("06", 16).unwrap(),
            ],
        };

        assert_eq!(actual_tcp, expected_tcp);
    }

    #[test]
    fn test_tcp_without_options() {
        let hex_actual = "40 61 86 9A F1 F5 00 1A 8C 15 F9 80 08 00 45 00 00 56 2B 9A 00 00 34 06 79 3B 48 0E D5 93 C0 A8 03 83 01 BB CB B8 EE BA 28 1D 18 D9 BD 5F 50 18 00 D5 37 24 00 00 DE A9 06 7D DE 13 B6 78 A0 EA 50 53 29 A3 75 9C 1B B3 B0 3B 4D E5 21 DD 11 D4 75 A8 79 D5 58 B6 9F 6D 32 EA 72 F8 B0 54 C3 2F E9 AF 98 E4".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 100,
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
            total_length: 86,
            identification: 0x2b9a,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 52,
            protocol_inner: IpNextLevelProtocol::TCP,
            checksum: 0x793b,
            address_source: Ipv4Addr::new(72, 14, 213, 147),
            address_destination: Ipv4Addr::new(192, 168, 3, 131),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_tcp = match metadata.layers[2].clone() {
            ProtocolData::TCP(value) => value,
            _ => panic!(),
        };

        let expected_tcp = TCP {
            port_source: 443,
            port_destination: 52152,
            sequence_number: 4005177373,
            acknowledgement_number: 416922975,
            data_offset: 20,
            reserved: 0,
            flags: [0, 0, 0, 1, 1, 0, 0, 0],
            window: 213,
            checksum: 0x3724,
            urgent_pointer: 0,
            options: vec![],
        };

        assert_eq!(actual_tcp, expected_tcp);
    }
}
