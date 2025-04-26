use crate::ParseResult;
use crate::frame::FrameMetadata;
use crate::protocols::ethernet::{EtherType, MAC_LENGTH, MacAddress};
use crate::protocols::{ProtocolData, ProtocolId, ipv4};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// ARP Protocol
// RFC 826: https://datatracker.ietf.org/doc/html/rfc826
pub const PACKET_LENGTH: usize = 28;

pub const HARDWARE_TYPE_LENGTH: usize = 2;
pub const PROTOCOL_TYPE_LENGTH: usize = 2;
pub const HARDWARE_ADDRESS_LENGTH: usize = 1;
pub const PROTOCOL_ADDRESS_LENGTH: usize = 1;
pub const OPERATION_LENGTH: usize = 2;

pub fn parse<'a>(bytes: &'a [u8], metadata: &mut FrameMetadata) -> ParseResult<'a> {
    if bytes.len() < PACKET_LENGTH {
        return ParseResult::Failed;
    };

    // Cutting Ethernet padding & FCS
    let bytes = if bytes.len() > PACKET_LENGTH {
        &bytes[..PACKET_LENGTH]
    } else {
        bytes
    };

    // Parsing HTYPE
    let data_link_layer = match metadata.layers.first() {
        Some(value) => value,
        None => {
            return ParseResult::Failed;
        },
    };
    let ethernet = match data_link_layer {
        ProtocolData::Ethernet(value) => value,
        _ => {
            return ParseResult::Failed;
        },
    };
    if ethernet.ether_type.ne(&EtherType::Arp) {
        return ParseResult::Failed;
    }
    let hardware_type = match <[u8; HARDWARE_TYPE_LENGTH]>::try_from(&bytes[0..2]) {
        Ok(value) => match HardwareType::from_bytes(&value) {
            Some(value) => value,
            None => return ParseResult::Failed,
        },
        Err(_) => return ParseResult::Failed,
    };

    // Parsing PTYPE
    let protocol_type = match EtherType::from_bytes(&[bytes[2], bytes[3]]) {
        Some(value) => value,
        None => return ParseResult::Failed,
    };
    if protocol_type != EtherType::Ipv4 {
        return ParseResult::Failed;
    }

    // Parsing HLEN
    let hardware_address_length = if bytes[4] == MAC_LENGTH as u8 {
        MAC_LENGTH as u8
    } else {
        return ParseResult::Failed;
    };

    // Parsing PLEN
    let protocol_address_length = if bytes[5] == ipv4::LOGICAL_ADDRESS_LENGTH as u8 {
        ipv4::LOGICAL_ADDRESS_LENGTH as u8
    } else {
        return ParseResult::Failed;
    };

    // Parsing OPERATION
    let operation = match <[u8; OPERATION_LENGTH]>::try_from(&bytes[6..8]) {
        Ok(value) => match Operation::from_bytes(&value) {
            Some(value) => value,
            None => return ParseResult::Failed,
        },
        Err(_) => return ParseResult::Failed,
    };

    // Parsing SENDER_HARDWARE_ADDRESS
    let sender_hardware_address = match <[u8; MAC_LENGTH]>::try_from(&bytes[8..14]) {
        Ok(value) => MacAddress::from_bytes(value),
        Err(_) => return ParseResult::Failed,
    };

    // Parsing SENDER_PROTOCOL_ADDRESS
    let sender_protocol_address =
        match <[u8; ipv4::LOGICAL_ADDRESS_LENGTH]>::try_from(&bytes[14..18]) {
            Ok(value) => Ipv4Addr::from(value),
            Err(_) => return ParseResult::Failed,
        };

    // Parsing TARGET_HARDWARE_ADDRESS
    let target_hardware_address = match <[u8; MAC_LENGTH]>::try_from(&bytes[18..24]) {
        Ok(value) => MacAddress::from_bytes(value),
        Err(_) => return ParseResult::Failed,
    };

    // Parsing TARGET_PROTOCOL_ADDRESS
    let target_protocol_address =
        match <[u8; ipv4::LOGICAL_ADDRESS_LENGTH]>::try_from(&bytes[24..28]) {
            Ok(value) => Ipv4Addr::from(value),
            Err(_) => return ParseResult::Failed,
        };

    let arp = Arp {
        id: ProtocolId::Arp,
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
    metadata.layers.push(ProtocolData::Arp(arp));

    ParseResult::Success
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Arp {
    pub id: ProtocolId,

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

#[derive(Clone, Debug, Serialize, Deserialize, EnumIter, PartialEq)]
pub enum HardwareType {
    Ethernet,
}

impl HardwareType {
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Ethernet => &[0x00, 0x01],
        }
    }

    pub fn from_bytes(bytes: &[u8; 2]) -> Option<Self> {
        Self::iter().find(|hardware_type| hardware_type.bytes() == bytes)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumIter, PartialEq)]
pub enum Operation {
    Request,
    Reply,
}

impl Operation {
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Request => &[0x00, 0x01],
            Self::Reply => &[0x00, 0x02],
        }
    }

    pub fn from_bytes(bytes: &[u8; 2]) -> Option<Self> {
        Self::iter().find(|operation| operation.bytes() == bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtocolParser;
    use crate::frame::FrameType;
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

        let parser = ProtocolParser::new(false);
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
            id: ProtocolId::Ethernet,
            destination_mac: MacAddress::from_string("00:1A:8C:10:AD:30").unwrap(),
            source_mac: MacAddress::from_string("00:1E:68:51:4F:A9").unwrap(),
            ether_type: EtherType::Arp,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_arp = match metadata.layers[1].clone() {
            ProtocolData::Arp(value) => value,
            _ => panic!(),
        };

        let expected_arp = Arp {
            id: ProtocolId::Arp,
            hardware_type: HardwareType::Ethernet,
            protocol_type: EtherType::Ipv4,
            hardware_address_length: MAC_LENGTH as u8,
            protocol_address_length: ipv4::LOGICAL_ADDRESS_LENGTH as u8,
            operation: Operation::Reply,
            sender_mac: MacAddress::from_string("00:1E:68:51:4F:A9").unwrap(),
            sender_ip: Ipv4Addr::new(172, 16, 255, 1),
            target_mac: MacAddress::from_string("00:1A:8C:10:AD:30").unwrap(),
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

        let parser = ProtocolParser::new(false);
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
            id: ProtocolId::Ethernet,
            destination_mac: MacAddress::from_string("00:1E:68:51:4F:A9").unwrap(),
            source_mac: MacAddress::from_string("00:1A:8C:10:AD:30").unwrap(),
            ether_type: EtherType::Arp,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_arp = match metadata.layers[1].clone() {
            ProtocolData::Arp(value) => value,
            _ => panic!(),
        };

        let expected_arp = Arp {
            id: ProtocolId::Arp,
            hardware_type: HardwareType::Ethernet,
            protocol_type: EtherType::Ipv4,
            hardware_address_length: MAC_LENGTH as u8,
            protocol_address_length: ipv4::LOGICAL_ADDRESS_LENGTH as u8,
            operation: Operation::Request,
            sender_mac: MacAddress::from_string("00:1A:8C:10:AD:30").unwrap(),
            sender_ip: Ipv4Addr::new(172, 16, 0, 1),
            target_mac: MacAddress::from_string("00:00:00:00:00:00").unwrap(),
            target_ip: Ipv4Addr::new(172, 16, 255, 1),
        };

        assert_eq!(actual_arp, expected_arp);
    }
}
