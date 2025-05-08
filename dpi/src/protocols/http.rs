use crate::parser::ParserError;
use crate::protocols::ProtocolData;
use nom::IResult;
use nom::Parser;
use nom::bytes::complete::take;
use nom::bytes::{tag, take_until};
use nom::sequence::terminated;
use serde::{Deserialize, Serialize};

// HTTP Protocol
// RFC 2616: https://datatracker.ietf.org/doc/html/rfc2616

pub const CRLF: &str = "\r\n";
pub fn parse(bytes: &[u8]) -> IResult<&[u8], ProtocolData> {
    let (rest, potential_starting_line) =
        terminated(take_until(CRLF), tag(CRLF)).parse(bytes)?;
    let starting_line = std::str::from_utf8(potential_starting_line)
        .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    let (rest, http) = if starting_line.starts_with("HTTP/") {
        parse_response(rest, starting_line)
    } else {
        parse_request(rest, starting_line)
    }
    .map_err(|_| ParserError::ErrorVerify.to_nom(bytes))?;

    if !rest.is_empty() {
        return Err(ParserError::ErrorVerify.to_nom(bytes));
    }

    Ok((rest, ProtocolData::HTTP(http)))
}

pub fn parse_request<'a>(
    input: &'a [u8], starting_line: &str,
) -> IResult<&'a [u8], HTTP> {
    let mut starting_line_parts = starting_line.splitn(3, " ");
    let method = Methods::try_from(
        starting_line_parts
            .next()
            .ok_or(ParserError::ErrorVerify.to_nom(input))?,
    )
    .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
    let target = starting_line_parts
        .next()
        .ok_or(ParserError::ErrorVerify.to_nom(input))?;
    let version = starting_line_parts
        .next()
        .ok_or(ParserError::ErrorVerify.to_nom(input))?;

    let (rest, headers) = parse_headers(input)?;

    let (rest, body) = parse_body(rest, &headers)?;

    let protocol = HTTPRequest {
        method,
        target: target.to_string(),
        version: version.to_string(),
        headers,
        body,
    };

    Ok((rest, HTTP::Request(protocol)))
}

pub fn parse_response<'a>(
    input: &'a [u8], starting_line: &str,
) -> IResult<&'a [u8], HTTP> {
    let mut starting_line_parts = starting_line.splitn(3, " ");
    let version = starting_line_parts
        .next()
        .ok_or(ParserError::ErrorVerify.to_nom(input))?;
    let status_code = starting_line_parts
        .next()
        .ok_or(ParserError::ErrorVerify.to_nom(input))?
        .parse::<u16>()
        .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
    let reason_phrase = starting_line_parts
        .next()
        .ok_or(ParserError::ErrorVerify.to_nom(input))?;

    let (rest, headers) = parse_headers(input)?;

    let (rest, body) = parse_body(rest, &headers)?;

    let protocol = HTTPResponse {
        version: version.to_string(),
        status_code,
        reason: reason_phrase.to_string(),
        headers,
        body,
    };

    Ok((rest, HTTP::Response(protocol)))
}

fn parse_headers(input: &[u8]) -> IResult<&[u8], Vec<Header>> {
    let mut headers: Vec<Header> = Vec::new();
    let mut rest_buffer = input;

    loop {
        if let Ok((rest, _)) =
            tag::<_, _, (&[u8], nom::error::ErrorKind)>(CRLF).parse(rest_buffer)
        {
            rest_buffer = rest;
            break;
        }

        let (rest, header_bytes) =
            terminated(take_until(CRLF), tag(CRLF)).parse(rest_buffer)?;
        let header_line = std::str::from_utf8(header_bytes)
            .map_err(|_| ParserError::ErrorVerify.to_nom(rest_buffer))?;
        if let Some((key, value)) = header_line.split_once(": ") {
            headers.push((key.to_string(), value.to_string()));
        }

        rest_buffer = rest;
    }

    Ok((rest_buffer, headers))
}

fn parse_body<'a>(input: &'a [u8], headers: &[Header]) -> IResult<&'a [u8], Vec<u8>> {
    // Seeking for 'Content-Length'
    if let Some((_, value)) = headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("Content-Length"))
    {
        let len: usize = value
            .parse::<usize>()
            .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
        let (rest, body) = take(len)(input)?;
        return Ok((rest, body.to_vec()));
    }
    // Otherwise seeking for 'Transfer-Encoding'
    if headers.iter().any(|(key, value)| {
        key.eq_ignore_ascii_case("Transfer-Encoding")
            && value.eq_ignore_ascii_case("chunked")
    }) {
        return parse_chunked(input);
    }
    // No body
    Ok((input, Vec::new()))
}

