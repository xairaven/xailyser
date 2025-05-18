use crate::protocols::ethernet::mac::{MacAddress, Vendor};
use radix_tree::{Node, Radix};
use std::io;
use std::io::BufRead;
use std::path::PathBuf;

pub type OuiRadixTree = Node<char, Vendor>;

pub fn read_database(path: PathBuf) -> io::Result<(OuiRadixTree, usize)> {
    let file = std::fs::File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut tree: OuiRadixTree = Node {
        path: vec![],
        data: None,
        indices: vec![],
        nodes: vec![],
    };
    let mut records: usize = 0;
    for line_raw in reader.lines().map_while(Result::ok) {
        let line = line_raw.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, "\t").map(|part| part.trim()).collect();
        debug_assert!(parts.len() == 3);
        let prefix_raw = match parts.first() {
            Some(value) => *value,
            None => return Err(io::ErrorKind::InvalidData.into()),
        };
        let (address_str, mask_length) = match prefix_raw.split_once('/') {
            Some((address, mask)) => (
                address,
                mask.parse::<u8>().map_err(|_| io::ErrorKind::InvalidData)?,
            ),
            None => (prefix_raw, 24),
        };
        let short = match parts.get(1) {
            Some(value) => *value,
            None => return Err(io::ErrorKind::InvalidData.into()),
        };
        let full = match parts.get(2) {
            Some(value) => *value,
            None => return Err(io::ErrorKind::InvalidData.into()),
        };

        let binary = address_str
            .split(':')
            .map(|hex| u8::from_str_radix(hex, 16).map(|byte| format!("{:08b}", byte)))
            .collect::<Result<Vec<String>, _>>()
            .map_err(|_| io::ErrorKind::InvalidData)?
            .join("")
            .get(..mask_length as usize)
            .ok_or(io::ErrorKind::InvalidData)?
            .to_string();

        records = records.checked_add(1).ok_or(io::ErrorKind::InvalidData)?;
        tree.insert(
            binary,
            Vendor {
                short: short.to_string(),
                full: full.to_string(),
            },
        );
    }

    Ok((tree, records))
}

pub fn lookup_vendor(tree: &OuiRadixTree, mac: &MacAddress) -> Option<Vendor> {
    const MASKS: [usize; 3] = [36, 28, 24];
    let bits = mac.to_bit_string();
    for mask in MASKS.iter() {
        let bits = match bits.get(..*mask) {
            Some(value) => value,
            None => continue,
        };
        if let Some(node) = tree.find(bits.to_string()) {
            return node.data.clone();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    const PATH: &str = "../resources/oui-database.txt";
    static TREE: LazyLock<OuiRadixTree> =
        LazyLock::new(|| read_database(PathBuf::from(PATH)).unwrap().0);

    #[test]
    fn test_mac_24() {
        let macs: Vec<MacAddress> = vec![
            MacAddress::try_from("84:D8:1B:6E:C1:4A").unwrap(),
            MacAddress::try_from("AE:AE:C5:85:7B:A3").unwrap(),
            MacAddress::try_from("04:E8:B9:18:55:10").unwrap(),
        ];

        let vendor_actual: Vendor = lookup_vendor(&TREE, &macs[0]).unwrap();
        let vendor_expected = Vendor {
            short: "TpLinkTechno".to_string(),
            full: "Tp-Link Technologies Co.,Ltd.".to_string(),
        };
        assert_eq!(vendor_actual, vendor_expected);

        let vendor_actual: Option<Vendor> = lookup_vendor(&TREE, &macs[1]);
        let vendor_expected = None;
        assert_eq!(vendor_actual, vendor_expected);

        let vendor_actual: Vendor = lookup_vendor(&TREE, &macs[2]).unwrap();
        let vendor_expected = Vendor {
            short: "Intel".to_string(),
            full: "Intel Corporate".to_string(),
        };
        assert_eq!(vendor_actual, vendor_expected);
    }

    #[test]
    fn test_mac_28() {
        let macs: Vec<MacAddress> = vec![
            MacAddress::try_from("FC:A4:7A:40:0F:FF").unwrap(),
            MacAddress::try_from("FC:A4:7A:90:12:34").unwrap(),
            MacAddress::try_from("FC:A4:7A:E0:F0:FF").unwrap(),
        ];

        let vendor_actual: Vendor = lookup_vendor(&TREE, &macs[0]).unwrap();
        let vendor_expected = Vendor {
            short: "Hooc".to_string(),
            full: "Hooc Ag".to_string(),
        };
        assert_eq!(vendor_actual, vendor_expected);

        let vendor_actual = lookup_vendor(&TREE, &macs[1]).unwrap();
        let vendor_expected = Vendor {
            short: "OberixGroup".to_string(),
            full: "Oberix Group Pty Ltd".to_string(),
        };
        assert_eq!(vendor_actual, vendor_expected);

        let vendor_actual: Vendor = lookup_vendor(&TREE, &macs[2]).unwrap();
        let vendor_expected = Vendor {
            short: "HefeiFeierSm".to_string(),
            full: "Hefei Feier Smart Science&Technology Co. Ltd".to_string(),
        };
        assert_eq!(vendor_actual, vendor_expected);
    }

    #[test]
    fn test_mac_36() {
        let macs: Vec<MacAddress> = vec![
            MacAddress::try_from("40:D8:55:00:00:0A").unwrap(),
            MacAddress::try_from("40:D8:55:03:00:0A").unwrap(),
            MacAddress::try_from("40:D8:55:04:A0:0F").unwrap(),
        ];

        let vendor_actual: Vendor = lookup_vendor(&TREE, &macs[0]).unwrap();
        let vendor_expected = Vendor {
            short: "Xronos".to_string(),
            full: "Xronos.Inc".to_string(),
        };
        assert_eq!(vendor_actual, vendor_expected);

        let vendor_actual = lookup_vendor(&TREE, &macs[1]).unwrap();
        let vendor_expected = Vendor {
            short: "TecnologiasP".to_string(),
            full: "Tecnologias Plexus".to_string(),
        };
        assert_eq!(vendor_actual, vendor_expected);

        let vendor_actual: Vendor = lookup_vendor(&TREE, &macs[2]).unwrap();
        let vendor_expected = Vendor {
            short: "GatewayTechn".to_string(),
            full: "Gateway Technologies SA de CV".to_string(),
        };
        assert_eq!(vendor_actual, vendor_expected);
    }
}
