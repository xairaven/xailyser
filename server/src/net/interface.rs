use pnet::datalink::NetworkInterface;

/// Usable interfaces. <br>
/// Necessary: Presence of MAC address. <br>
/// Necessary: Presence of IP (at least 1).
pub fn usable_sorted() -> Vec<NetworkInterface> {
    let mut interfaces: Vec<NetworkInterface> = pnet::datalink::interfaces()
        .into_iter()
        .filter(|interface| interface.mac.is_some() && !interface.ips.is_empty())
        .collect();

    interfaces.sort_by_key(|interface| interface.ips.len());
    interfaces.reverse();

    interfaces
}