// Parser of chunked
fn parse_chunked(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let mut body = Vec::new();
    let mut rest_buffer = input;

    loop {
        // 1) Reading chunk size (HEX) until CRLF
        let (rest, size_line) =
            terminated(take_until(CRLF), tag(CRLF)).parse(rest_buffer)?;
        let size_str = std::str::from_utf8(size_line)
            .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
        let size = usize::from_str_radix(size_str.trim(), 16)
            .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
        rest_buffer = rest;

        // 2) 0 Size - End
        if size == 0 {
            if let Ok((rest, _)) =
                tag::<_, _, (&[u8], nom::error::ErrorKind)>(CRLF).parse(rest_buffer)
            {
                return Ok((rest, body));
            }
        }

        // 3) Reading chunk + CRLF
        let (rest, chunk) = take(size).parse(rest_buffer)?;
        let (rest, _) = tag(CRLF).parse(rest)?;
        body.extend_from_slice(chunk);
        rest_buffer = rest;
    }
}

pub fn is_protocol_default(port_source: u16, port_destination: u16) -> bool {
    const PORT: u16 = 80;

    port_source == PORT || port_destination == PORT
}

#[derive(Clone, Debug, PartialEq)]
pub enum HTTP {
    Request(HTTPRequest),
    Response(HTTPResponse),
}

pub type Header = (String, String);

