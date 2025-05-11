use crate::dto::frame::{FrameMetadata, FrameType, OwnedFrame};
use crate::protocols::{ProtocolData, ProtocolId};
use nom::IResult;
use nom::Parser;
use nom::bytes::take;
use nom::number::be_u8;

pub struct ProtocolParser {
    raw_needed: bool,
    root: Option<ProtocolId>,
}

impl ProtocolParser {
    pub fn new(link_type: &pcap::Linktype, raw_needed: bool) -> Self {
        Self {
            raw_needed,
            root: ProtocolId::root(link_type),
        }
    }

    pub fn process(&self, packet: pcap::Packet) -> Option<FrameType> {
        let mut metadata = FrameMetadata::from_header(packet.header);

        if let Some(root_protocol) = &self.root {
            let result = traversal(root_protocol, &packet, &mut metadata, 0);
            return match result {
                ProcessResult::Complete => Some(FrameType::Metadata(metadata.into())),
                ProcessResult::Incomplete => match self.raw_needed {
                    true => Some(FrameType::Raw(OwnedFrame::from(packet))),
                    false => Some(FrameType::Metadata(metadata.into())),
                },
                ProcessResult::Failed => match self.raw_needed {
                    true => Some(FrameType::Raw(OwnedFrame::from(packet))),
                    false => Some(FrameType::Header(metadata.header)),
                },
            };
        }

        None
    }
}

fn traversal(
    id: &ProtocolId, bytes: &[u8], metadata: &mut FrameMetadata, depth: usize,
) -> ProcessResult {
    const MAX_DEPTH: usize = 16;
    if depth > MAX_DEPTH {
        return ProcessResult::Failed;
    }

    let result = id.parse()(bytes);

    match result {
        Ok(([], layer)) => {
            metadata.layers.push(layer);
            ProcessResult::Complete
        },
        Ok((rest, layer)) => {
            metadata.layers.push(layer);

            if let Some(best) = id.best_children(metadata) {
                return match depth.checked_add(1) {
                    Some(new_depth) => traversal(&best, rest, metadata, new_depth),
                    None => ProcessResult::Failed,
                };
            }

            let children = match id.children() {
                Some(value) => value,
                None => {
                    return ProcessResult::Incomplete;
                },
            };

            for id in children {
                let result = match depth.checked_add(1) {
                    Some(new_depth) => traversal(&id, rest, metadata, new_depth),
                    None => return ProcessResult::Failed,
                };

                match result {
                    ProcessResult::Complete | ProcessResult::Incomplete => {
                        return result;
                    },
                    ProcessResult::Failed => continue,
                }
            }

            ProcessResult::Incomplete
        },
        Err(_) => ProcessResult::Failed,
    }
}

pub fn wire_format(input: &[u8]) -> IResult<&[u8], String> {
    let mut labels = Vec::new();
    let mut rest_buffer = input;
    while !rest_buffer.is_empty() {
        let (rest, len_byte) = be_u8().parse(rest_buffer)?;
        // Null-terminator
        if len_byte == 0 {
            debug_assert!(rest.is_empty());
            rest_buffer = rest;
            break;
        }

        // Unexpected length
        if len_byte as usize > rest.len() {
            return Err(ParserError::ErrorVerify.to_nom(input));
        }

        // Creating label
        let (rest, label): (&[u8], &[u8]) = take(len_byte).parse(rest)?;
        let label = String::from_utf8(label.to_vec())
            .map_err(|_| ParserError::ErrorVerify.to_nom(input))?;
        labels.push(label);

        rest_buffer = rest;
    }

    Ok((rest_buffer, labels.join(".")))
}

pub fn cast_to_bool(bit: u8) -> Result<bool, ParserError> {
    match bit {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ParserError::ErrorVerify),
    }
}

pub type ParseFn = fn(&[u8]) -> IResult<&[u8], ProtocolData>;
pub type PortFn = fn(u16, u16) -> bool;

