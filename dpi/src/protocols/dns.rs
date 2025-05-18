use crate::parser;
use crate::parser::ParserError;
use crate::protocols::ProtocolData;
use nom::IResult;
use nom::bytes::take;
use nom::number::{be_u8, be_u16, be_u32};
use nom::{Finish, Parser, bits};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};
use strum_macros::Display;

// DNS Protocol
// RFC 1035: https://datatracker.ietf.org/doc/html/rfc1035

pub const MESSAGE_TYPE_LENGTH_BITS: usize = 1;
pub const OPERATION_CODE_LENGTH_BITS: usize = 4;
pub const AUTHORITATIVE_ANSWER_LENGTH_BITS: usize = 1;
pub const TRUNCATION_FLAG_LENGTH_BITS: usize = 1;
pub const RECURSION_DESIRED_LENGTH_BITS: usize = 1;
pub const RECURSION_AVAILABLE_LENGTH_BITS: usize = 1;
pub const RESERVED_LENGTH_BITS: usize = 3;
pub const RESPONSE_CODE_LENGTH_BITS: usize = 4;
pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    // HEADER
    // Identifier - 16 bits.
    let (rest, id) = be_u16().parse(bytes)?;

    // Message Type (QR), Operation Code (OPCODE)
    // Authoritative Answer (AA), Truncation (TC), Recursion Desired (RD)
    // Recursion Available (RA), Reserved (Z), Response Code (RCODE)
    type DnsHeaderBits = (u8, u8, u8, u8, u8, u8, u8, u8);
    let (rest, (qr, opcode, aa, tc, rd, ra, z, rcode)): (&[u8], DnsHeaderBits) =
        bits::bits::<_, _, nom::error::Error<_>, _, _>((
            bits::complete::take(MESSAGE_TYPE_LENGTH_BITS),
            bits::complete::take(OPERATION_CODE_LENGTH_BITS),
            bits::complete::take(AUTHORITATIVE_ANSWER_LENGTH_BITS),
            bits::complete::take(TRUNCATION_FLAG_LENGTH_BITS),
            bits::complete::take(RECURSION_DESIRED_LENGTH_BITS),
            bits::complete::take(RECURSION_AVAILABLE_LENGTH_BITS),
            bits::complete::take(RESERVED_LENGTH_BITS),
            bits::complete::take(RESPONSE_CODE_LENGTH_BITS),
        ))(rest)?;
    let message_type =
        MessageType::try_from(qr).map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;
    let operation_code = OperationCode::try_from(opcode)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;
    let authoritative_answer =
        parser::cast_to_bool(aa).map_err(|err| err.to_nom(bytes))?;
    let truncation = parser::cast_to_bool(tc).map_err(|err| err.to_nom(bytes))?;
    let recursion_desired = parser::cast_to_bool(rd).map_err(|err| err.to_nom(bytes))?;
    let recursion_available =
        parser::cast_to_bool(ra).map_err(|err| err.to_nom(bytes))?;
    if z != 0 {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }
    let response_code = ResponseCode::try_from(rcode)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // QDCOUNT - 16 bits
    let (rest, question_entries) = be_u16().parse(rest)?;

    // ANCOUNT - 16 bits,
    let (rest, answer_records) = be_u16().parse(rest)?;

    // NSCOUNT - 16 bits,
    let (rest, authority_records) = be_u16().parse(rest)?;

    // ARCOUNT - 16 bits
    let (mut rest, additional_records) = be_u16().parse(rest)?;

    let header = Header {
        id,
        message_type,
        operation_code,
        authoritative_answer,
        truncation,
        recursion_desired,
        recursion_available,
        response_code,

        question_entries,
        answer_records,
        authority_records,
        additional_records,
    };

    // QUESTION SECTION
    let question_section: Vec<QuestionEntry> = match question_entries > 0 {
        true => {
            let mut vec: Vec<QuestionEntry> = vec![];
            for _ in 0..question_entries {
                let (section_rest, question) = parse_question_section(rest, bytes)?;
                vec.push(question);
                rest = section_rest;
            }
            vec
        },
        false => vec![],
    };

    // ANSWER SECTION
    let (rest, answer_section) = parse_record_section(rest, answer_records, bytes)?;

    // AUTHORITY SECTION
    let (rest, authority_section) = parse_record_section(rest, authority_records, bytes)?;

    // ADDITIONAL SECTION
    let (rest, additional_section) =
        parse_record_section(rest, additional_records, bytes)?;

    let protocol = DNS {
        header,
        question_section,
        answer_section,
        authority_section,
        additional_section,
    };

    if !rest.is_empty() {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    Finish::finish(Ok((rest, ProtocolData::DNS(protocol))))
}

pub fn is_protocol_default(port_source: u16, port_destination: u16) -> bool {
    const PORT_DNS: u16 = 53;

    port_source == PORT_DNS || port_destination == PORT_DNS
}