#[derive(Clone, Debug, PartialEq)]
pub struct HTTPRequest {
    pub method: Methods,
    pub target: String,
    pub version: String,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HTTPResponse {
    pub version: String,
    pub status_code: u16,
    pub reason: String,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Methods {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
    PATCH,
    CONNECT,
}

impl TryFrom<&str> for Methods {
    type Error = ParserError;
    fn try_from(method: &str) -> Result<Self, Self::Error> {
        let line = method.trim().to_ascii_uppercase();
        let line = line.as_str();
        match line {
            "GET" => Ok(Methods::GET),
            "POST" => Ok(Methods::POST),
            "PUT" => Ok(Methods::PUT),
            "DELETE" => Ok(Methods::DELETE),
            "HEAD" => Ok(Methods::HEAD),
            "OPTIONS" => Ok(Methods::OPTIONS),
            "TRACE" => Ok(Methods::TRACE),
            "PATCH" => Ok(Methods::PATCH),
            "CONNECT" => Ok(Methods::CONNECT),
            _ => Err(ParserError::ErrorVerify),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum HttpDto {
    Request(HTTPRequestDto),
    Response(HTTPResponseDto),
}

impl From<HTTP> for HttpDto {
    fn from(value: HTTP) -> Self {
        match value {
            HTTP::Request(value) => Self::Request(value.into()),
            HTTP::Response(value) => Self::Response(value.into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HTTPRequestDto {
    pub method: Methods,
    pub target: String,
    pub headers: Vec<Header>,
}

impl From<HTTPRequest> for HTTPRequestDto {
    fn from(value: HTTPRequest) -> Self {
        Self {
            method: value.method,
            target: value.target,
            headers: value.headers,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HTTPResponseDto {
    pub status_code: u16,
    pub reason: String,
    pub headers: Vec<Header>,
}

impl From<HTTPResponse> for HTTPResponseDto {
    fn from(value: HTTPResponse) -> Self {
        Self {
            status_code: value.status_code,
            reason: value.reason,
            headers: value.headers,
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
    use crate::protocols::tcp;
    use crate::protocols::tcp::TCP;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    #[test]
    fn test_http_request() {
        let hex_actual = "00 1F F3 3C E1 13 F8 1E DF E5 84 3A 08 00 45 00 02 09 C7 2A 40 00 40 06 2D 58 AC 10 0B 0C D8 22 B5 2D FC 45 00 50 E8 B7 30 BC EC B9 A3 7A 80 18 FF FF 1A 7E 00 00 01 01 08 0A 1A 7D 84 38 AA E7 7F C8 47 45 54 20 2F 20 48 54 54 50 2F 31 2E 31 0D 0A 48 6F 73 74 3A 20 73 6C 61 73 68 64 6F 74 2E 6F 72 67 0D 0A 55 73 65 72 2D 41 67 65 6E 74 3A 20 4D 6F 7A 69 6C 6C 61 2F 35 2E 30 20 28 4D 61 63 69 6E 74 6F 73 68 3B 20 55 3B 20 49 6E 74 65 6C 20 4D 61 63 20 4F 53 20 58 20 31 30 2E 36 3B 20 65 6E 2D 55 53 3B 20 72 76 3A 31 2E 39 2E 32 2E 36 29 20 47 65 63 6B 6F 2F 32 30 31 30 30 36 32 35 20 46 69 72 65 66 6F 78 2F 33 2E 36 2E 36 0D 0A 41 63 63 65 70 74 3A 20 74 65 78 74 2F 68 74 6D 6C 2C 61 70 70 6C 69 63 61 74 69 6F 6E 2F 78 68 74 6D 6C 2B 78 6D 6C 2C 61 70 70 6C 69 63 61 74 69 6F 6E 2F 78 6D 6C 3B 71 3D 30 2E 39 2C 2A 2F 2A 3B 71 3D 30 2E 38 0D 0A 41 63 63 65 70 74 2D 4C 61 6E 67 75 61 67 65 3A 20 65 6E 2D 75 73 2C 65 6E 3B 71 3D 30 2E 35 0D 0A 41 63 63 65 70 74 2D 45 6E 63 6F 64 69 6E 67 3A 20 67 7A 69 70 2C 64 65 66 6C 61 74 65 0D 0A 41 63 63 65 70 74 2D 43 68 61 72 73 65 74 3A 20 49 53 4F 2D 38 38 35 39 2D 31 2C 75 74 66 2D 38 3B 71 3D 30 2E 37 2C 2A 3B 71 3D 30 2E 37 0D 0A 4B 65 65 70 2D 41 6C 69 76 65 3A 20 31 31 35 0D 0A 43 6F 6E 6E 65 63 74 69 6F 6E 3A 20 6B 65 65 70 2D 61 6C 69 76 65 0D 0A 43 6F 6F 6B 69 65 3A 20 5F 5F 75 74 6D 61 3D 39 32 37 33 38 34 37 2E 31 38 36 38 36 30 35 31 37 36 2E 31 31 34 31 33 32 33 37 35 38 2E 31 31 35 31 30 33 39 38 38 34 2E 31 31 36 37 35 38 37 30 32 34 2E 34 0D 0A 43 61 63 68 65 2D 43 6F 6E 74 72 6F 6C 3A 20 6D 61 78 2D 61 67 65 3D 30 0D 0A 0D 0A".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 535,
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
            destination_mac: MacAddress::try_from("00:1f:f3:3c:e1:13").unwrap(),
            source_mac: MacAddress::try_from("f8:1e:df:e5:84:3a").unwrap(),
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
            total_length: 521,
            identification: 0xc72a,
            flags: 2,
            fragment_offset: 0,
            time_to_live: 64,
            protocol_inner: IpNextLevelProtocol::TCP,
            checksum: 0x2d58,
            address_source: Ipv4Addr::from_str("172.16.11.12").unwrap(),
            address_destination: Ipv4Addr::from_str("216.34.181.45").unwrap(),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_tcp = match metadata.layers[2].clone() {
            ProtocolData::TCP(value) => value,
            _ => panic!(),
        };

        let expected_tcp = TCP {
            port_source: 64581,
            port_destination: 80,
            sequence_number: 0xe8b730bc,
            acknowledgement_number: 0xecb9a37a,
            data_offset: 32,
            reserved: 0,
            flags: tcp::Flags {
                congestion_window_reduced: false,
                ecn_echo: false,
                urgent: false,
                acknowledgment: true,
                push: true,
                reset: false,
                syn: false,
                fin: false,
            },
            window: 65535,
            checksum: 0x1a7e,
            urgent_pointer: 0,
            options: vec![
                tcp::OptionData::NoOperation,
                tcp::OptionData::NoOperation,
                tcp::OptionData::Timestamps(444433464, 2867298248),
            ],
        };

        assert_eq!(actual_tcp, expected_tcp);

        let actual_http = match metadata.layers[3].clone() {
            ProtocolData::HTTP(HTTP::Request(value)) => value,
            _ => panic!(),
        };

        let expected_http = HTTPRequest {
            method: Methods::GET,
            target: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: vec![
                ("Host".to_string(), "slashdot.org".to_string()),
                ("User-Agent".to_string(), "Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; en-US; rv:1.9.2.6) Gecko/20100625 Firefox/3.6.6".to_string()),
                ("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
                ("Accept-Language".to_string(), "en-us,en;q=0.5".to_string()),
                ("Accept-Encoding".to_string(), "gzip,deflate".to_string()),
                ("Accept-Charset".to_string(), "ISO-8859-1,utf-8;q=0.7,*;q=0.7".to_string()),
                ("Keep-Alive".to_string(), "115".to_string()),
                ("Connection".to_string(), "keep-alive".to_string()),
                ("Cookie".to_string(), "__utma=9273847.1868605176.1141323758.1151039884.1167587024.4".to_string()),
                ("Cache-Control".to_string(), "max-age=0".to_string()),
            ],
            body: vec![],
        };

        assert_eq!(actual_http, expected_http);
    }

    #[test]
    fn test_http_response() {
        let hex_actual = "00 21 70 61 E1 F8 00 90 7F 3E 02 D0 08 00 45 00 00 C0 1F BF 40 00 7E 06 7E 5D AC 10 80 A9 AC 10 85 51 1F 4E E6 8E ED 44 84 44 F8 AA 17 EB 50 19 FF FF 28 1F 00 00 48 54 54 50 2F 31 2E 31 20 32 30 30 20 4F 4B 0D 0A 44 61 74 65 3A 20 54 75 65 2C 20 32 36 20 46 65 62 20 32 30 31 33 20 32 31 3A 35 37 3A 30 35 20 47 4D 54 0D 0A 53 65 72 76 65 72 3A 20 41 70 61 63 68 65 0D 0A 43 6F 6E 6E 65 63 74 69 6F 6E 3A 20 63 6C 6F 73 65 0D 0A 43 6F 6E 74 65 6E 74 2D 4C 65 6E 67 74 68 3A 20 32 0D 0A 43 6F 6E 74 65 6E 74 2D 54 79 70 65 3A 20 61 70 70 6C 69 63 61 74 69 6F 6E 2F 78 2D 6D 73 64 6F 77 6E 6C 6F 61 64 0D 0A 0D 0A 4F 4B".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();
        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 206,
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
            destination_mac: MacAddress::try_from("00:21:70:61:e1:f8").unwrap(),
            source_mac: MacAddress::try_from("00:90:7f:3e:02:d0").unwrap(),
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
            total_length: 192,
            identification: 0x1fbf,
            flags: 2,
            fragment_offset: 0,
            time_to_live: 126,
            protocol_inner: IpNextLevelProtocol::TCP,
            checksum: 0x7e5d,
            address_source: Ipv4Addr::from_str("172.16.128.169").unwrap(),
            address_destination: Ipv4Addr::from_str("172.16.133.81").unwrap(),
        };

        assert_eq!(actual_ipv4, expected_ipv4);

        let actual_tcp = match metadata.layers[2].clone() {
            ProtocolData::TCP(value) => value,
            _ => panic!(),
        };

        let expected_tcp = TCP {
            port_source: 8014,
            port_destination: 59022,
            sequence_number: 0xed448444,
            acknowledgement_number: 0xf8aa17eb,
            data_offset: 20,
            reserved: 0,
            flags: tcp::Flags {
                congestion_window_reduced: false,
                ecn_echo: false,
                urgent: false,
                acknowledgment: true,
                push: true,
                reset: false,
                syn: false,
                fin: true,
            },
            window: 65535,
            checksum: 0x281f,
            urgent_pointer: 0,
            options: vec![],
        };

        assert_eq!(actual_tcp, expected_tcp);

        let actual_http = match metadata.layers[3].clone() {
            ProtocolData::HTTP(HTTP::Response(value)) => value,
            _ => panic!(),
        };

        let expected_http = HTTPResponse {
            version: "HTTP/1.1".to_string(),
            status_code: 200,
            reason: "OK".to_string(),
            headers: vec![
                (
                    "Date".to_string(),
                    "Tue, 26 Feb 2013 21:57:05 GMT".to_string(),
                ),
                ("Server".to_string(), "Apache".to_string()),
                ("Connection".to_string(), "close".to_string()),
                ("Content-Length".to_string(), "2".to_string()),
                (
                    "Content-Type".to_string(),
                    "application/x-msdownload".to_string(),
                ),
            ],
            body: vec![0x4F, 0x4B],
        };

        assert_eq!(actual_http, expected_http);
    }
}
