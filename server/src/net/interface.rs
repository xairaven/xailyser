use thiserror::Error;

/// Usable interfaces. <br>
/// Necessary: Presence of adresses.
pub fn usable_sorted() -> Result<Vec<pcap::Device>, InterfaceError> {
    let mut interfaces: Vec<pcap::Device> = pcap::Device::list()
        .map_err(InterfaceError::PcapError)?
        .into_iter()
        .filter(|device| !device.addresses.is_empty())
        .collect();

    interfaces.sort_by_key(|device| device.addresses.len());
    interfaces.reverse();

    Ok(interfaces)
}

pub fn get_network_interface_name(network_interface: &pcap::Device) -> String {
    #[cfg(target_os = "windows")]
    let name = if let Some(desc) = &network_interface.desc {
        desc.clone()
    } else {
        network_interface.name.clone()
    };

    #[cfg(target_os = "linux")]
    let name = network_interface.name.clone();

    name
}

/// Get `Device` by its name.
pub fn get_network_interface(device_name: &str) -> Result<pcap::Device, InterfaceError> {
    let needed_interface = |device: &pcap::Device| {
        device.name == device_name || device.desc.as_deref() == Some(device_name)
    };

    usable_sorted()?
        .into_iter()
        .find(needed_interface)
        .ok_or(InterfaceError::UnknownInterface)
}

pub fn get_capture(
    device: pcap::Device, timeout: i32,
) -> Result<pcap::Capture<pcap::Active>, InterfaceError> {
    pcap::Capture::from_device(device)
        .map_err(InterfaceError::PcapError)?
        .timeout(timeout)
        .immediate_mode(true)
        .open()
        .map_err(InterfaceError::PcapError)
}

#[derive(Debug, Error)]
pub enum InterfaceError {
    #[error("Pcap Library error.")]
    PcapError(pcap::Error),

    #[error("There are no interfaces with config interface name.")]
    UnknownInterface,
}
