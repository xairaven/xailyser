use crate::parser::ParserError;
use crate::protocols::arp::hardware_type::HardwareType;
use crate::protocols::arp::operation::Operation;
use crate::protocols::ethernet::ether_type::EtherType;
use crate::protocols::ethernet::mac::MacAddress;
use crate::protocols::{ProtocolData, ethernet, ip};
use nom::Parser;
use nom::number::be_u8;
use nom::{Finish, IResult};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use thiserror::Error;

// ARP Protocol
// RFC 826: https://datatracker.ietf.org/doc/html/rfc826

pub const PACKET_LENGTH: usize = 28;

pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    if bytes.len() < PACKET_LENGTH {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    };

    // Cutting Ethernet padding & FCS
    let bytes = if bytes.len() > PACKET_LENGTH {
        match bytes.get(..PACKET_LENGTH) {
            Some(value) => value,
            None => return Err(ParserError::ErrorVerify.to_nom(bytes)),
        }
    } else {
        bytes
    };

    // HTYPE
    let (rest, hardware_type) = hardware_type::parse(bytes)?;

    // PTYPE
    let (rest, protocol_type) = ethernet::ether_type::parse(rest)?;
    if protocol_type != EtherType::Ipv4 {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    // HLEN
    let (rest, hardware_address_length) = be_u8().parse(rest)?;
    hardware_type
        .validate_length(hardware_address_length as usize)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // PLEN
    let (rest, protocol_address_length) = be_u8().parse(rest)?;
    if protocol_address_length != ip::address::V4_LENGTH_BYTES as u8 {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    // OP
    let (rest, operation) = operation::parse(rest)?;

    // SENDER_HARDWARE_ADDRESS
    let (rest, sender_hardware_address) = ethernet::mac::parse(rest)?;

    // SENDER_PROTOCOL_ADDRESS
    let (rest, sender_protocol_address) = ip::address::v4_parse(rest)?;

    // TARGET_HARDWARE_ADDRESS
    let (rest, target_hardware_address) = ethernet::mac::parse(rest)?;

    // TARGET_PROTOCOL_ADDRESS
    let (rest, target_protocol_address) = ip::address::v4_parse(rest)?;

    if !rest.is_empty() {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    let arp = Arp {
        hardware_type,
        protocol_type,
        hardware_address_length,
        protocol_address_length,
        operation,
        sender_mac: sender_hardware_address,
        sender_ip: sender_protocol_address,
        target_mac: target_hardware_address,
        target_ip: target_protocol_address,
    };

    Finish::finish(Ok((rest, ProtocolData::Arp(arp))))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Arp {
    pub hardware_type: HardwareType,
    pub protocol_type: EtherType,

    pub hardware_address_length: u8,
    pub protocol_address_length: u8,

    pub operation: Operation,

    pub sender_mac: MacAddress,
    pub sender_ip: Ipv4Addr,

    pub target_mac: MacAddress,
    pub target_ip: Ipv4Addr,
}

#[derive(Clone, Debug, Error, Serialize, Deserialize, PartialEq)]
pub enum ArpError {
    #[error("Bad hardware length")]
    BadHardwareLength,

    #[error("Unknown hardware type")]
    HardwareTypeUnknown,

    #[error("Unknown operation type")]
    OperationUnknown,
}

pub mod hardware_type;
pub mod operation;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::FrameType;
    use crate::parser::ProtocolParser;
    use crate::protocols::arp::operation::Operation;
    use crate::protocols::ethernet::Ethernet;
    use crate::wrapper::FrameHeader;

    #[test]
    fn test_arp_without_ethernet_padding() {
        let hex_actual = "00 1A 8C 10 AD 30 00 1E 68 51 4F A9 08 06 00 01 08 00 06 04 00 02 00 1E 68 51 4F A9 AC 10 FF 01 00 1A 8C 10 AD 30 AC 10 00 01".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 42,
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
            ether_type: EtherType::Arp,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_arp = match metadata.layers[1].clone() {
            ProtocolData::Arp(value) => value,
            _ => panic!(),
        };

        let expected_arp = Arp {
            hardware_type: HardwareType::Ethernet,
            protocol_type: EtherType::Ipv4,
            hardware_address_length: ethernet::mac::LENGTH_BYTES as u8,
            protocol_address_length: ip::address::V4_LENGTH_BYTES as u8,
            operation: Operation::Reply,
            sender_mac: MacAddress::try_from("00:1E:68:51:4F:A9").unwrap(),
            sender_ip: Ipv4Addr::new(172, 16, 255, 1),
            target_mac: MacAddress::try_from("00:1A:8C:10:AD:30").unwrap(),
            target_ip: Ipv4Addr::new(172, 16, 0, 1),
        };

        assert_eq!(actual_arp, expected_arp);
    }

    #[test]
    fn test_arp_with_ethernet_padding() {
        let hex_actual = "00 1E 68 51 4F A9 00 1A 8C 10 AD 30 08 06 00 01 08 00 06 04 00 01 00 1A 8C 10 AD 30 AC 10 00 01 00 00 00 00 00 00 AC 10 FF 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00".replace(" ", "");
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
            destination_mac: MacAddress::try_from("00:1E:68:51:4F:A9").unwrap(),
            source_mac: MacAddress::try_from("00:1A:8C:10:AD:30").unwrap(),
            ether_type: EtherType::Arp,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_arp = match metadata.layers[1].clone() {
            ProtocolData::Arp(value) => value,
            _ => panic!(),
        };

        let expected_arp = Arp {
            hardware_type: HardwareType::Ethernet,
            protocol_type: EtherType::Ipv4,
            hardware_address_length: ethernet::mac::LENGTH_BYTES as u8,
            protocol_address_length: ip::address::V4_LENGTH_BYTES as u8,
            operation: Operation::Request,
            sender_mac: MacAddress::try_from("00:1A:8C:10:AD:30").unwrap(),
            sender_ip: Ipv4Addr::new(172, 16, 0, 1),
            target_mac: MacAddress::try_from("00:00:00:00:00:00").unwrap(),
            target_ip: Ipv4Addr::new(172, 16, 255, 1),
        };

        assert_eq!(actual_arp, expected_arp);
    }
}
