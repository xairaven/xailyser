use crate::net::raw::RawStorage;

pub const PCAP_FILTER_NAME: &str = "PCAP";
pub const PCAP_FILTER_EXTENSIONS: &[&str] = &["pcap"];

#[derive(Default)]
pub struct NetStorage {
    pub raw: RawStorage,
}

pub mod raw;
