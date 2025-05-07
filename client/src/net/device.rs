use dpi::protocols::ethernet::mac::MacAddress;
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
}

pub struct LocalDevice {
    id: u16,
    mac: MacAddress,
    ip: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
}

impl LocalDevice {
    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn mac(&self) -> &MacAddress {
        &self.mac
    }

    pub fn ip(&self) -> Option<Ipv4Addr> {
        self.ip
    }

    pub fn ipv6(&self) -> Option<Ipv6Addr> {
        self.ipv6
    }
}

pub struct LocalDeviceBuilder {
    pub mac: MacAddress,
    pub ip: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
}

impl LocalDeviceBuilder {
    pub fn build(self, id: u16) -> LocalDevice {
        LocalDevice {
            id,
            mac: self.mac,
            ip: self.ip,
            ipv6: self.ipv6,
        }
    }
}
