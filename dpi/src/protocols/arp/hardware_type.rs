use crate::error;
use crate::protocols::arp::ArpError;
use nom::IResult;
use nom::Parser;
use nom::number::be_u16;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const LENGTH_BYTES: usize = 2;

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
}

impl TryFrom<&[u8; 2]> for HardwareType {
    type Error = ArpError;

    fn try_from(value: &[u8; 2]) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|hardware_type| hardware_type.bytes() == value)
            .ok_or(ArpError::HardwareTypeUnknown)
    }
}

impl TryFrom<u16> for HardwareType {
    type Error = ArpError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(Self::Ethernet),
            _ => Err(ArpError::HardwareTypeUnknown),
        }
    }
}

pub fn parse(input: &[u8]) -> IResult<&[u8], HardwareType> {
    let (input, number) = be_u16().parse(input)?;

    let hardware_type =
        HardwareType::try_from(number).map_err(|_| error::nom_failure_verify(input))?;

    Ok((input, hardware_type))
}
