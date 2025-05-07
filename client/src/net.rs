use crate::net::device::DeviceStorage;
use crate::net::raw::RawStorage;
use crate::net::speed::SpeedData;

pub const PCAP_FILTER_NAME: &str = "PCAP";
pub const PCAP_FILTER_EXTENSIONS: &[&str] = &["pcap"];

pub struct NetStorage {
    pub devices: DeviceStorage,
    pub raw: RawStorage,
    pub speed: SpeedData,
}

pub mod device;
pub mod raw;
pub mod speed;
