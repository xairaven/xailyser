use crate::parser::ParserError;
use crate::protocols::arp::hardware_type::HardwareType;
use crate::protocols::ethernet::mac::MacAddress;
use crate::protocols::{ProtocolData, ethernet, ip};
use nom::bytes::take;
use nom::number::{be_u8, be_u16, be_u32};
use nom::{Finish, IResult, Parser, bits};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

// DHCPv4 Protocol
// RFC 2131: https://datatracker.ietf.org/doc/html/rfc2131

pub const BROADCAST_FLAG_LENGTH_BITS: usize = 1;
pub const HARDWARE_ADDRESS_WITH_PADDING_LENGTH_BYTES: usize = 16;
pub const SERVER_NAME_LENGTH_BYTES: usize = 64;
pub const FILE_NAME_LENGTH_BYTES: usize = 128;
pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    // Operation, 1 byte.
    let (rest, op) = be_u8().parse(bytes)?;
    let op = OperationType::try_from(op)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // Hardware Type, 1 byte
    let (rest, htype) = be_u8().parse(rest)?;
    let htype = HardwareType::try_from(htype as u16)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // Hardware Address Length, 1 byte
    let (rest, hlen) = be_u8().parse(rest)?;
    htype
        .validate_length(hlen as usize)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // Hops, 1 byte
    let (rest, hops) = be_u8().parse(rest)?;

    // xId, 4 bytes
    let (rest, xid) = be_u32().parse(rest)?;

    // Secs, 2 bytes
    let (rest, secs) = be_u16().parse(rest)?;

    // Flags, 2 bytes
    let (rest, (broadcast_flag, _)): (&[u8], (u8, u16)) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>((
            bits::complete::take(BROADCAST_FLAG_LENGTH_BITS),
            bits::complete::take(16 - BROADCAST_FLAG_LENGTH_BITS),
        ))(rest)?;

    // Old Client IP, 4 bytes
    let (rest, ciaddr) = ip::address::v4_parse(rest)?;

    // New Client IP, 4 bytes
    let (rest, yiaddr) = ip::address::v4_parse(rest)?;

    // Server IP Address, 4 bytes
    let (rest, siaddr) = ip::address::v4_parse(rest)?;

    // Relay agent IP Address, 4 bytes
    let (rest, giaddr) = ip::address::v4_parse(rest)?;

    // Hardware address, 16 bytes
    let (rest, chaddr) = take(HARDWARE_ADDRESS_WITH_PADDING_LENGTH_BYTES).parse(rest)?;
    let (_, chaddr) = ethernet::mac::parse(chaddr)?;

    // Optional server name string, 64 bytes
    let (rest, sname) = take(SERVER_NAME_LENGTH_BYTES).parse(rest)?;
    let sname_vec = sname.to_vec();
    let sname = if sname_vec.iter().all(|&b| b == 0) {
        None
    } else {
        String::from_utf8(sname_vec).ok()
    };

    // File name, 128 bytes
    let (rest, file) = take(FILE_NAME_LENGTH_BYTES).parse(rest)?;
    let file_vec = file.to_vec();
    let file = if file_vec.iter().all(|&b| b == 0) {
        None
    } else {
        String::from_utf8(file_vec).ok()
    };

    // Options, variable length
    let mut options: Vec<OptionData> = Vec::new();
    if !rest.is_empty() {
        // Magic cookies
        const MAGIC_NUMBERS: [u8; 4] = [0x63, 0x82, 0x53, 0x63];
        let (rest, magic_octets) = take(MAGIC_NUMBERS.len()).parse(rest)?;
        let magic_octets = <[u8; MAGIC_NUMBERS.len()]>::try_from(magic_octets)
            .map_err(|_| ParserError::ErrorVerify.to_nom(rest))?;
        if magic_octets != MAGIC_NUMBERS {
            return Err(ParserError::ErrorVerify.to_nom(rest));
        }

        let mut rest_buffer = rest;
        loop {
            let (rest, option) = Options::parse(rest_buffer)?;
            if option == OptionData::Pad {
                continue;
            } else if option == OptionData::End {
                break;
            }

            options.push(option);
            rest_buffer = rest;
        }
    }

    let protocol = DHCPv4 {
        message_type: op,
        hardware_type: htype,
        hardware_length: hlen,
        hops,
        x_id: xid,
        secs,
        broadcast_flag,
        old_client_address: ciaddr,
        new_client_address: yiaddr,
        server_address: siaddr,
        relay_agent_address: giaddr,
        hardware_address_client: chaddr,
        server_name: sname,
        file_name: file,
        options,
    };

    let empty: &[u8] = &[];
    Finish::finish(Ok((empty, ProtocolData::DHCPv4(protocol))))
}

