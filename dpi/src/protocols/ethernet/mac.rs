use crate::protocols::ethernet::EthernetError;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

pub const LENGTH_BYTES: usize = 6;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MacAddress(pub [u8; LENGTH_BYTES]);

impl From<[u8; LENGTH_BYTES]> for MacAddress {
    fn from(value: [u8; LENGTH_BYTES]) -> Self {
        Self(value)
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
