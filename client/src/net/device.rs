use crate::errors::FileError;
use common::io::FileKind;
use dpi::protocols::ethernet::mac::{MacAddress, Vendor};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

const ALIASES_FILENAME: &str = "device_aliases.toml";
const ALIASES_FILETYPE: FileKind = FileKind::Data;
type DeviceAliases = HashMap<MacAddress, String>;

#[derive(Default)]
pub struct DeviceStorage {
    pub list: Vec<LocalDevice>,
    pub aliases: DeviceAliases,
}

impl DeviceStorage {
    pub fn find_by_mac(&mut self, mac: &MacAddress) -> Option<&mut LocalDevice> {
        self.list.iter_mut().find(|dev| dev.mac.eq(mac))
    }

    pub fn from_file() -> Result<Self, FileError> {
        match common::io::get_storage_file_path(ALIASES_FILENAME, ALIASES_FILETYPE) {
            Ok(path) => {
                let data = std::fs::read_to_string(path);
                if data.is_err() {
                    let storage = DeviceStorage::default();
                    storage.save_aliases_to_file()?;
                    return Ok(storage);
                }

                let mut aliases = HashMap::new();
                let raw_map: std::collections::BTreeMap<String, String> =
                    toml::from_str(&data.unwrap_or_default())
                        .map_err(FileError::TomlDeserialization)?;
                for (key, value) in raw_map {
                    let mac = MacAddress::try_from(key.as_str()).map_err(|_| {
                        FileError::TomlDeserialization(serde::de::Error::missing_field(
                            "MAC",
                        ))
                    })?;
                    aliases.insert(mac, value);
                }
                Ok(DeviceStorage {
                    list: Default::default(),
                    aliases,
                })
            },
            Err(_) => Ok(DeviceStorage::default()),
        }
    }

    pub fn save_aliases_to_file(&self) -> Result<(), FileError> {
        let string_map: std::collections::BTreeMap<String, String> = self
            .aliases
            .iter()
            .map(|(mac, alias)| (mac.to_string(), alias.clone()))
            .collect();

        let data = toml::to_string(&string_map).map_err(FileError::TomlSerialization)?;

        let path = common::io::get_storage_file_path(ALIASES_FILENAME, ALIASES_FILETYPE)?;
        common::io::create_parent_directories(&path)?;
        std::fs::write(path, data)?;

        Ok(())
    }
}

pub struct LocalDevice {
    pub mac: MacAddress,
    pub ip: Vec<Ipv4Addr>,
    pub ipv6: Vec<Ipv6Addr>,
    pub vendor: Option<Vendor>,
}
