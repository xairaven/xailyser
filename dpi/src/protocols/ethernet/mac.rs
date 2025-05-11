use crate::parser::ParserError;
use crate::protocols::ethernet::EthernetError;
use nom::IResult;
use nom::bytes::complete::take;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

pub const LENGTH_BYTES: usize = 6;
pub const BROADCAST_MAC: [u8; LENGTH_BYTES] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct MacAddress(pub [u8; LENGTH_BYTES]);

impl MacAddress {
    pub fn is_broadcast(&self) -> bool {
        self.0.eq(&BROADCAST_MAC)
    }

    pub fn is_multicast(&self) -> bool {
        if self.is_broadcast() {
            return false;
        }

        self.0[0] & 0b00000001 == 1
    }

    pub fn to_bit_string(&self) -> String {
        self.0.map(|num| format!("{:08b}", num)).join("")
    }
}

impl From<[u8; LENGTH_BYTES]> for MacAddress {
    fn from(value: [u8; LENGTH_BYTES]) -> Self {
        Self(value)
    }
}

impl TryFrom<&[u8]> for MacAddress {
    type Error = EthernetError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let bytes = <[u8; LENGTH_BYTES]>::try_from(value)
            .map_err(|_| EthernetError::MacInvalidBytesLength)?;

        Ok(MacAddress(bytes))
    }
}

impl TryFrom<&str> for MacAddress {
    type Error = EthernetError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let s = value.replace(":", "").replace(".", "").replace("-", "");
        let bytes = hex::decode(&s).map_err(|_| EthernetError::MacFailedHexDecode)?;
        let bytes = <[u8; LENGTH_BYTES]>::try_from(bytes)
            .map_err(|_| EthernetError::MacInvalidStringLength)?;

        Ok(Self(bytes))
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        );

        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vendor {
    pub short: String,
    pub full: String,
}

pub fn parse(input: &[u8]) -> IResult<&[u8], MacAddress> {
    let (input, mac_bytes) = take(LENGTH_BYTES)(input)?;
    let mac = match MacAddress::try_from(mac_bytes) {
        Ok(mac) => mac,
        Err(_) => return Err(ParserError::ErrorVerify.to_nom(input)),
    };

    Ok((input, mac))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_broadcast() {
        let mac = MacAddress([0xFF; LENGTH_BYTES]);
        assert_eq!(mac.is_broadcast(), true);
    }

    #[test]
    fn test_is_not_broadcast() {
        let mac = MacAddress::try_from("00:1A:2B:3C:4D:5E").unwrap();
        assert_eq!(mac.is_broadcast(), false);
    }

    #[test]
    fn test_is_multicast() {
        let mac = MacAddress::try_from("01:00:5e:02:02:04").unwrap();
        assert_eq!(mac.is_multicast(), true);
    }

    #[test]
    fn test_is_multicast_ipv6() {
        let mac = MacAddress::try_from("33:33:00:00:00:02").unwrap();
        assert_eq!(mac.is_multicast(), true);
    }

    #[test]
    fn test_is_not_multicast() {
        let mac = MacAddress::try_from("00:1A:2B:3C:4D:5E").unwrap();
        assert_eq!(mac.is_multicast(), false);
    }
}
