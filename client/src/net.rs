use crate::net::device::DeviceStorage;
use crate::net::raw::RawStorage;
use crate::net::speed::SpeedData;
use dpi::analysis::ports::{PortInfo, PortServiceTable};
use dpi::analysis::vendor::OuiRadixTree;
use dpi::protocols::ethernet::mac::{MacAddress, Vendor};

pub const PCAP_FILTER_NAME: &str = "PCAP";
pub const PCAP_FILTER_EXTENSIONS: &[&str] = &["pcap"];

pub struct NetStorage {
    pub devices: DeviceStorage,
    pub lookup: Lookup,
    pub raw: RawStorage,
    pub speed: SpeedData,
}

pub struct Lookup {
    pub port_service: PortServiceTable,
    pub vendors: OuiRadixTree,
}

impl Lookup {
    pub fn load() -> std::io::Result<Lookup> {
        let port_service = dpi::analysis::ports::read_database()?;
        let vendors = dpi::analysis::vendor::read_database()?;

        Ok(Self {
            port_service,
            vendors,
        })
    }

    pub fn find_port(&self, port: &u16) -> Option<&Vec<PortInfo>> {
        self.port_service.get(port)
    }

    pub fn find_vendor(&self, mac: &MacAddress) -> Option<Vendor> {
        dpi::analysis::vendor::lookup_vendor(&self.vendors, mac)
    }
}

pub mod device;
pub mod raw;
pub mod speed;
