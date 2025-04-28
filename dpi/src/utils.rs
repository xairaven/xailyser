use nom::{IResult, bits, sequence};

pub fn nom_error_verify(input: &[u8]) -> nom::Err<nom::error::Error<&[u8]>> {
    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
}

pub fn nom_failure_verify(input: &[u8]) -> nom::Err<nom::error::Error<&[u8]>> {
    nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
}

pub(crate) fn pair_from_byte(input: &[u8], length: usize) -> IResult<&[u8], (u8, u8)> {
    bits::bits::<_, _, nom::error::Error<_>, _, _>(sequence::pair(
        bits::complete::take(length),
        bits::complete::take(8 - length),
    ))(input)
}
