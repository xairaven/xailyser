use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{DataLinkReceiver, NetworkInterface};
use thiserror::Error;

/// Usable interfaces. <br>
/// Necessary: Presence of MAC address. <br>
/// Necessary: Presence of IP (at least 1).
pub fn usable_sorted() -> Vec<NetworkInterface> {
    let mut interfaces: Vec<NetworkInterface> = datalink::interfaces()
        .into_iter()
        .filter(|interface| interface.mac.is_some() && !interface.ips.is_empty())
        .collect();

    interfaces.sort_by_key(|interface| interface.ips.len());
    interfaces.reverse();

    interfaces
}

/// Get `NetworkInterface` by its name.
pub fn get_network_interface(
    iface_name: &str,
) -> Result<NetworkInterface, InterfaceError> {
    let needed_interface = |iface: &NetworkInterface| {
        iface.name == iface_name || iface.description == iface_name
    };

    let interfaces = datalink::interfaces();
    interfaces
        .into_iter()
        .find(needed_interface)
        .ok_or(InterfaceError::UnknownInterface)
}

pub fn get_datalink_channel(
    interface: &NetworkInterface,
) -> Result<Box<dyn DataLinkReceiver>, InterfaceError> {
    let (_, rx) = match datalink::channel(interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err(InterfaceError::UnhandledChannelType),
        Err(err) => return Err(InterfaceError::UnableCreateChannel(err)),
    };

    Ok(rx)
}

#[derive(Debug, Error)]
pub enum InterfaceError {
    #[error("There are no interfaces with config interface name.")]
    UnknownInterface,

    #[error("There are no interfaces with config interface name.")]
    UnhandledChannelType,

    #[error("Unable to create channel.")]
    UnableCreateChannel(std::io::Error),
}
