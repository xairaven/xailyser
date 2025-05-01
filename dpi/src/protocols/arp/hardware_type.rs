use crate::parser::ParserError;
use crate::protocols::arp::ArpError;
use crate::protocols::ethernet;
use nom::IResult;
use nom::Parser;
use nom::number::be_u16;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

pub const LENGTH_BYTES: usize = 2;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u16)]
pub enum HardwareType {
    Ethernet = 1,
}

impl HardwareType {
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Ethernet => &[0x00, 0x01],
        }
    }

    pub fn validate_length(&self, length: usize) -> Result<(), ArpError> {
        match self {
            Self::Ethernet => {
                let is_validated = length == ethernet::mac::LENGTH_BYTES;
                if is_validated {
                    Ok(())
                } else {
                    Err(ArpError::BadHardwareLength)
                }
            },
        }
    }
}

impl TryFrom<&[u8; 2]> for HardwareType {
    type Error = ArpError;

    fn try_from(value: &[u8; 2]) -> Result<Self, Self::Error> {
        match value {
            [0x00, 0x01] => Ok(Self::Ethernet),
            _ => Err(ArpError::HardwareTypeUnknown),
        }
    }
}

pub fn parse(input: &[u8]) -> IResult<&[u8], HardwareType> {
    let (input, number) = be_u16().parse(input)?;

    let hardware_type = HardwareType::try_from(number)
        .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;

    Ok((input, hardware_type))
}
