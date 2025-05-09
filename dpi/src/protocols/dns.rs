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
    let (rest, qname) = parse_name(bytes, whole)?;

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

fn parse_name<'a>(bytes: &'a [u8], whole: &'a [u8]) -> IResult<&'a [u8], String> {
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
                let low6 = (binary_original_octet & 0b0011_1111) as u16; // оставшиеся 6 бит первого байта :contentReference[oaicite:2]{index=2}
                let combined = (low6 << 8) | (next_byte as u16);
                let pointed_slice = whole
                    .get(usize::from(combined)..)
                    .ok_or(ParserError::ErrorVerify.to_nom(bytes))?;
                let (_, str) = parse_name(pointed_slice, whole)?;
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
    let (rest, name) = parse_name(bytes, whole)?;

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
                let (rest, cname) = parse_name(input, whole)?;
                Ok((rest, Self::CNAME(cname)))
            },
            DnsType::NS => {
                let (rest, cname) = parse_name(input, whole)?;
                Ok((rest, Self::NS(cname)))
            },
            DnsType::SOA => {
                let (rest, primary_name_server) = parse_name(input, whole)?;
                let (rest, mailbox) = parse_name(rest, whole)?;
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
}
