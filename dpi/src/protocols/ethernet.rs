use crate::ParseResult;
use crate::frame::FrameMetadata;
use crate::protocols::{ProtocolData, ProtocolId};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// ETHERNET II.
pub const FRAME_LENGTH: usize = 14;
pub const FCS_LENGTH: usize = 4;
pub const MAC_LENGTH: usize = 6;
pub fn parse<'a>(bytes: &'a [u8], metadata: &mut FrameMetadata) -> ParseResult<'a> {
    if bytes.len() < FRAME_LENGTH {
        return ParseResult::Failed;
    }

    let dst_mac = match <[u8; MAC_LENGTH]>::try_from(&bytes[0..6]) {
        Ok(value) => MacAddress::from_bytes(value),
        Err(_) => return ParseResult::Failed,
    };
    let src_mac = match <[u8; MAC_LENGTH]>::try_from(&bytes[6..12]) {
        Ok(value) => MacAddress::from_bytes(value),
        Err(_) => return ParseResult::Failed,
    };
    let ether_type = match EtherType::from_bytes(&[bytes[12], bytes[13]]) {
        Some(value) => value,
        None => return ParseResult::Failed,
    };

    let ethernet = Ethernet {
        id: ProtocolId::Ethernet,
        destination_mac: dst_mac,
        source_mac: src_mac,
        ether_type,
    };
    metadata.layers.push(ProtocolData::Ethernet(ethernet));

    if bytes.len() > FRAME_LENGTH {
        ParseResult::SuccessIncomplete(&bytes[14..])
    } else {
        ParseResult::Failed
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ethernet {
    pub id: ProtocolId,
    pub destination_mac: MacAddress,
    pub source_mac: MacAddress,
    pub ether_type: EtherType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MacAddress {
    bytes: [u8; MAC_LENGTH],
    string: String,
}

impl MacAddress {
    pub fn from_bytes(bytes: [u8; MAC_LENGTH]) -> Self {
        let string = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
        );
        Self { bytes, string }
    }

    pub fn from_string(raw: &str) -> Result<Self, hex::FromHexError> {
        let raw = raw.to_string();
        let s = raw.replace(":", "").replace(".", "").replace("-", "");
        let bytes = hex::decode(&s)?;
        Ok(Self {
            bytes: <[u8; MAC_LENGTH]>::try_from(bytes)
                .map_err(|_| hex::FromHexError::InvalidStringLength)?,
            string: raw,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumIter, PartialEq)]
pub enum EtherType {
    Arp,
    ArpFrameRelay,
    ArpReverse,
    Ipv4,
    Ipv6,
    Lldp,
    Vlan,
}

impl EtherType {
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Arp => &[0x08, 0x06],
            Self::ArpFrameRelay => &[0x08, 0x08],
            Self::ArpReverse => &[0x80, 0x35],
            Self::Ipv4 => &[0x08, 0x00],
            Self::Ipv6 => &[0x86, 0xDD],
            Self::Lldp => &[0x88, 0xCC],
            Self::Vlan => &[0x81, 0x00],
        }
    }

    pub fn from_bytes(bytes: &[u8; 2]) -> Option<Self> {
        Self::iter().find(|ether_type| ether_type.bytes() == bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrapper::FrameHeader;

    #[test]
    fn test_ethernet() {
        let ethernet_header = hex::decode("04E8B918551084D81B6EC14A0800").unwrap();
        let mut metadata = FrameMetadata {
            header: FrameHeader {
                tv_sec: 0,
                tv_usec: 0,
                caplen: 0,
                len: 0,
            },
            layers: vec![],
        };
        let _ = parse(&ethernet_header, &mut metadata);
        let actual = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected = Ethernet {
            id: ProtocolId::Ethernet,
            destination_mac: MacAddress::from_string("04:E8:B9:18:55:10").unwrap(),
            source_mac: MacAddress::from_string("84:D8:1B:6E:C1:4A").unwrap(),
            ether_type: EtherType::Ipv4,
        };

        assert_eq!(actual, expected);
    }
}