fn parse_question_section<'a>(
    bytes: &'a [u8], whole: &'a [u8],
) -> IResult<&'a [u8], QuestionEntry> {
    // QNAME
    let (rest, qname) = parse_name(bytes, whole, 1)?;

    // QTYPE
    let (rest, qtype) = be_u16().parse(rest)?;
    let qtype =
        DnsType::try_from(qtype).map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // QCLASS
    let (rest, qclass) = be_u16().parse(rest)?;
    let qclass =
        Class::try_from(qclass).map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    let section = QuestionEntry {
        name: qname,
        entry_type: qtype,
        class: qclass,
    };

    Ok((rest, section))
}

fn parse_name<'a>(
    bytes: &'a [u8], whole: &'a [u8], depth: u8,
) -> IResult<&'a [u8], String> {
    const MAX_DEPTH_LEVEL_RECURSION_NAME_PARSING: u8 = 7;
    if depth > MAX_DEPTH_LEVEL_RECURSION_NAME_PARSING {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    let mut labels: Vec<String> = Vec::new();

    let mut main_rest = bytes;
    loop {
        let (rest, length_octet) = be_u8().parse(main_rest)?;
        if length_octet == 0 {
            main_rest = rest;
            break;
        }

        let is_simple_parsing = (length_octet & 0b1100_0000) != 0b1100_0000;

        match is_simple_parsing {
            true => {
                let (rest, word) = take(length_octet).parse(rest)?;
                let word = String::from_utf8(word.to_vec())
                    .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;
                labels.push(word);
                main_rest = rest;
            },

            false => {
                let (rest, next_byte) = be_u8().parse(rest)?;

                let binary_original_octet = length_octet;
                let low6 = (binary_original_octet & 0b0011_1111) as u16;
                let combined = (low6 << 8) | (next_byte as u16);
                let pointed_slice = whole
                    .get(usize::from(combined)..)
                    .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;
                let (_, str) = parse_name(
                    pointed_slice,
                    whole,
                    depth
                        .checked_add(1)
                        .ok_or(ParserError::ErrorVerify.to_nom(bytes))?,
                )?;
                labels.push(str.to_string());
                main_rest = rest;
                break;
            },
        }
    }
    let name = labels.join(".");

    Ok((main_rest, name))
}

fn parse_record_section<'a>(
    bytes: &'a [u8], records: u16, whole: &'a [u8],
) -> IResult<&'a [u8], Vec<ResourceRecord>> {
    let mut rest = bytes;
    let result = match records > 0 {
        true => {
            let mut vec: Vec<ResourceRecord> = vec![];
            for _ in 0..records {
                let (section_rest, record) = parse_resource_record(rest, whole)?;
                vec.push(record);
                rest = section_rest;
            }
            vec
        },
        false => vec![],
    };

    Ok((rest, result))
}

fn parse_resource_record<'a>(
    bytes: &'a [u8], whole: &'a [u8],
) -> IResult<&'a [u8], ResourceRecord> {
    // NAME
    let (rest, name) = parse_name(bytes, whole, 1)?;

    // TYPE
    let (rest, record_type) = be_u16().parse(rest)?;
    let record_type = DnsType::try_from(record_type)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // CLASS
    let (rest, class) = be_u16().parse(rest)?;
    let class =
        Class::try_from(class).map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    // TTL
    let (rest, time_to_live) = be_u32().parse(rest)?;

    // RDLength
    let (rest, data_length) = be_u16().parse(rest)?;

    // RDATA
    let (rest, data) = take(data_length).parse(rest)?;
    let (_, data) = DnsTypeData::try_from_bytes(data, whole, &record_type)?;

    let record = ResourceRecord {
        name,
        record_type,
        class,
        time_to_live,
        data_length,
        data,
    };

    Ok((rest, record))
}

#[derive(Clone, Debug, PartialEq)]
pub struct DNS {
    pub header: Header,
    pub question_section: Vec<QuestionEntry>,
    pub answer_section: Vec<ResourceRecord>,
    pub authority_section: Vec<ResourceRecord>,
    pub additional_section: Vec<ResourceRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub id: u16,
    pub message_type: MessageType,
    pub operation_code: OperationCode,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub response_code: ResponseCode,

