use crate::ParseResult;
use crate::frame::FrameMetadata;
use crate::protocols::ethernet::ether_type::EtherType;
use crate::protocols::ethernet::mac::MacAddress;
use crate::protocols::{ProtocolData, ProtocolId};
use serde::{Deserialize, Serialize};

// ETHERNET II.
pub const FRAME_LENGTH: usize = 14;
pub const FCS_LENGTH: usize = 4;

pub fn parse<'a>(bytes: &'a [u8], _: &FrameMetadata) -> ParseResult<'a> {
    if bytes.len() < FRAME_LENGTH {
        return ParseResult::Failed;
    }

    let dst_mac = match <[u8; mac::LENGTH_BYTES]>::try_from(&bytes[0..6]) {
        Ok(value) => MacAddress::from(value),
        Err(_) => return ParseResult::Failed,
    };
    let src_mac = match <[u8; mac::LENGTH_BYTES]>::try_from(&bytes[6..12]) {
        Ok(value) => MacAddress::from(value),
        Err(_) => return ParseResult::Failed,
    };
    let ether_type = match EtherType::try_from(&[bytes[12], bytes[13]]) {
        Ok(value) => value,
        Err(_) => return ParseResult::Failed,
    };

    let ethernet = Ethernet {
        id: ProtocolId::Ethernet,
        destination_mac: dst_mac,
        source_mac: src_mac,
        ether_type,
    };

    if bytes.len() > FRAME_LENGTH {
        ParseResult::SuccessIncomplete(ProtocolData::Ethernet(ethernet), &bytes[14..])
    } else {
        ParseResult::Failed
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ethernet {
    pub id: ProtocolId,
    pub destination_mac: MacAddress,
    pub source_mac: MacAddress,
    pub ether_type: EtherType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtocolParser;
    use crate::frame::FrameType;
    use crate::wrapper::FrameHeader;

    #[test]
    fn test_ethernet() {
        let hex_actual = "40 61 86 9A F1 F5 00 1A 8C 15 F9 80 08 00 45 00 00 28 59 B0 40 00 38 06 86 FC CE 6C CF 8B C0 A8 03 83 00 50 DA 98 A4 28 53 A0 9A 18 FA A4 50 10 00 D8 E5 69 00 00 00 00 00 00 00 00".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 60,
            len: 0,
        };

        let parser = ProtocolParser::new(&pcap::Linktype(1), false);
        let packet = pcap::Packet {
            header: &pcap::PacketHeader::from(&header),
            data: &frame,
        };
        let result = parser.process(packet);
        let metadata = match result {
            Some(value) => match value {
                FrameType::Metadata(value) => value,
                FrameType::Raw(_) => panic!(),
            },
            None => panic!(),
        };

        let actual_ethernet = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            id: ProtocolId::Ethernet,
            destination_mac: MacAddress::try_from("40:61:86:9A:F1:F5").unwrap(),
            source_mac: MacAddress::try_from("00:1A:8C:15:F9:80").unwrap(),
            ether_type: EtherType::Ipv4,
        };

        assert_eq!(actual_ethernet, expected_ethernet);
    }
}

pub mod ether_type;
pub mod mac;