pub fn is_protocol_default(port_source: u16, port_destination: u16) -> bool {
    const SERVER_PORT: u16 = 67;
    const CLIENT_PORT: u16 = 68;

    port_source == SERVER_PORT
        || port_source == CLIENT_PORT
        || port_destination == SERVER_PORT
        || port_destination == CLIENT_PORT
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DHCPv4 {
    pub message_type: OperationType,
    pub hardware_type: HardwareType,
    pub hardware_length: u8,
    pub hops: u8,
    pub x_id: u32,
    pub secs: u16,
    pub broadcast_flag: u8,
    pub old_client_address: Ipv4Addr,
    pub new_client_address: Ipv4Addr,
    pub server_address: Ipv4Addr,
    pub relay_agent_address: Ipv4Addr,
    pub hardware_address_client: MacAddress,
    pub server_name: Option<String>,
    pub file_name: Option<String>,
    pub options: Vec<OptionData>,
}

// FUTURE: Other options...
// RFC 2132: https://datatracker.ietf.org/doc/html/rfc2132
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Options {
    Pad = 0,
    End = 255,

    SubnetMask = 1,
    RouterOption = 3,
    DomainNameServer = 6,
    DomainName = 15,
    MessageType = 53,
    ServerIdentifier = 54,
}

impl Options {
    pub fn parse(input: &[u8]) -> IResult<&[u8], OptionData> {
        let (rest, code) = be_u8().parse(input)?;
        let option = match Options::try_from(code) {
            Ok(value) => value,
            Err(_) => return Err(ParserError::ErrorVerify.to_nom(input)),
        };

        // Just skipping or breaking
        if let Options::Pad = option {
            return Ok((rest, OptionData::Pad));
        } else if let Options::End = option {
            return Ok((rest, OptionData::End));
        }

        let (rest, length) = be_u8().parse(rest)?;
        let (rest, content) = take(length).parse(rest)?;

        let data = match option {
            Options::DomainName => {
                let domain_name = String::from_utf8(content.to_vec())
                    .map_err(|_| ParserError::ErrorVerify.to_nom(rest))?
                    .trim_matches('\0')
                    .to_string();

                OptionData::DomainName(domain_name)
            },

            Options::DomainNameServer => {
                if length % 4 != 0 {
                    return Err(ParserError::ErrorVerify.to_nom(input));
                }
                let addresses = length / 4;

                let mut ips: Vec<Ipv4Addr> = vec![];
                let mut outer_rest = content;
                for _ in 0..addresses {
                    let (rest, address) = ip::address::v4_parse(outer_rest)?;
                    ips.push(address);
                    outer_rest = rest;
                }

                OptionData::DomainNameServer(ips)
            },

            Options::MessageType => {
                let (_, message_type) = be_u8().parse(content)?;
                let value = MessageType::try_from(message_type)
                    .map_err(|_| ParserError::ErrorVerify.to_nom(rest))?;
                OptionData::MessageType(value)
            },

            Options::RouterOption => {
                if length % 4 != 0 {
                    return Err(ParserError::ErrorVerify.to_nom(input));
                }
                let addresses = length / 4;

                let mut ips: Vec<Ipv4Addr> = vec![];
                let mut outer_rest = content;
                for _ in 0..addresses {
                    let (rest, address) = ip::address::v4_parse(outer_rest)?;
                    ips.push(address);
                    outer_rest = rest;
                }

                OptionData::RouterOption(ips)
            },

            Options::ServerIdentifier => {
                let (_, ip) = ip::address::v4_parse(content)?;
                OptionData::ServerIdentifier(ip)
            },

            Options::SubnetMask => {
                let (_, mask) = ip::address::v4_parse(content)?;
                OptionData::SubnetMask(mask)
            },

            Options::Pad | Options::End => {
                return Err(ParserError::ErrorVerify.to_nom(input));
            },
        };

        Ok((rest, data))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OptionData {
    Pad,
    End,

    DomainName(String),
    DomainNameServer(Vec<Ipv4Addr>),
    MessageType(MessageType),
    RouterOption(Vec<Ipv4Addr>),
    ServerIdentifier(Ipv4Addr),
    SubnetMask(Ipv4Addr),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum OperationType {
    BootRequest = 1,
    BootReply = 2,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum MessageType {
    Discover = 1,
    Offer = 2,
    Request = 3,
    Decline = 4,
    ACK = 5,
    NAK = 6,
    Release = 7,
    Inform = 8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DHCPv4Dto {
    pub message_type: OperationType,
    pub old_client_address: Ipv4Addr,
    pub new_client_address: Ipv4Addr,
    pub server_address: Ipv4Addr,
    pub relay_agent_address: Ipv4Addr,
    pub hardware_address_client: MacAddress,
}

impl From<DHCPv4> for DHCPv4Dto {
    fn from(value: DHCPv4) -> Self {
        Self {
            message_type: value.message_type,
            old_client_address: value.old_client_address,
            new_client_address: value.new_client_address,
            server_address: value.server_address,
            relay_agent_address: value.relay_agent_address,
            hardware_address_client: value.hardware_address_client,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::frame::FrameHeader;
    use crate::parser::tests::FrameType;
    use crate::parser::tests::ProtocolParser;
    use crate::protocols::ProtocolData;
    use crate::protocols::arp::hardware_type::HardwareType;
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv4::IPv4;
    use crate::protocols::udp::UDP;
    use std::str::FromStr;

    #[test]
    fn test_dhcp() {
        let hex_actual = "FF FF FF FF FF FF 00 19 B9 DA 15 A0 08 00 45 00 01 48 13 DE 00 00 80 11 F4 B0 AC 10 85 06 FF FF FF FF 00 43 00 44 01 34 38 ED 02 01 06 00 65 BB D3 BB 00 00 80 00 AC 10 85 27 00 00 00 00 00 00 00 00 00 00 00 00 D4 BE D9 28 21 33 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 63 82 53 63 35 01 05 36 04 AC 10 85 06 01 04 FF FF FF 00 0F 0B 6A 61 61 6C 61 6D 2E 6E 65 74 00 03 04 AC 10 85 01 06 08 AC 10 85 06 AC 10 80 CA FF 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 342,
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
            destination_mac: MacAddress::try_from("ff:ff:ff:ff:ff:ff").unwrap(),
            source_mac: MacAddress::try_from("00:19:b9:da:15:a0").unwrap(),
            ether_type: EtherType::Ipv4,
        };

        assert_eq!(actual_ethernet, expected_ethernet);

        let actual_ipv4 = match metadata.layers[1].clone() {
            ProtocolData::IPv4(value) => value,
            _ => panic!(),
        };

        let expected_ipv4 = IPv4 {
            version: 4,
            internet_header_length: 20,
            differentiated_services_code_point: 0,
            explicit_congestion_notification: 0,
            total_length: 328,
            identification: 0x13de,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 128,
            protocol_inner: IpNextLevelProtocol::UDP,
            checksum: 0xf4b0,
            address_source: Ipv4Addr::from_str("172.16.133.6").unwrap(),
            address_destination: Ipv4Addr::from_str("255.255.255.255").unwrap(),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_udp = match metadata.layers[2].clone() {
            ProtocolData::UDP(value) => value,
            _ => panic!(),
        };

        let expected_udp = UDP {
            port_source: 67,
            port_destination: 68,
            length: 308,
            checksum: 0x38ed,
        };

        assert_eq!(actual_udp, expected_udp);

        let actual_dhcp = match metadata.layers[3].clone() {
            ProtocolData::DHCPv4(value) => value,
            _ => panic!(),
        };

        let expected_dhcp = DHCPv4 {
            message_type: OperationType::BootReply,
            hardware_type: HardwareType::Ethernet,
            hardware_length: 6,
            hops: 0,
            x_id: 0x65bbd3bb,
            secs: 0,
            broadcast_flag: 1,
            old_client_address: Ipv4Addr::from_str("172.16.133.39").unwrap(),
            new_client_address: Ipv4Addr::from_str("0.0.0.0").unwrap(),
            server_address: Ipv4Addr::from_str("0.0.0.0").unwrap(),
            relay_agent_address: Ipv4Addr::from_str("0.0.0.0").unwrap(),
            hardware_address_client: MacAddress::try_from("d4:be:d9:28:21:33").unwrap(),
            server_name: None,
            file_name: None,
            options: vec![
                OptionData::MessageType(MessageType::ACK),
                OptionData::ServerIdentifier(Ipv4Addr::from_str("172.16.133.6").unwrap()),
                OptionData::SubnetMask(Ipv4Addr::from_str("255.255.255.0").unwrap()),
                OptionData::DomainName("jaalam.net".to_string()),
                OptionData::RouterOption(vec![
                    Ipv4Addr::from_str("172.16.133.1").unwrap(),
                ]),
                OptionData::DomainNameServer(vec![
                    Ipv4Addr::from_str("172.16.133.6").unwrap(),
                    Ipv4Addr::from_str("172.16.128.202").unwrap(),
                ]),
            ],
        };

        assert_eq!(actual_dhcp, expected_dhcp);
    }
}