    pub question_entries: u16,
    pub answer_records: u16,
    pub authority_records: u16,
    pub additional_records: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct QuestionEntry {
    pub name: String,
    pub entry_type: DnsType,
    pub class: Class,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResourceRecord {
    pub name: String,
    pub record_type: DnsType,
    pub class: Class,
    pub time_to_live: u32,
    pub data_length: u16,
    pub data: DnsTypeData,
}

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum MessageType {
    Query = 0,
    Response = 1,
}

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum OperationCode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatusRequest = 2,

    #[num_enum(alternatives = [3..15])]
    Reserved = 15,
}

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ResponseCode {
    NoErrorCondition = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,

    #[num_enum(alternatives = [6..15])]
    Reserved = 15,
}

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u16)]
pub enum DnsType {
    A = 1,           // A host address
    NS = 2,          // An authoritative name server
    MD = 3,          // A mail destination (Obsolete - use MX)
    MF = 4,          // A mail forwarder (Obsolete - use MX)
    CNAME = 5,       // The canonical name for an alias
    SOA = 6,         // Marks the start of a zone of authority
    MB = 7,          // A mailbox domain name (EXPERIMENTAL)
    MG = 8,          // A mail group member (EXPERIMENTAL)
    MR = 9,          // A mail rename domain name (EXPERIMENTAL)
    NULL = 10,       // A null RR (EXPERIMENTAL)
    WKS = 11,        // A well known service description
    PTR = 12,        // A domain name pointer
    HINFO = 13,      // Host information
    MINFO = 14,      // Mailbox or mail list information
    MX = 15,         // Mail exchange
    TXT = 16,        // Text strings
    RP = 17,         // Responsible Person
    AFSDB = 18,      // AFS database record
    SIG = 24,        // Signature
    KEY = 25,        // Key record
    AAAA = 28,       // IPv6 address record
    LOC = 29,        // Location record
    SRV = 33,        // Service locator
    NAPTR = 35,      // Naming Authority Pointer
    KX = 36,         // 	Key Exchanger record
    CERT = 37,       // Certificate record
    DNAME = 39,      // 	Delegation name record
    APL = 42,        // Address Prefix List
    DS = 43,         // Delegation signer
    SSHFP = 44,      // SSH Public Key Fingerprint
    IPSECKEY = 45,   // IPsec Key
    RPSIG = 46,      // DNSSEC signature
    NSEC = 47,       // Next Secure record
    DNSKEY = 48,     // DNS Key record
    DHCID = 49,      // DHCP identifier
    NSEC3 = 50,      // Next Secure record version 3
    NSEC3PARAM = 51, // NSEC3 parameters
    TLSA = 52,       // TLSA certificate association
    SMIMEA = 53,     // S/MIME cert association
    HIP = 55,        // Host Identity Protocol
    CDS = 59,        // Child DS
    CDNSKEY = 60,    // ...
    OPENPGPKEY = 61, // OpenPGP public key record
    CSYNC = 62,      // Child-to-Parent Synchronization
    ZONEMD = 63,     // Message Digests for DNS Zones
    SVCB = 64,       // Service Binding
    HTTPS = 65,      // HTTPS Binding

    EUI48 = 108, // MAC address (EUI-48)
    EUI64 = 109, // MAC address (EUI-64)

    TKEY = 249,  // Transaction Key record
    TSIG = 250,  // Transaction Signature
    AXFR = 252,  // A request for a transfer of an entire zone
    MAILB = 253, // A request for mailbox-related records (MB, MG or MR)
    MAILA = 254, // A request for mail agent RRs (Obsolete - see MX)
    ALL = 255,   // A request for all records
    URI = 256,   // Uniform Resource Identifier
    CAA = 257,   // Certification Authority Authorization

    TA = 32768,  // 	DNSSEC Trust Authorities
    DLV = 32769, // DNSSEC Lookaside Validation record
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DnsTypeData {
    AIPv4(Ipv4Addr),
    AIPv6(Ipv6Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
    NS(String),
    SOA {
        primary_name_server: String,
        mailbox: String,
        serial: u32,
        refresh_interval: u32,
        retry_interval: u32,
        expire_limit: u32,
        minimum_ttl: u32,
    },
    Unknown,
}

impl std::fmt::Display for DnsTypeData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let text = match self {
            DnsTypeData::AIPv4(address) => address.to_string(),
            DnsTypeData::AIPv6(address) => address.to_string(),
            DnsTypeData::AAAA(address) => address.to_string(),
            DnsTypeData::CNAME(value) => value.to_string(),
            DnsTypeData::NS(value) => value.to_string(),
            DnsTypeData::SOA {
                primary_name_server,
                mailbox,
                ..
            } => format!("{} <{}>", primary_name_server, mailbox),
            DnsTypeData::Unknown => "Unknown".to_string(),
        };

        write!(f, "{}", text)
    }
}

