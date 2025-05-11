use dpi::protocols::ethernet::mac::{MacAddress, Vendor};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Default)]
pub struct DeviceStorage {
    pub list: Vec<LocalDevice>,
    pub aliases: HashMap<MacAddress, String>,
}

impl DeviceStorage {
    pub fn find_by_mac(&mut self, mac: &MacAddress) -> Option<&mut LocalDevice> {
        self.list.iter_mut().find(|dev| dev.mac.eq(mac))
    }
}

pub struct LocalDevice {
    pub mac: MacAddress,
    pub ip: Vec<Ipv4Addr>,
    pub ipv6: Vec<Ipv6Addr>,
    pub vendor: Option<Vendor>,
}
