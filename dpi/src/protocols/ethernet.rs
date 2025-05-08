use crate::dto::frame::FrameMetadata;
use crate::parser::ParserError;
use crate::protocols::ethernet::ether_type::EtherType;
use crate::protocols::ethernet::mac::MacAddress;
use crate::protocols::{ProtocolData, ProtocolId};
use nom::IResult;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ETHERNET II.
pub const FRAME_LENGTH: usize = 14;
pub const FCS_LENGTH: usize = 4;

pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    if bytes.len() <= FRAME_LENGTH {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    let (rest, destination_mac) = mac::parse(bytes)?;
    let (rest, source_mac) = mac::parse(rest)?;
    let (rest, ether_type) = ether_type::parse(rest)?;

    let layer = Ethernet {
        destination_mac,
        source_mac,
        ether_type,
    };

    Ok((rest, ProtocolData::Ethernet(layer)))
}

pub fn best_children(metadata: &FrameMetadata) -> Option<ProtocolId> {
    // Checking ethernet ether type
    let ethernet = match metadata.layers.first() {
        Some(ProtocolData::Ethernet(value)) => value,
        _ => return None,
    };
    match ethernet.ether_type {
        EtherType::Arp => Some(ProtocolId::Arp),
        EtherType::Ipv4 => Some(ProtocolId::IPv4),
        EtherType::Ipv6 => Some(ProtocolId::IPv6),
        _ => None,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ethernet {
    pub destination_mac: MacAddress,
    pub source_mac: MacAddress,
    pub ether_type: EtherType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EthernetDto {
    pub destination_mac: MacAddress,
    pub source_mac: MacAddress,
}

impl From<Ethernet> for EthernetDto {
    fn from(value: Ethernet) -> Self {
        Self {
            destination_mac: value.destination_mac,
            source_mac: value.source_mac,
        }
    }
}

#[derive(Clone, Debug, Error, Serialize, Deserialize, PartialEq)]
pub enum EthernetError {
    #[error("Unknown EtherType")]
    EtherTypeUnknown,

    #[error("Invalid hex characters found while parsing Mac address")]
    MacFailedHexDecode,

    #[error("Invalid bytes length for Mac address")]
    MacInvalidBytesLength,

    #[error("Invalid string length for Mac address")]
    MacInvalidStringLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethernet() {
        let hex_actual = "40 61 86 9A F1 F5 00 1A 8C 15 F9 80 08 00 45 00 00 28 59 B0 40 00 38 06 86 FC CE 6C CF 8B C0 A8 03 83 00 50 DA 98 A4 28 53 A0 9A 18 FA A4 50 10 00 D8 E5 69 00 00 00 00 00 00 00 00".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();

        let layer = parse(&frame);
        let actual_ethernet = match layer {
            Ok((_, ProtocolData::Ethernet(layer))) => layer,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            destination_mac: MacAddress::try_from("40:61:86:9A:F1:F5").unwrap(),
            source_mac: MacAddress::try_from("00:1A:8C:15:F9:80").unwrap(),
            ether_type: EtherType::Ipv4,
        };

        assert_eq!(actual_ethernet, expected_ethernet);
    }
}

pub mod ether_type;
pub mod mac;