impl DnsTypeData {
    pub fn try_from_bytes<'a>(
        input: &'a [u8], whole: &'a [u8], dns_type: &DnsType,
    ) -> IResult<&'a [u8], Self> {
        match dns_type {
            DnsType::A => match input.len() {
                4 => {
                    let address = <[u8; 4]>::try_from(input)
                        .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
                    let address = Ipv4Addr::from(address);
                    Ok((&[], Self::AIPv4(address)))
                },
                16 => {
                    let address = <[u8; 16]>::try_from(input)
                        .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
                    let address = Ipv6Addr::from(address);
                    Ok((&[], Self::AIPv6(address)))
                },
                _ => Err(ParserError::ErrorVerify.to_nom(input)),
            },
            DnsType::AAAA => match input.len() {
                16 => {
                    let address = <[u8; 16]>::try_from(input)
                        .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
                    let address = Ipv6Addr::from(address);
                    Ok((&[], Self::AAAA(address)))
                },
                _ => Err(ParserError::ErrorVerify.to_nom(input)),
            },
            DnsType::CNAME => {
                let (rest, cname) = parse_name(input, whole, 1)?;
                Ok((rest, Self::CNAME(cname)))
            },
            DnsType::NS => {
                let (rest, cname) = parse_name(input, whole, 1)?;
                Ok((rest, Self::NS(cname)))
            },
            DnsType::SOA => {
                let (rest, primary_name_server) = parse_name(input, whole, 1)?;
                let (rest, mailbox) = parse_name(rest, whole, 1)?;
                let (rest, serial) = be_u32().parse(rest)?;
                let (rest, refresh_interval) = be_u32().parse(rest)?;
                let (rest, retry_interval) = be_u32().parse(rest)?;
                let (rest, expire_limit) = be_u32().parse(rest)?;
                let (rest, minimum_ttl) = be_u32().parse(rest)?;

                debug_assert!(rest.is_empty());

                Ok((
                    rest,
                    Self::SOA {
                        primary_name_server,
                        mailbox,
                        serial,
                        refresh_interval,
                        retry_interval,
                        expire_limit,
                        minimum_ttl,
                    },
                ))
            },
            _ => Ok((&[], Self::Unknown)),
        }
    }
}

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u16)]
pub enum Class {
    IN = 1, // The Internet
    CS = 2, // The CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CH = 3, // The CHAOS class
    HS = 4, // Hesiod [Dyer 87]

