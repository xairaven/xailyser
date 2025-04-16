use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::io::{Read, Write};

pub fn compress(message: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(message.as_bytes())?;
    encoder.finish()
}

pub fn decompress(message: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = ZlibDecoder::new(message);
    let mut buffer = String::new();
    decoder.read_to_string(&mut buffer)?;

    Ok(buffer)
}