pub enum ParserError {
    ErrorVerify,
    FailureVerify,
}

impl ParserError {
    pub fn to_nom<T>(&self, input: T) -> nom::Err<nom::error::Error<T>> {
        match self {
            Self::ErrorVerify => nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )),
            Self::FailureVerify => nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProcessResult {
    // Fully parsed
    Complete,

    // Some protocols parsed (we are going into the deep), but some in the deepness are not
    Incomplete,

    // Not matched
    Failed,
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::dto::frame::{FrameHeader, FrameMetadata};
    use crate::parser::ProcessResult;
    use crate::protocols::ProtocolId;
    use libc::timeval;

    pub enum FrameType {
        Metadata(FrameMetadata),
        Header(()),
        Raw(()),
    }

    pub struct ProtocolParser {
        raw_needed: bool,
        root: Option<ProtocolId>,
    }

    impl ProtocolParser {
        pub fn new(link_type: &pcap::Linktype, raw_needed: bool) -> Self {
            Self {
                raw_needed,
                root: ProtocolId::root(link_type),
            }
        }

        pub fn process(&self, packet: pcap::Packet) -> Option<FrameType> {
            let mut metadata = FrameMetadata::from_header(packet.header);

            if let Some(root_protocol) = &self.root {
                let result = super::traversal(root_protocol, &packet, &mut metadata, 0);
                return match result {
                    ProcessResult::Complete => Some(FrameType::Metadata(metadata)),
                    ProcessResult::Incomplete => match self.raw_needed {
                        true => Some(FrameType::Raw(())),
                        false => Some(FrameType::Metadata(metadata)),
                    },
                    ProcessResult::Failed => match self.raw_needed {
                        true => Some(FrameType::Raw(())),
                        false => Some(FrameType::Header(())),
                    },
                };
            }

            None
        }
    }

    #[test]
    fn test_overflow() {
        let hex_actual = "04 E8 B9 18 55 10 84 D8 1B 6E C1 4A 08 00 45 00 05 D4 FC 86 40 00 34 06 44 CF D4 7C 6A 42 C0 A8 00 67 01 BB FF 50 2C 27 E0 83 E8 40 00 EB 50 10 00 A5 BF D3 00 00 29 4A 72 09 DD 3C 71 24 6C 8D 9F F9 C0 0C 15 5C D9 F0 A5 F6 20 51 03 06 CD 99 CE 38 EB 19 CB 92 38 F5 AE 98 BA 8D 98 05 1D 1E DC 37 21 D9 DA DE 04 7B AB BD 6E 1B 80 4A 65 BA CC 3E 62 88 62 74 85 20 B4 A9 18 85 90 D6 66 7A 10 F6 E4 DC 85 55 68 89 4B AE 66 F9 B2 16 DF 00 A3 19 0C 86 97 6F 0F 4C D1 D6 2F 1B D0 A7 30 B2 0A C4 EF 10 AB BB 17 3B 4B 4E 11 D5 E0 05 CE 29 56 94 CB F4 30 CD F4 1C 56 54 30 2F C1 E9 D3 72 17 8F 1B E6 3B EF C7 54 38 97 62 89 3D 65 BE B5 A9 1A A1 07 08 5F 74 DA F0 EE BA E5 FC 2D 82 A9 F8 E8 6D 0A D2 03 D9 9F 26 C8 14 15 2C FD 37 DD 1B 31 1E 6E 46 16 F1 1C 8F 28 7A F7 D5 DB 66 24 41 23 0D F2 C7 1D B1 77 79 69 24 92 61 FB A3 B6 38 83 5C 48 CB AB D0 51 2A C1 0B 2A 51 61 22 A0 51 2A 89 98 5A 75 F3 58 AA B8 D0 6B E9 C8 7F 51 0B A7 22 90 B8 2D D6 F1 A9 16 3F B2 EF A0 E7 40 9D C2 66 3E 07 A7 99 E3 4E 0E A1 5F 22 FD 0E C2 70 A9 47 21 43 A8 09 2F CA 95 8C B1 15 45 52 B4 30 61 C2 27 F1 F1 8F 7D 52 F5 6A 07 21 0A 2B A6 4A 08 74 DD E8 95 87 BC 7B ED 38 78 CD BC 93 F5 2D E8 2E 0A C7 9A 65 30 E0 D8 DC 03 9C 88 08 88 27 21 37 65 0B EB AF 1E 24 BC 65 B6 B3 4E 71 FE 32 6B 5B 53 53 CA EB BE 41 4C B4 AF E1 16 05 50 CB DD FA 13 9A 6B 9F 71 08 1E 0A 79 3A 66 25 CB A7 78 9E 0B EB 79 71 D9 95 5A 8D E3 F7 21 3F E7 9C 1F CE 26 5B 4D 9D 53 C6 B1 5A 27 69 BD AC 1D 91 26 B2 15 F2 ED 7B 0B CD EE 50 EF DE 84 C7 BE 3B 27 C1 DE 20 B3 DB A4 04 5D FE A1 7A 52 E3 5D E9 29 CE 73 44 BD E8 B3 EE BA 89 27 2E 51 35 4D 63 7B 9E CF C2 2D 81 AF E9 8C C9 14 F5 8F B4 AE 6D B1 50 78 41 49 F9 EF 57 20 79 B8 53 5D 04 4E C4 3F 31 73 29 25 26 F7 06 52 62 AE EA 77 22 2B AB 59 FD BD B3 30 31 31 C0 F3 40 14 0C 74 3F D0 B8 E2 52 3A C8 E2 5D B2 27 89 78 C5 27 B3 0C 05 E0 6F 0F E8 8C E5 E9 64 A4 2F AA 62 44 FA 46 27 4F 7F C1 26 7B 13 32 7C 5D 3E 94 73 EA BD B6 0E 32 B0 40 FB 61 90 74 8C 35 B0 E8 86 76 00 37 84 8F 9B 9A 13 9F B5 77 9D 4B 1E 30 91 66 38 E1 17 8D 4C 1D 48 BE 98 8C 47 10 48 D6 A9 31 07 92 0A 57 80 9D 42 84 BD CD 19 AE 8D 98 CE 87 0C FF 83 FD 3B 9F CB E6 D1 F9 8F B4 9E 03 0A 3E 51 FE 41 15 B5 78 C7 1B 3C 77 F7 56 45 1F B9 3E 19 43 C0 BD 0C B0 E6 D2 30 8A 0E 2D 9F 31 52 1A A2 F1 1D BD 8E 89 5E 02 BA 6C DD A8 C3 15 FB CA B6 6C B3 52 5D 27 69 75 D8 45 4D 5A 98 A5 2C 13 11 73 0E 60 9C 75 B5 74 09 6D 79 F1 4A 94 8E FC FC 49 3D C3 17 A3 C8 EA B7 8A 03 38 44 E4 D3 44 5A 65 43 10 2A 7E 5A A7 42 A5 F4 74 6C B4 C7 65 39 40 1F F7 0D D5 9A 0D 00 82 6D 8B 9A 8D E9 FE 50 AB BF F9 23 6C 25 45 71 55 25 E7 D0 20 DF 94 21 82 69 4C 70 A9 EE 8D AE 10 E7 71 A9 9A 5C 75 32 B6 8C B5 C1 8F 5A A3 C0 59 E6 E9 FC 14 61 5C F4 A5 CF 85 B8 0A E6 73 24 2C B4 9E 3C 92 47 FC 1D 30 DC 9E ED FE F4 B1 FF F8 FF F9 6A F3 91 8A 5C F1 B3 28 16 64 4C 16 89 1B 23 55 83 6F 6F F5 CD 5D CC B3 20 54 87 4F CB A0 A2 68 AB A2 9C 04 64 F1 7B 13 B7 78 57 EA A8 1A 5E 24 9E D9 84 66 EA BE 9B 55 3F A4 EE D9 09 E2 05 8E 59 A5 04 9F 0D F4 F2 DC A6 11 25 3E BE 13 49 40 25 AD 6D 3D 65 58 54 4C 98 69 FF 7D 44 25 60 48 D9 2F E8 D3 B5 D0 00 84 7F 98 D8 14 D4 4C 4B 5B 92 9F 0D 6E 1A A2 97 7A E8 FC 66 D8 CB 48 5C AA C2 91 48 40 15 14 D7 20 AA 09 AD 6E 71 69 F7 45 2F B2 47 9E E3 80 3C 0E 1A 46 3A 58 9B 3A 0F 73 3F 36 B6 F3 F4 1A B3 6C BD 4A FF 10 A6 C2 AF AD B0 65 00 28 C4 93 25 3F C2 80 F8 23 62 B3 0B 67 F8 B9 89 66 5A 12 88 12 0A 3D 54 47 C1 76 B7 BA 1D 9C E8 A7 00 73 29 C9 C0 85 DE B8 96 AE D1 B4 DA B4 77 7A 6D B2 15 AF 85 3F D6 97 B3 71 15 91 36 0E D6 41 62 F9 C7 10 0C 41 43 FA 83 07 78 3F 61 EC 32 60 42 BF E8 7D 9D 20 AD 3E 7B 5E C5 BB 00 9A D2 E9 C9 31 36 DD 43 57 1C BF 72 41 8E 78 AA DC 7C 81 21 15 D6 78 55 F6 7D 70 94 1D 75 5A BF 8A E6 A9 CD 19 32 8D B4 E2 D0 26 41 01 C7 28 ED DF 97 F3 B3 40 25 5A F9 70 95 95 77 52 96 BE 78 EB B0 91 4D 6A 65 28 BB 38 2E 55 71 FD E6 05 C3 C6 DF C9 1F BE 3E E0 BA EC A2 A6 5B 50 9E 09 29 01 FE 4B AF BE 0D 70 A6 B7 6A DF 4C F1 DD A3 23 2D 0C 55 4E D5 C8 2F 93 1C 0F 5B 47 58 B7 45 86 07 ED A3 BF 24 9C 9D 09 CC D8 4C 77 EB B4 80 BE 01 B4 E6 BE 56 9B 79 D2 1C E4 60 76 83 0E A9 14 10 DD 2A 43 9D 5D 04 45 58 58 B5 68 1B 93 38 77 65 BD 1E BA A9 DE 85 78 22 9C 65 24 49 26 A9 80 CD B3 AD 38 B2 00 9D F6 34 2E E4 B1 D7 E5 F5 38 FB E8 7A AF C5 2B C8 9E 4C E9 67 2D 14 C4 57 76 FB 92 FA D2 73 FA 08 C0 96 AB 75 3F CA 7E 43 5E 99 77 C5 5D E3 2E 03 31 ED DD 69 C3 7C 6D 2B C0 1F 51 79 7C 20 B7 27 61 0E EC C0 2E 2B 54 90 CC B6 28 B0 13 CC 13 B4 D1 EA 1E 61 3F 5A 6B 8D 59 D1 5B 81 8B 07 97 45 BB BD E2 65 79 A4 E6 A6 BA 8C 1C 4A C2 C1 E2 2D 7C".replace(" ", "");
        let frame = hex::decode(hex_actual).unwrap();

        let header = FrameHeader {
            tv_sec: 0,
            tv_usec: 0,
            caplen: 1506,
            len: 1506,
        };

        let parser = crate::parser::ProtocolParser::new(&pcap::Linktype(1), false);
        let frame_type = parser.process(pcap::Packet {
            header: &pcap::PacketHeader {
                ts: timeval {
                    tv_sec: header.tv_sec,
                    tv_usec: header.tv_usec,
                },
                caplen: header.caplen,
                len: header.len,
            },
            data: &frame,
        });

        assert!(frame_type.is_some())
    }
}
