use crate::parser::ParserError;
use nom::IResult;
use nom::Parser;
use nom::bytes::take;
use std::net::Ipv4Addr;

pub const V4_LENGTH_BYTES: usize = 4;

pub fn v4_parse(input: &[u8]) -> IResult<&[u8], Ipv4Addr> {
    let (input, address) = take(V4_LENGTH_BYTES).parse(input)?;

    let address = Ipv4Addr::from(
        <[u8; 4]>::try_from(address)
            .map_err(|_| ParserError::ErrorVerify.to_nom(input))?,
    );

    Ok((input, address))
}
