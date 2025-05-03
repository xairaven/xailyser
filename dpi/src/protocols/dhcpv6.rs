use crate::parser::ParserError;
use crate::protocols::{ProtocolData, ip};
use nom::IResult;
use nom::bytes::take;
use nom::number::{be_u8, be_u16, be_u32};
use nom::{Finish, Parser};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::net::Ipv6Addr;

// DHCPv6 Protocol
// RFC 8415: https://datatracker.ietf.org/doc/html/rfc8415

pub const TRANSACTION_ID_LENGTH_BYTES: usize = 3;
pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    // Message Type, 1 byte.
    let (rest, message_type) = be_u8().parse(bytes)?;
    let message_type = MessageType::try_from(message_type)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // Transaction ID. 3 bytes
    let (rest, transaction_id) = take(TRANSACTION_ID_LENGTH_BYTES).parse(rest)?;
    let transaction_id = <&[u8; TRANSACTION_ID_LENGTH_BYTES]>::try_from(transaction_id)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;
    let transaction_id = ((transaction_id[0] as u32) << 16)
        | ((transaction_id[1] as u32) << 8)
        | (transaction_id[2] as u32);

    let mut options: Vec<OptionData> = Vec::new();
    let mut rest_buffer = rest;
    while !rest_buffer.is_empty() {
        let (rest, option) = Options::parse(rest_buffer)?;
        options.push(option);
        rest_buffer = rest;
    }

    let protocol = DHCPv6 {
        message_type,
        transaction_id,
        options,
    };

    let empty: &[u8] = &[];
    Finish::finish(Ok((empty, ProtocolData::DHCPv6(protocol))))
}

pub fn is_protocol_default(port_source: u16, port_destination: u16) -> bool {
    const SERVER_PORT: u16 = 547;
    const CLIENT_PORT: u16 = 546;

    port_source == SERVER_PORT
        || port_source == CLIENT_PORT
        || port_destination == SERVER_PORT
        || port_destination == CLIENT_PORT
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DHCPv6 {
    pub message_type: MessageType,
    pub transaction_id: u32,
    pub options: Vec<OptionData>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u16)]
pub enum Options {
    ClientIdentifier = 1,
    ServerIdentifier = 2,
    IANA = 3,
    OptionRequest = 6,
    ElapsedTime = 8,
    VendorClass = 16,
    VendorSpecific = 17,

    // RFC for 23 & 24: https://datatracker.ietf.org/doc/html/rfc3646
    DnsRecursiveNameServer = 23,
    DomainSearchList = 24,

    ClientFQDN = 39,
}

