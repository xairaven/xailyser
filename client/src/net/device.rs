use dpi::protocols::ethernet::mac::{MacAddress, Vendor};
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Default)]
pub struct DeviceStorage {
    id: u16,
    vec: Vec<LocalDevice>,
}

impl DeviceStorage {
    pub fn add_device(&mut self, device_builder: LocalDeviceBuilder) {
        self.vec.push(device_builder.build(self.id));
        self.id += 1;
    }

    pub fn find_by_mac(&mut self, mac: &MacAddress) -> Option<&mut LocalDevice> {
        self.vec.iter_mut().find(|dev| dev.mac.eq(mac))
    }
}

pub struct LocalDevice {
    id: u16,
    name: Option<String>,
    mac: MacAddress,
    ip: Vec<Ipv4Addr>,
    ipv6: Vec<Ipv6Addr>,
    vendor: Option<Vendor>,
}

impl LocalDevice {
    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn mac(&self) -> &MacAddress {
        &self.mac
    }

    pub fn ip(&self) -> &[Ipv4Addr] {
        &self.ip
    }

    pub fn ip_mut(&mut self) -> &mut Vec<Ipv4Addr> {
        &mut self.ip
    }

    pub fn ipv6(&self) -> &[Ipv6Addr] {
        &self.ipv6
    }

    pub fn ipv6_mut(&mut self) -> &mut Vec<Ipv6Addr> {
        &mut self.ipv6
    }

    pub fn vendor(&self) -> &Option<Vendor> {
        &self.vendor
    }
}

pub struct LocalDeviceBuilder {
    pub name: Option<String>,
    pub mac: MacAddress,
    pub ip: Vec<Ipv4Addr>,
    pub ipv6: Vec<Ipv6Addr>,
    pub vendor: Option<Vendor>,
}

impl LocalDeviceBuilder {
    pub fn build(self, id: u16) -> LocalDevice {
        LocalDevice {
            id,
            name: self.name,
            mac: self.mac,
            ip: self.ip,
            ipv6: self.ipv6,
            vendor: self.vendor,
        }
    }

    pub fn new(mac_address: MacAddress) -> Self {
        Self {
            name: None,
            mac: mac_address,
            ip: vec![],
            ipv6: vec![],
            vendor: None,
        }
    }
}
