pub fn nom_error_verify(input: &[u8]) -> nom::Err<nom::error::Error<&[u8]>> {
    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
}

pub fn nom_failure_verify(input: &[u8]) -> nom::Err<nom::error::Error<&[u8]>> {
    nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
}
