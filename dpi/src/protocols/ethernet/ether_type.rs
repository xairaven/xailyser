use crate::parser::ParserError;
use crate::protocols::ethernet::EthernetError;
use nom::IResult;
use nom::Parser;
use nom::number::be_u16;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
}

impl TryFrom<&[u8; 2]> for EtherType {
    type Error = EthernetError;

    fn try_from(value: &[u8; 2]) -> Result<Self, Self::Error> {
        match value {
            [0x08, 0x06] => Ok(Self::Arp),
            [0x08, 0x08] => Ok(Self::ArpFrameRelay),
            [0x80, 0x35] => Ok(Self::ArpReverse),
            [0x08, 0x00] => Ok(Self::Ipv4),
            [0x86, 0xDD] => Ok(Self::Ipv6),
            [0x88, 0xCC] => Ok(Self::Lldp),
            [0x81, 0x00] => Ok(Self::Vlan),
            _ => Err(EthernetError::EtherTypeUnknown),
        }
    }
}

impl TryFrom<u16> for EtherType {
    type Error = EthernetError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0806 => Ok(Self::Arp),
            0x0808 => Ok(Self::ArpFrameRelay),
            0x8035 => Ok(Self::ArpReverse),
            0x0800 => Ok(Self::Ipv4),
            0x86DD => Ok(Self::Ipv6),
            0x88CC => Ok(Self::Lldp),
            0x8100 => Ok(Self::Vlan),
            _ => Err(EthernetError::EtherTypeUnknown),
        }
    }
}

pub fn parse(input: &[u8]) -> IResult<&[u8], EtherType> {
    let (input, ether_type) = be_u16().parse(input)?;
    let ether_type = EtherType::try_from(ether_type)
        .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;

    Ok((input, ether_type))
}
