use crate::parser::{ParserError, ProtocolParser};
use crate::protocols::ProtocolData;
use nom::IResult;
use nom::bytes::take;
use nom::number::{be_u8, be_u16, be_u32};
use nom::{Finish, Parser, bits};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};
use thiserror::Error;

// DNS Protocol
// RFC 1035: https://datatracker.ietf.org/doc/html/rfc1035

pub const PORT_DNS: u16 = 53;
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
        ProtocolParser::cast_to_bool(aa).map_err(|err| err.to_nom(bytes))?;
    let truncation = ProtocolParser::cast_to_bool(tc).map_err(|err| err.to_nom(bytes))?;
    let recursion_desired =
        ProtocolParser::cast_to_bool(rd).map_err(|err| err.to_nom(bytes))?;
    let recursion_available =
        ProtocolParser::cast_to_bool(ra).map_err(|err| err.to_nom(bytes))?;
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
    let question_section: Option<Vec<QuestionEntry>> = match question_entries > 0 {
        true => {
            let mut vec: Vec<QuestionEntry> = vec![];
            for _ in 0..question_entries {
                let (section_rest, question) = parse_question_section(rest, bytes)?;
                vec.push(question);
                rest = section_rest;
            }
            Some(vec)
        },
        false => None,
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
) -> IResult<&'a [u8], Option<Vec<ResourceRecord>>> {
    let mut rest = bytes;
    let result = match records > 0 {
        true => {
            let mut vec: Vec<ResourceRecord> = vec![];
            for _ in 0..records {
                let (section_rest, question) = parse_resource_record(rest, whole)?;
                vec.push(question);
                rest = section_rest;
            }
            Some(vec)
        },
        false => None,
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
    let data = DnsTypeData::try_from_bytes(data, &record_type)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DNS {
    pub header: Header,
    pub question_section: Option<Vec<QuestionEntry>>,
    pub answer_section: Option<Vec<ResourceRecord>>,
    pub authority_section: Option<Vec<ResourceRecord>>,
    pub additional_section: Option<Vec<ResourceRecord>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    name: String,
    entry_type: DnsType,
    class: Class,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResourceRecord {
    name: String,
    record_type: DnsType,
    class: Class,
    time_to_live: u32,
    data_length: u16,
    data: DnsTypeData,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Query = 0,
    Response = 1,
}

impl TryFrom<u8> for MessageType {
    type Error = DnsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Query),
            1 => Ok(Self::Response),
            _ => Err(Self::Error::MessageTypeUnknown),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OperationCode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatusRequest = 2,
    Reserved,
}

impl TryFrom<u8> for OperationCode {
    type Error = DnsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::StandardQuery),
            1 => Ok(Self::InverseQuery),
            2 => Ok(Self::ServerStatusRequest),
            3..=15 => Ok(Self::Reserved),
            _ => Err(Self::Error::OperationCodeUnknown),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ResponseCode {
    NoErrorCondition = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
    Reserved,
}

impl TryFrom<u8> for ResponseCode {
    type Error = DnsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoErrorCondition),
            1 => Ok(Self::FormatError),
            2 => Ok(Self::ServerFailure),
            3 => Ok(Self::NameError),
            4 => Ok(Self::NotImplemented),
            5 => Ok(Self::Refused),
            6..=15 => Ok(Self::Reserved),
            _ => Err(Self::Error::ResponseCodeUnknown),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

impl TryFrom<u16> for DnsType {
    type Error = DnsError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::A),
            2 => Ok(Self::NS),
            3 => Ok(Self::MD),
            4 => Ok(Self::MF),
            5 => Ok(Self::CNAME),
            6 => Ok(Self::SOA),
            7 => Ok(Self::MB),
            8 => Ok(Self::MG),
            9 => Ok(Self::MR),
            10 => Ok(Self::NULL),
            11 => Ok(Self::WKS),
            12 => Ok(Self::PTR),
            13 => Ok(Self::HINFO),
            14 => Ok(Self::MINFO),
            15 => Ok(Self::MX),
            16 => Ok(Self::TXT),

            17 => Ok(Self::RP),
            18 => Ok(Self::AFSDB),
            24 => Ok(Self::SIG),
            25 => Ok(Self::KEY),
            28 => Ok(Self::AAAA),
            29 => Ok(Self::LOC),
            33 => Ok(Self::SRV),
            35 => Ok(Self::NAPTR),
            36 => Ok(Self::KX),
            37 => Ok(Self::CERT),
            39 => Ok(Self::DNAME),
            42 => Ok(Self::APL),
            43 => Ok(Self::DS),
            44 => Ok(Self::SSHFP),
            45 => Ok(Self::IPSECKEY),
            46 => Ok(Self::RPSIG),
            47 => Ok(Self::NSEC),
            48 => Ok(Self::DNSKEY),
            49 => Ok(Self::DHCID),
            50 => Ok(Self::NSEC3),
            51 => Ok(Self::NSEC3PARAM),
            52 => Ok(Self::TLSA),
            53 => Ok(Self::SMIMEA),
            55 => Ok(Self::HIP),
            59 => Ok(Self::CDS),
            60 => Ok(Self::CDNSKEY),
            61 => Ok(Self::OPENPGPKEY),
            62 => Ok(Self::CSYNC),
            63 => Ok(Self::ZONEMD),
            64 => Ok(Self::SVCB),
            65 => Ok(Self::HTTPS),
            108 => Ok(Self::EUI48),
            109 => Ok(Self::EUI64),
            249 => Ok(Self::TKEY),
            250 => Ok(Self::TSIG),
            252 => Ok(Self::AXFR),
            253 => Ok(Self::MAILB),
            254 => Ok(Self::MAILA),
            255 => Ok(Self::ALL),
            256 => Ok(Self::URI),
            257 => Ok(Self::CAA),
            32768 => Ok(Self::TA),
            32769 => Ok(Self::DLV),

            _ => Err(Self::Error::DnsTypeUnknown),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DnsTypeData {
    AIPv4(Ipv4Addr),
    AIPv6(Ipv6Addr),
    CNAME(String),
    Unknown(Vec<u8>),
}

impl DnsTypeData {
    pub fn try_from_bytes(bytes: &[u8], dns_type: &DnsType) -> Result<Self, DnsError> {
        match dns_type {
            DnsType::A => match bytes.len() {
                4 => {
                    let bytes = <[u8; 4]>::try_from(bytes)
                        .map_err(|_| DnsError::FailedConvertResourceData)?;
                    let address = Ipv4Addr::from(bytes);
                    Ok(Self::AIPv4(address))
                },
                16 => {
                    let bytes = <[u8; 16]>::try_from(bytes)
                        .map_err(|_| DnsError::FailedConvertResourceData)?;
                    let address = Ipv6Addr::from(bytes);
                    Ok(Self::AIPv6(address))
                },
                _ => Err(DnsError::FailedConvertResourceData),
            },
            DnsType::CNAME => {
                let str = String::from_utf8(bytes.to_vec())
                    .map_err(|_| DnsError::FailedConvertResourceData)?;

                Ok(Self::CNAME(str))
            },
            _ => Ok(Self::Unknown(bytes.to_vec())),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Class {
    IN = 1, // The Internet
    CS = 2, // The CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CH = 3, // The CHAOS class
    HS = 4, // Hesiod [Dyer 87]

    ALL = 255,
}

impl TryFrom<u16> for Class {
    type Error = DnsError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::IN),
            2 => Ok(Self::CS),
            3 => Ok(Self::CH),
            4 => Ok(Self::HS),
            255 => Ok(Self::ALL),
            _ => Err(Self::Error::ClassUnknown),
        }
    }
}

#[derive(Clone, Debug, Error, Serialize, Deserialize, PartialEq)]
pub enum DnsError {
    #[error("Unknown Class")]
    ClassUnknown,

    #[error("Unknown DNS Type")]
    DnsTypeUnknown,

    #[error("Failed to convert resource data")]
    FailedConvertResourceData,

    #[error("Unknown QR (Message type)")]
    MessageTypeUnknown,

    #[error("Unknown OPCODE (Operation code)")]
    OperationCodeUnknown,

    #[error("Unknown RCODE (Response Code)")]
    ResponseCodeUnknown,

    #[error("Unknown DNS QType")]
    QTypeUnknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::FrameType;
    use crate::protocols::ethernet::Ethernet;
    use crate::protocols::ethernet::ether_type::EtherType;
    use crate::protocols::ethernet::mac::MacAddress;
    use crate::protocols::ip::protocol::IpNextLevelProtocol;
    use crate::protocols::ipv4::IPv4;
    use crate::protocols::udp::UDP;
    use crate::wrapper::FrameHeader;
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
                FrameType::Raw(_) => panic!(),
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
            question_section: Some(vec![QuestionEntry {
                name: "download.jetbrains.com".to_string(),
                entry_type: DnsType::A,
                class: Class::IN,
            }]),
            answer_section: None,
            authority_section: None,
            additional_section: None,
        };

        assert_eq!(actual_dns, expected_dns);
    }

    #[test]
    fn test_dns_query_authoritative() {
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
                FrameType::Raw(_) => panic!(),
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
            question_section: Some(vec![QuestionEntry {
                name: "www.googleapis.com".to_string(),
                entry_type: DnsType::HTTPS,
                class: Class::IN,
            }]),
            answer_section: None,
            authority_section: Some(vec![ResourceRecord {
                name: "googleapis.com".to_string(),
                record_type: DnsType::SOA,
                class: Class::IN,
                time_to_live: 55,
                data_length: 45,
                data: DnsTypeData::Unknown(vec![
                    0x03, 0x6E, 0x73, 0x31, 0x06, 0x67, 0x6F, 0x6F, 0x67, 0x6C, 0x65,
                    0xC0, 0x1B, 0x09, 0x64, 0x6E, 0x73, 0x2D, 0x61, 0x64, 0x6D, 0x69,
                    0x6E, 0xC0, 0x34, 0x2C, 0xC2, 0x48, 0x8D, 0x00, 0x00, 0x03, 0x84,
                    0x00, 0x00, 0x03, 0x84, 0x00, 0x00, 0x07, 0x08, 0x00, 0x00, 0x00,
                    0x3C,
                ]),
            }]),
            additional_section: None,
        };

        assert_eq!(actual_dns, expected_dns);
    }
}