    ALL = 255,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DnsDto {
    pub message_type: MessageType,
    pub operation_code: OperationCode,
    pub authoritative_answer: bool,
    pub response_code: ResponseCode,
    #[serde(default)]
    pub question_section: Vec<QuestionEntry>,
    #[serde(default)]
    pub answer_section: Vec<ResourceRecord>,
    #[serde(default)]
    pub authority_section: Vec<ResourceRecord>,
    #[serde(default)]
    pub additional_section: Vec<ResourceRecord>,
}

impl From<DNS> for DnsDto {
    fn from(value: DNS) -> Self {
        Self {
            message_type: value.header.message_type,
            operation_code: value.header.operation_code,
            authoritative_answer: value.header.authoritative_answer,
            response_code: value.header.response_code,
            question_section: value.question_section,
            answer_section: value.answer_section,
            authority_section: value.authority_section,
            additional_section: value.additional_section,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::frame::FrameHeader;
    use crate::parser::tests::FrameType;
    use crate::parser::tests::ProtocolParser;
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv4::IPv4;
    use crate::protocols::ipv6::IPv6;
    use crate::protocols::udp::UDP;
    use std::str::FromStr;

    #[test]
    fn test_dns_single_query() {
        let hex_actual = "84 D8 1B 6E C1 4A 04 E8 B9 18 55 10 08 00 45 00 00 44 D2 6E 00 00 80 11 00 00 C0 A8 00 67 C0 A8 00 01 E5 13 00 35 00 30 81 FA F3 31 01 00 00 01 00 00 00 00 00 00 08 64 6F 77 6E 6C 6F 61 64 09 6A 65 74 62 72 61 69 6E 73 03 63 6F 6D 00 00 01 00 01".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 82,
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
            destination_mac: MacAddress::try_from("84:D8:1B:6E:C1:4A").unwrap(),
            source_mac: MacAddress::try_from("04:E8:B9:18:55:10").unwrap(),
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
            total_length: 68,
            identification: 0xd26e,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 128,
            protocol_inner: IpNextLevelProtocol::UDP,
            checksum: 0x0000,
            address_source: Ipv4Addr::from_str("192.168.0.103").unwrap(),
            address_destination: Ipv4Addr::from_str("192.168.0.1").unwrap(),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_udp = match metadata.layers[2].clone() {
            ProtocolData::UDP(value) => value,
            _ => panic!(),
        };

        let expected_udp = UDP {
            port_source: 58643,
            port_destination: 53,
            length: 48,
            checksum: 0x81fa,
        };

        assert_eq!(actual_udp, expected_udp);

        let actual_dns = match metadata.layers[3].clone() {
            ProtocolData::DNS(value) => value,
            _ => panic!(),
        };

        let expected_dns = DNS {
            header: Header {
                id: 0xf331,
                message_type: MessageType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncation: false,
                recursion_desired: true,
                recursion_available: false,
                response_code: ResponseCode::NoErrorCondition,
                question_entries: 1,
                answer_records: 0,
                authority_records: 0,
                additional_records: 0,
            },
            question_section: vec![QuestionEntry {
                name: "download.jetbrains.com".to_string(),
                entry_type: DnsType::A,
                class: Class::IN,
            }],
            answer_section: vec![],
            authority_section: vec![],
            additional_section: vec![],
        };

        assert_eq!(actual_dns, expected_dns);
    }

    #[test]
    fn test_dns_query_authoritative_soa() {
        let hex_actual = "04 E8 B9 18 55 10 84 D8 1B 6E C1 4A 08 00 45 00 00 79 56 FF 00 00 3D 11 A4 BC C0 A8 00 01 C0 A8 00 67 00 35 C3 8C 00 65 89 02 BF 9D 81 80 00 01 00 00 00 01 00 00 03 77 77 77 0A 67 6F 6F 67 6C 65 61 70 69 73 03 63 6F 6D 00 00 41 00 01 C0 10 00 06 00 01 00 00 00 37 00 2D 03 6E 73 31 06 67 6F 6F 67 6C 65 C0 1B 09 64 6E 73 2D 61 64 6D 69 6E C0 34 2C C2 48 8D 00 00 03 84 00 00 03 84 00 00 07 08 00 00 00 3C".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 135,
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
            destination_mac: MacAddress::try_from("04:E8:B9:18:55:10").unwrap(),
            source_mac: MacAddress::try_from("84:D8:1B:6E:C1:4A").unwrap(),
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
            total_length: 121,
            identification: 0x56ff,
            flags: 0,
            fragment_offset: 0,
            time_to_live: 61,
            protocol_inner: IpNextLevelProtocol::UDP,
            checksum: 0xa4bc,
            address_source: Ipv4Addr::from_str("192.168.0.1").unwrap(),
            address_destination: Ipv4Addr::from_str("192.168.0.103").unwrap(),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_udp = match metadata.layers[2].clone() {
            ProtocolData::UDP(value) => value,
            _ => panic!(),
        };

        let expected_udp = UDP {
            port_source: 53,
            port_destination: 50060,
            length: 101,
            checksum: 0x8902,
        };

        assert_eq!(actual_udp, expected_udp);

        let actual_dns = match metadata.layers[3].clone() {
            ProtocolData::DNS(value) => value,
            _ => panic!(),
        };

        let expected_dns = DNS {
            header: Header {
                id: 0xbf9d,
                message_type: MessageType::Response,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncation: false,
                recursion_desired: true,
                recursion_available: true,
                response_code: ResponseCode::NoErrorCondition,
                question_entries: 1,
                answer_records: 0,
                authority_records: 1,
                additional_records: 0,
            },
            question_section: vec![QuestionEntry {
                name: "www.googleapis.com".to_string(),
                entry_type: DnsType::HTTPS,
                class: Class::IN,
            }],
            answer_section: vec![],
            authority_section: vec![ResourceRecord {
                name: "googleapis.com".to_string(),
                record_type: DnsType::SOA,
                class: Class::IN,
                time_to_live: 55,
                data_length: 45,
                data: DnsTypeData::SOA {
                    primary_name_server: "ns1.google.com".to_string(),
                    mailbox: "dns-admin.google.com".to_string(),
                    serial: 750930061,
                    refresh_interval: 900,
                    retry_interval: 900,
                    expire_limit: 1800,
                    minimum_ttl: 60,
                },
            }],
            additional_section: vec![],
        };

        assert_eq!(actual_dns, expected_dns);
    }

    #[test]
    fn test_dns_aaaa_ns() {
        let hex_actual = "00 00 86 05 80 DA 00 60 97 07 69 EA 86 DD 60 00 00 00 01 0C 11 E6 3F FE 05 01 48 19 00 00 00 00 00 00 00 00 00 42 3F FE 05 07 00 00 00 01 02 00 86 FF FE 05 80 DA 00 35 09 65 01 0C 19 FC B3 62 85 80 00 01 00 02 00 03 00 03 03 77 77 77 04 77 69 64 65 02 61 64 02 6A 70 00 00 1C 00 01 C0 0C 00 05 00 01 00 00 0E 10 00 11 04 65 6E 64 6F 04 77 69 64 65 02 61 64 02 6A 70 00 04 65 6E 64 6F C0 10 00 1C 00 01 00 00 0E 10 00 10 3F FE 05 01 00 00 10 01 00 00 00 00 00 00 00 02 C0 10 00 02 00 01 00 00 0E 10 00 0F 02 6E 73 04 77 69 64 65 02 61 64 02 6A 70 00 C0 10 00 02 00 01 00 00 0E 10 00 15 02 6E 73 05 74 6F 6B 79 6F 04 77 69 64 65 02 61 64 02 6A 70 00 C0 10 00 02 00 01 00 00 0E 10 00 13 02 6E 73 04 72 63 61 63 03 74 64 69 02 63 6F 02 6A 70 00 02 6E 73 C0 10 00 01 00 01 00 00 0E 10 00 04 CB B2 88 3F 02 6E 73 05 74 6F 6B 79 6F C0 10 00 01 00 01 00 00 0E 10 00 04 CB B2 88 3D 02 6E 73 04 72 63 61 63 03 74 64 69 02 63 6F C0 18 00 01 00 01 00 01 51 80 00 04 CA F9 11 11".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 322,
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
            destination_mac: MacAddress::try_from("00:00:86:05:80:da").unwrap(),
            source_mac: MacAddress::try_from("00:60:97:07:69:ea").unwrap(),
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
            payload_length: 268,
            next_header: IpNextLevelProtocol::UDP,
            hop_limit: 230,
            address_source: Ipv6Addr::from_str("3ffe:501:4819::42").unwrap(),
            address_destination: Ipv6Addr::from_str("3ffe:507:0:1:200:86ff:fe05:80da")
                .unwrap(),
        };

        assert_eq!(actual_ipv6, expected_ipv6);

        let actual_udp = match metadata.layers[2].clone() {
            ProtocolData::UDP(value) => value,
            _ => panic!(),
        };

        let expected_udp = UDP {
            port_source: 53,
            port_destination: 2405,
            length: 268,
            checksum: 0x19fc,
        };

        assert_eq!(actual_udp, expected_udp);

        let actual_dns = match metadata.layers[3].clone() {
            ProtocolData::DNS(value) => value,
            _ => panic!(),
        };

        let expected_dns = DNS {
            header: Header {
                id: 0xb362,
                message_type: MessageType::Response,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: true,
                truncation: false,
                recursion_desired: true,
                recursion_available: true,
                response_code: ResponseCode::NoErrorCondition,
                question_entries: 1,
                answer_records: 2,
                authority_records: 3,
                additional_records: 3,
            },
            question_section: vec![QuestionEntry {
                name: "www.wide.ad.jp".to_string(),
                entry_type: DnsType::AAAA,
                class: Class::IN,
            }],
            answer_section: vec![
                ResourceRecord {
                    name: "www.wide.ad.jp".to_string(),
                    record_type: DnsType::CNAME,
                    class: Class::IN,
                    time_to_live: 3600,
                    data_length: 17,
                    data: DnsTypeData::CNAME("endo.wide.ad.jp".to_string()),
                },
                ResourceRecord {
                    name: "endo.wide.ad.jp".to_string(),
                    record_type: DnsType::AAAA,
                    class: Class::IN,
                    time_to_live: 3600,
                    data_length: 16,
                    data: DnsTypeData::AAAA(
                        Ipv6Addr::from_str("3ffe:501:0:1001::2").unwrap(),
                    ),
                },
            ],
            authority_section: vec![
                ResourceRecord {
                    name: "wide.ad.jp".to_string(),
                    record_type: DnsType::NS,
                    class: Class::IN,
                    time_to_live: 3600,
                    data_length: 15,
                    data: DnsTypeData::NS("ns.wide.ad.jp".to_string()),
                },
                ResourceRecord {
                    name: "wide.ad.jp".to_string(),
                    record_type: DnsType::NS,
                    class: Class::IN,
                    time_to_live: 3600,
                    data_length: 21,
                    data: DnsTypeData::NS("ns.tokyo.wide.ad.jp".to_string()),
                },
                ResourceRecord {
                    name: "wide.ad.jp".to_string(),
                    record_type: DnsType::NS,
                    class: Class::IN,
                    time_to_live: 3600,
                    data_length: 19,
                    data: DnsTypeData::NS("ns.rcac.tdi.co.jp".to_string()),
                },
            ],
            additional_section: vec![
                ResourceRecord {
                    name: "ns.wide.ad.jp".to_string(),
                    record_type: DnsType::A,
                    class: Class::IN,
                    time_to_live: 3600,
                    data_length: 4,
                    data: DnsTypeData::AIPv4(
                        Ipv4Addr::from_str("203.178.136.63").unwrap(),
                    ),
                },
                ResourceRecord {
                    name: "ns.tokyo.wide.ad.jp".to_string(),
                    record_type: DnsType::A,
                    class: Class::IN,
                    time_to_live: 3600,
                    data_length: 4,
                    data: DnsTypeData::AIPv4(
                        Ipv4Addr::from_str("203.178.136.61").unwrap(),
                    ),
                },
                ResourceRecord {
                    name: "ns.rcac.tdi.co.jp".to_string(),
                    record_type: DnsType::A,
                    class: Class::IN,
                    time_to_live: 86400,
                    data_length: 4,
                    data: DnsTypeData::AIPv4(
                        Ipv4Addr::from_str("202.249.17.17").unwrap(),
                    ),
                },
            ],
        };

        assert_eq!(actual_dns, expected_dns);
    }

    #[test]
    fn test_overflow_name() {
        let hex_actual = "04 E8 B9 18 55 10 84 D8 1B 6E C1 4A 08 00 45 00 05 D4 FC 86 40 00 34 06 44 CF D4 7C 6A 42 C0 A8 00 67 01 BB FF 50 2C 27 E0 83 E8 40 00 EB 50 10 00 A5 BF D3 00 00 29 4A 72 09 DD 3C 71 24 6C 8D 9F F9 C0 0C 15 5C D9 F0 A5 F6 20 51 03 06 CD 99 CE 38 EB 19 CB 92 38 F5 AE 98 BA 8D 98 05 1D 1E DC 37 21 D9 DA DE 04 7B AB BD 6E 1B 80 4A 65 BA CC 3E 62 88 62 74 85 20 B4 A9 18 85 90 D6 66 7A 10 F6 E4 DC 85 55 68 89 4B AE 66 F9 B2 16 DF 00 A3 19 0C 86 97 6F 0F 4C D1 D6 2F 1B D0 A7 30 B2 0A C4 EF 10 AB BB 17 3B 4B 4E 11 D5 E0 05 CE 29 56 94 CB F4 30 CD F4 1C 56 54 30 2F C1 E9 D3 72 17 8F 1B E6 3B EF C7 54 38 97 62 89 3D 65 BE B5 A9 1A A1 07 08 5F 74 DA F0 EE BA E5 FC 2D 82 A9 F8 E8 6D 0A D2 03 D9 9F 26 C8 14 15 2C FD 37 DD 1B 31 1E 6E 46 16 F1 1C 8F 28 7A F7 D5 DB 66 24 41 23 0D F2 C7 1D B1 77 79 69 24 92 61 FB A3 B6 38 83 5C 48 CB AB D0 51 2A C1 0B 2A 51 61 22 A0 51 2A 89 98 5A 75 F3 58 AA B8 D0 6B E9 C8 7F 51 0B A7 22 90 B8 2D D6 F1 A9 16 3F B2 EF A0 E7 40 9D C2 66 3E 07 A7 99 E3 4E 0E A1 5F 22 FD 0E C2 70 A9 47 21 43 A8 09 2F CA 95 8C B1 15 45 52 B4 30 61 C2 27 F1 F1 8F 7D 52 F5 6A 07 21 0A 2B A6 4A 08 74 DD E8 95 87 BC 7B ED 38 78 CD BC 93 F5 2D E8 2E 0A C7 9A 65 30 E0 D8 DC 03 9C 88 08 88 27 21 37 65 0B EB AF 1E 24 BC 65 B6 B3 4E 71 FE 32 6B 5B 53 53 CA EB BE 41 4C B4 AF E1 16 05 50 CB DD FA 13 9A 6B 9F 71 08 1E 0A 79 3A 66 25 CB A7 78 9E 0B EB 79 71 D9 95 5A 8D E3 F7 21 3F E7 9C 1F CE 26 5B 4D 9D 53 C6 B1 5A 27 69 BD AC 1D 91 26 B2 15 F2 ED 7B 0B CD EE 50 EF DE 84 C7 BE 3B 27 C1 DE 20 B3 DB A4 04 5D FE A1 7A 52 E3 5D E9 29 CE 73 44 BD E8 B3 EE BA 89 27 2E 51 35 4D 63 7B 9E CF C2 2D 81 AF E9 8C C9 14 F5 8F B4 AE 6D B1 50 78 41 49 F9 EF 57 20 79 B8 53 5D 04 4E C4 3F 31 73 29 25 26 F7 06 52 62 AE EA 77 22 2B AB 59 FD BD B3 30 31 31 C0 F3 40 14 0C 74 3F D0 B8 E2 52 3A C8 E2 5D B2 27 89 78 C5 27 B3 0C 05 E0 6F 0F E8 8C E5 E9 64 A4 2F AA 62 44 FA 46 27 4F 7F C1 26 7B 13 32 7C 5D 3E 94 73 EA BD B6 0E 32 B0 40 FB 61 90 74 8C 35 B0 E8 86 76 00 37 84 8F 9B 9A 13 9F B5 77 9D 4B 1E 30 91 66 38 E1 17 8D 4C 1D 48 BE 98 8C 47 10 48 D6 A9 31 07 92 0A 57 80 9D 42 84 BD CD 19 AE 8D 98 CE 87 0C FF 83 FD 3B 9F CB E6 D1 F9 8F B4 9E 03 0A 3E 51 FE 41 15 B5 78 C7 1B 3C 77 F7 56 45 1F B9 3E 19 43 C0 BD 0C B0 E6 D2 30 8A 0E 2D 9F 31 52 1A A2 F1 1D BD 8E 89 5E 02 BA 6C DD A8 C3 15 FB CA B6 6C B3 52 5D 27 69 75 D8 45 4D 5A 98 A5 2C 13 11 73 0E 60 9C 75 B5 74 09 6D 79 F1 4A 94 8E FC FC 49 3D C3 17 A3 C8 EA B7 8A 03 38 44 E4 D3 44 5A 65 43 10 2A 7E 5A A7 42 A5 F4 74 6C B4 C7 65 39 40 1F F7 0D D5 9A 0D 00 82 6D 8B 9A 8D E9 FE 50 AB BF F9 23 6C 25 45 71 55 25 E7 D0 20 DF 94 21 82 69 4C 70 A9 EE 8D AE 10 E7 71 A9 9A 5C 75 32 B6 8C B5 C1 8F 5A A3 C0 59 E6 E9 FC 14 61 5C F4 A5 CF 85 B8 0A E6 73 24 2C B4 9E 3C 92 47 FC 1D 30 DC 9E ED FE F4 B1 FF F8 FF F9 6A F3 91 8A 5C F1 B3 28 16 64 4C 16 89 1B 23 55 83 6F 6F F5 CD 5D CC B3 20 54 87 4F CB A0 A2 68 AB A2 9C 04 64 F1 7B 13 B7 78 57 EA A8 1A 5E 24 9E D9 84 66 EA BE 9B 55 3F A4 EE D9 09 E2 05 8E 59 A5 04 9F 0D F4 F2 DC A6 11 25 3E BE 13 49 40 25 AD 6D 3D 65 58 54 4C 98 69 FF 7D 44 25 60 48 D9 2F E8 D3 B5 D0 00 84 7F 98 D8 14 D4 4C 4B 5B 92 9F 0D 6E 1A A2 97 7A E8 FC 66 D8 CB 48 5C AA C2 91 48 40 15 14 D7 20 AA 09 AD 6E 71 69 F7 45 2F B2 47 9E E3 80 3C 0E 1A 46 3A 58 9B 3A 0F 73 3F 36 B6 F3 F4 1A B3 6C BD 4A FF 10 A6 C2 AF AD B0 65 00 28 C4 93 25 3F C2 80 F8 23 62 B3 0B 67 F8 B9 89 66 5A 12 88 12 0A 3D 54 47 C1 76 B7 BA 1D 9C E8 A7 00 73 29 C9 C0 85 DE B8 96 AE D1 B4 DA B4 77 7A 6D B2 15 AF 85 3F D6 97 B3 71 15 91 36 0E D6 41 62 F9 C7 10 0C 41 43 FA 83 07 78 3F 61 EC 32 60 42 BF E8 7D 9D 20 AD 3E 7B 5E C5 BB 00 9A D2 E9 C9 31 36 DD 43 57 1C BF 72 41 8E 78 AA DC 7C 81 21 15 D6 78 55 F6 7D 70 94 1D 75 5A BF 8A E6 A9 CD 19 32 8D B4 E2 D0 26 41 01 C7 28 ED DF 97 F3 B3 40 25 5A F9 70 95 95 77 52 96 BE 78 EB B0 91 4D 6A 65 28 BB 38 2E 55 71 FD E6 05 C3 C6 DF C9 1F BE 3E E0 BA EC A2 A6 5B 50 9E 09 29 01 FE 4B AF BE 0D 70 A6 B7 6A DF 4C F1 DD A3 23 2D 0C 55 4E D5 C8 2F 93 1C 0F 5B 47 58 B7 45 86 07 ED A3 BF 24 9C 9D 09 CC D8 4C 77 EB B4 80 BE 01 B4 E6 BE 56 9B 79 D2 1C E4 60 76 83 0E A9 14 10 DD 2A 43 9D 5D 04 45 58 58 B5 68 1B 93 38 77 65 BD 1E BA A9 DE 85 78 22 9C 65 24 49 26 A9 80 CD B3 AD 38 B2 00 9D F6 34 2E E4 B1 D7 E5 F5 38 FB E8 7A AF C5 2B C8 9E 4C E9 67 2D 14 C4 57 76 FB 92 FA D2 73 FA 08 C0 96 AB 75 3F CA 7E 43 5E 99 77 C5 5D E3 2E 03 31 ED DD 69 C3 7C 6D 2B C0 1F 51 79 7C 20 B7 27 61 0E EC C0 2E 2B 54 90 CC B6 28 B0 13 CC 13 B4 D1 EA 1E 61 3F 5A 6B 8D 59 D1 5B 81 8B 07 97 45 BB BD E2 65 79 A4 E6 A6 BA 8C 1C 4A C2 C1 E2 2D 7C".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();

        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 1506,
            len: 1506,
        };
        let packet_header: pcap::PacketHeader = (&header).into();

        let parser = parser::ProtocolParser::new(&pcap::Linktype(1), false);
        let frame_type = parser.process(pcap::Packet {
            header: &packet_header,
            data: &frame,
        });

        assert!(frame_type.is_some())
    }
}
