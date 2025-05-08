use crate::parser::ParserError;
use crate::protocols::arp::ArpError;
use nom::IResult;
use nom::Parser;
use nom::number::be_u16;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub const LENGTH_BYTES: usize = 2;
#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u16)]
pub enum Operation {
    Request = 1,
    Reply = 2,
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
        match value {
            [0x00, 0x01] => Ok(Self::Request),
            [0x00, 0x02] => Ok(Self::Reply),
            _ => Err(ArpError::OperationUnknown),
        }
    }
}

pub fn parse(input: &[u8]) -> IResult<&[u8], Operation> {
    let (input, number) = be_u16().parse(input)?;

    let operation = Operation::try_from(number)
        .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;

    Ok((input, operation))
}
