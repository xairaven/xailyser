use crate::dto::frame::FrameMetadata;
use crate::parser::{ParserError, ProtocolParser};
use crate::protocols::{ProtocolData, ProtocolId};
use nom::number::{be_u8, be_u16, be_u32, be_u64, be_u128};
use nom::{IResult, Parser, bits};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
// TCP Protocol
// RFC 9293: https://datatracker.ietf.org/doc/html/rfc9293

pub const DATA_OFFSET_LENGTH_BITS: usize = 4;
pub const RESERVED_LENGTH_BITS: usize = 4;
pub const FLAG_LENGTH_BITS: usize = 1;
type TcpFlags = (u8, u8, u8, u8, u8, u8, u8, u8);
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
    let (rest, (data_offset, reserved)): (&[u8], (u16, u8)) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>(nom::sequence::pair(
            bits::complete::take(DATA_OFFSET_LENGTH_BITS),
            bits::complete::take(RESERVED_LENGTH_BITS),
        ))(rest)?;
    // Data Offset is stored in 32bit words. So, we are doing DOffset * 32 / 8 (bits in bytes)
    let data_offset = data_offset
        .checked_mul(4)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;

    // Already parsed 13 bytes, so doing sub 13.
    let boundary = data_offset
        .checked_sub(13)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))? as usize;
    let payload = rest
        .get(boundary..)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;
    let rest = rest
        .get(..boundary)
        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;

    // Flags: 8 flags by 1 bit.
    let (rest, flags): (&[u8], TcpFlags) =
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
    let flags =
        Flags::try_from(flags).map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // Window: 2 bytes.
    let (rest, window) = be_u16().parse(rest)?;
    // Checksum: 2 bytes.
    let (rest, checksum) = be_u16().parse(rest)?;
    // Urgent pointer: 2 bytes.
    let (rest, urgent_pointer) = be_u16().parse(rest)?;

    // Options - up to 320 bits.
    let mut options: Vec<OptionData> = Vec::new();
    let mut option_bytes_buffer = rest;
    while !option_bytes_buffer.is_empty() {
        let (rest, kind) = be_u8().parse(option_bytes_buffer)?;
        let (rest, value) = OptionId::try_from(kind)
            .map_err(|_| ParserError::ErrorVerify.to_nom(rest))?
            .parse(rest)?;
        options.push(value);
        option_bytes_buffer = rest;
    }

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

    for children in ProtocolId::TCP.children()? {
        if let Some(port_validate) = children.check_ports() {
            let applicable = port_validate(layer.port_source, layer.port_destination);
            if applicable {
                return Some(children);
            }
        }
    }

    None
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TCP {
    pub port_source: u16,
    pub port_destination: u16,
    pub sequence_number: u32,
    pub acknowledgement_number: u32,
    pub data_offset: u16,
    pub reserved: u8,
    pub flags: Flags,
    pub window: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
    pub options: Vec<OptionData>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Flags {
    pub congestion_window_reduced: bool,
    pub ecn_echo: bool,
    pub urgent: bool,
    pub acknowledgment: bool,
    pub push: bool,
    pub reset: bool,
    pub syn: bool,
    pub fin: bool,
}

impl TryFrom<TcpFlags> for Flags {
    type Error = ParserError;

    fn try_from(value: TcpFlags) -> Result<Self, Self::Error> {
        Ok(Self {
            congestion_window_reduced: ProtocolParser::cast_to_bool(value.0)?,
            ecn_echo: ProtocolParser::cast_to_bool(value.1)?,
            urgent: ProtocolParser::cast_to_bool(value.2)?,
            acknowledgment: ProtocolParser::cast_to_bool(value.3)?,
            push: ProtocolParser::cast_to_bool(value.4)?,
            reset: ProtocolParser::cast_to_bool(value.5)?,
            syn: ProtocolParser::cast_to_bool(value.6)?,
            fin: ProtocolParser::cast_to_bool(value.7)?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum OptionId {
    EndOfOptionList = 0,
    NoOperation = 1,
    MaximumSegmentSize = 2,
    WindowScaling = 3,
    SAckPermitted = 4,
    SAck = 5,

    Timestamps = 8,
    FastOpen = 34,
}

impl OptionId {
    pub fn parse<'a>(&self, bytes: &'a [u8]) -> IResult<&'a [u8], OptionData> {
        match self {
            Self::EndOfOptionList => Ok((bytes, OptionData::EndOfOptionList)),

            Self::NoOperation => Ok((bytes, OptionData::NoOperation)),

            Self::MaximumSegmentSize => {
                let (rest, length) = be_u8().parse(bytes)?;
                if length != 4 {
                    return Err(ParserError::ErrorVerify.to_nom(bytes));
                }
                let (rest, maximum_segment_size) = be_u16().parse(rest)?;
                Ok((rest, OptionData::MaximumSegmentSize(maximum_segment_size)))
            },

            Self::WindowScaling => {
                let (rest, length) = be_u8().parse(bytes)?;
                if length != 3 {
                    return Err(ParserError::ErrorVerify.to_nom(bytes));
                }
                let (rest, window) = be_u8().parse(rest)?;
                Ok((rest, OptionData::WindowScaling(window)))
            },

            Self::SAckPermitted => {
                let (rest, length) = be_u8().parse(bytes)?;
                if length != 2 {
                    return Err(ParserError::ErrorVerify.to_nom(bytes));
                }
                Ok((rest, OptionData::SAckPermitted))
            },

            Self::SAck => {
                let (rest, length) = be_u8().parse(bytes)?;
                if length % 8 != 0 {
                    return Err(ParserError::ErrorVerify.to_nom(bytes));
                }
                let mut values = Vec::with_capacity(length as usize / 8);
                let mut buffer = rest;
                for _ in 0..(length / 8) {
                    let (rest, value) = be_u64().parse(buffer)?;
                    values.push(value);
                    buffer = rest;
                }

                Ok((rest, OptionData::SAck(values)))
            },

            Self::Timestamps => {
                let (rest, length) = be_u8().parse(bytes)?;
                if length != 10 {
                    return Err(ParserError::ErrorVerify.to_nom(bytes));
                }

                let (rest, initial_time) = be_u32().parse(rest)?;
                let (rest, reply_time) = be_u32().parse(rest)?;

                Ok((rest, OptionData::Timestamps(initial_time, reply_time)))
            },

            Self::FastOpen => {
                let (rest, length) = be_u8().parse(bytes)?;
                if length != 18 {
                    return Err(ParserError::ErrorVerify.to_nom(bytes));
                }
                let (rest, cookie) = be_u128().parse(rest)?;

                Ok((rest, OptionData::FastOpen(cookie)))
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OptionData {
    EndOfOptionList,
    NoOperation,
    MaximumSegmentSize(u16),
    WindowScaling(u8),
    SAckPermitted,
    SAck(Vec<u64>),
    Timestamps(u32, u32),
    FastOpen(u128),
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
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv4::IPv4;
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
            flags: Flags {
                congestion_window_reduced: false,
                ecn_echo: false,
                urgent: false,
                acknowledgment: true,
                push: false,
                reset: false,
                syn: true,
                fin: false,
            },
            window: 5720,
            checksum: 0xa094,
            urgent_pointer: 0,
            options: vec![
                OptionData::MaximumSegmentSize(1430),
                OptionData::NoOperation,
                OptionData::NoOperation,
                OptionData::SAckPermitted,
                OptionData::NoOperation,
                OptionData::WindowScaling(6),
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
            flags: Flags {
                congestion_window_reduced: false,
                ecn_echo: false,
                urgent: false,
                acknowledgment: true,
                push: true,
                reset: false,
                syn: false,
                fin: false,
            },
            window: 213,
            checksum: 0x3724,
            urgent_pointer: 0,
            options: vec![],
        };

        assert_eq!(actual_tcp, expected_tcp);
    }
}
