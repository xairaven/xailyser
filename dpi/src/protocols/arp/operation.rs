use crate::utils;
use crate::protocols::arp::ArpError;
use nom::IResult;
use nom::Parser;
use nom::number::be_u16;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const LENGTH_BYTES: usize = 2;
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
}

impl TryFrom<&[u8; 2]> for Operation {
    type Error = ArpError;

    fn try_from(value: &[u8; 2]) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|operation| operation.bytes() == value)
            .ok_or(ArpError::OperationUnknown)
    }
}

impl TryFrom<u16> for Operation {
    type Error = ArpError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(Self::Request),
            0x0002 => Ok(Self::Reply),
            _ => Err(ArpError::HardwareTypeUnknown),
        }
    }
}

pub fn parse(input: &[u8]) -> IResult<&[u8], Operation> {
    let (input, number) = be_u16().parse(input)?;

    let operation =
        Operation::try_from(number).map_err(|_| utils::nom_failure_verify(input))?;

    Ok((input, operation))
}