impl Options {
    pub fn parse(input: &[u8]) -> IResult<&[u8], OptionData> {
        let (rest, code) = be_u16().parse(input)?;
        let option_variant =
            Options::try_from(code).map_err(|_| ParserError::ErrorVerify.to_nom(rest))?;

        let (rest, length) = be_u16().parse(rest)?;
        let (rest, content) = take(length).parse(rest)?;

        let data: OptionData = match option_variant {
            Options::ClientIdentifier => OptionData::ClientIdentifier(content.to_vec()),

            Options::ServerIdentifier => OptionData::ServerIdentifier(content.to_vec()),

            Options::IANA => {
                let (rest, id) = be_u32().parse(content)?;
                let (rest, t1) = be_u32().parse(rest)?;
                let (rest, t2) = be_u32().parse(rest)?;
                let options = rest;

                OptionData::IANA {
                    id,
                    client_server_contact_seconds: t1,
                    client_any_server_contact_seconds: t2,
                    options: options.to_vec(),
                }
            },

            Options::OptionRequest => {
                let mut requested_options: Vec<Options> = Vec::new();
                let mut rest_buffer = content;
                for _ in 0..length / 2 {
                    let (rest, option) = be_u16().parse(rest_buffer)?;
                    let option = Options::try_from(option)
                        .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
                    requested_options.push(option);
                    rest_buffer = rest;
                }

                OptionData::OptionRequest(requested_options)
            },

            Options::ElapsedTime => {
                let (_, time) = be_u16().parse(content)?;
                OptionData::ElapsedTime(time)
            },

            Options::VendorClass | Options::VendorSpecific => {
                let (rest, enterprise_number) = be_u32().parse(content)?;
                let vendor_class_data = rest.to_vec();
                OptionData::VendorData {
                    enterprise_number,
                    vendor_class_data,
                }
            },

            Options::DnsRecursiveNameServer => {
                if length % 16 != 0 {
                    return Err(ParserError::ErrorVerify.to_nom(rest));
                }

                let mut addresses: Vec<Ipv6Addr> = Vec::new();
                let mut rest_buffer = content;
                for _ in 0..length / 16 {
                    let (rest, address) =
                        take(ip::address::V6_LENGTH_BYTES).parse(rest_buffer)?;
                    let (_, address) = ip::address::v6_parse(address)?;

                    addresses.push(address);
                    rest_buffer = rest;
                }

                OptionData::DnsRecursiveServers(addresses)
            },

            Options::DomainSearchList => OptionData::DomainSearchList(content.to_vec()),

            Options::ClientFQDN => {
                let (rest, flags) = be_u8().parse(content)?;
                let domain_name = String::from_utf8(rest.to_vec())
                    .map_err(|_| ParserError::ErrorVerify.to_nom(rest))?
                    .trim_matches(['\0', '\u{3}', '\u{6}'])
                    .split(['\u{3}', '\u{6}'])
                    .collect::<Vec<&str>>()
                    .join(".")
                    .to_string();

                OptionData::ClientFQDN { flags, domain_name }
            },
        };

        Ok((rest, data))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OptionData {
    ClientIdentifier(Vec<u8>),
    ServerIdentifier(Vec<u8>),
    IANA {
        id: u32,
        client_server_contact_seconds: u32,
        client_any_server_contact_seconds: u32,
        options: Vec<u8>,
    },
    ElapsedTime(u16),
    OptionRequest(Vec<Options>),
    VendorData {
        enterprise_number: u32,
        vendor_class_data: Vec<u8>,
    },
    DnsRecursiveServers(Vec<Ipv6Addr>),
    DomainSearchList(Vec<u8>),
    ClientFQDN {
        flags: u8,
        domain_name: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum MessageType {
    Solicit = 1,
    Advertise = 2,
    Request = 3,
    Confirm = 4,
    Renew = 5,
    Rebind = 6,
    Reply = 7,
    Release = 8,
    Decline = 9,
    Reconfigure = 10,
    InformationRequest = 11,
    RelayForward = 12,
    RelayRepl = 13,
    LeaseQuery = 14,
    LeaseQueryReply = 15,
    LeaseQueryDone = 16,
    LeaseQueryData = 17,
    ReconfigureRequest = 18,
    ReconfigureReply = 19,
    DHCPv4Query = 20,
    DHCPv4Response = 21,
    ActiveLeaseQuery = 22,
    StartTLS = 23,
}

#[cfg(test)]
mod tests {
    use crate::frame::FrameType;
    use crate::parser::ProtocolParser;
    use crate::protocols::ProtocolData;
    use crate::protocols::dhcpv6::OptionData::{ClientFQDN, VendorData};
    use crate::protocols::dhcpv6::{DHCPv6, MessageType, OptionData, Options};
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv6::IPv6;
    use crate::protocols::udp::UDP;
    use crate::wrapper::FrameHeader;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

    #[test]
    fn test_dhcpv6() {
        let hex_actual = "33 33 00 01 00 02 7C E9 D3 7C D3 9B 86 DD 60 00 00 00 00 6A 11 01 FE 80 00 00 00 00 00 00 B5 6E 75 8F D6 E2 B7 9E FF 02 00 00 00 00 00 00 00 00 00 00 00 01 00 02 02 22 02 23 00 6A E6 A9 01 76 31 13 00 08 00 02 01 2C 00 01 00 0E 00 01 00 01 16 2B 8C FE 00 21 70 63 3A E9 00 03 00 0C 17 7C E9 D3 00 00 00 00 00 00 00 00 00 27 00 14 00 06 4A 44 54 31 33 33 06 6A 61 61 6C 61 6D 03 6E 65 74 00 00 10 00 0E 00 00 01 37 00 08 4D 53 46 54 20 35 2E 30 00 06 00 08 00 18 00 17 00 11 00 27".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 160,
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
                _ => panic!(),
            },
            None => panic!(),
        };

        let actual_ethernet = match metadata.layers[0].clone() {
            ProtocolData::Ethernet(value) => value,
            _ => panic!(),
        };

        let expected_ethernet = Ethernet {
            destination_mac: MacAddress::try_from("33:33:00:01:00:02").unwrap(),
            source_mac: MacAddress::try_from("7c:e9:d3:7c:d3:9b").unwrap(),
            ether_type: EtherType::Ipv6,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_ipv6 = match metadata.layers[1].clone() {
            ProtocolData::IPv6(value) => value,
            _ => panic!(),
        };

        let expected_ipv6 = IPv6 {
            version: 6,
            traffic_class: 0,
            flow_label: 0,
            payload_length: 106,
            next_header: IpNextLevelProtocol::UDP,
            hop_limit: 1,
            address_source: Ipv6Addr::from_str("fe80::b56e:758f:d6e2:b79e").unwrap(),
            address_destination: Ipv6Addr::from_str("ff02::1:2").unwrap(),
        };

        assert_eq!(actual_ipv6, expected_ipv6);

        let actual_udp = match metadata.layers[2].clone() {
            ProtocolData::UDP(value) => value,
            _ => panic!(),
        };

        let expected_udp = UDP {
            port_source: 546,
            port_destination: 547,
            length: 106,
            checksum: 0xe6a9,
        };

        assert_eq!(actual_udp, expected_udp);

        let actual_dhcp = match metadata.layers[3].clone() {
            ProtocolData::DHCPv6(value) => value,
            _ => panic!(),
        };

        let expected_dhcp = DHCPv6 {
            message_type: MessageType::Solicit,
            transaction_id: 0x763113,
            options: vec![
                OptionData::ElapsedTime(300),
                OptionData::ClientIdentifier(vec![
                    0x00, 0x01, 0x00, 0x01, 0x16, 0x2b, 0x8c, 0xfe, 0x00, 0x21, 0x70,
                    0x63, 0x3a, 0xe9,
                ]),
                OptionData::IANA {
                    id: 0x177ce9d3,
                    client_server_contact_seconds: 0,
                    client_any_server_contact_seconds: 0,
                    options: vec![],
                },
                ClientFQDN {
                    flags: 0,
                    domain_name: "JDT133.jaalam.net".to_string(),
                },
                VendorData {
                    enterprise_number: 311,
                    vendor_class_data: vec![
                        0x00, 0x08, 0x4D, 0x53, 0x46, 0x54, 0x20, 0x35, 0x2E, 0x30,
                    ],
                },
                OptionData::OptionRequest(vec![
                    Options::DomainSearchList,
                    Options::DnsRecursiveNameServer,
                    Options::VendorSpecific,
                    Options::ClientFQDN,
                ]),
            ],
        };

        assert_eq!(actual_dhcp, expected_dhcp);
    }
}
