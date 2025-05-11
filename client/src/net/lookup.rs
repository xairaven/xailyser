use dpi::analysis::ports::{PortInfo, PortServiceTable};
use dpi::analysis::vendor::OuiRadixTree;
use dpi::protocols::ethernet::mac::{MacAddress, Vendor};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Lookup {
    pub port_service: PortServiceTable,
    pub vendors: OuiRadixTree,
    pub vendors_amount: usize,
}

const PORTS_DATABASE_PATH: &str = "dpi/resources/iana-port-service-database.csv";
const OUI_DATABASE_PATH: &str = "dpi/resources/oui-database.txt";

impl Lookup {
    pub fn load() -> std::io::Result<Lookup> {
        let port_service =
            dpi::analysis::ports::read_database(PathBuf::from(PORTS_DATABASE_PATH))?;
        let (vendors, vendors_amount) =
            dpi::analysis::vendor::read_database(PathBuf::from(OUI_DATABASE_PATH))?;

        Ok(Self {
            port_service,
            vendors,
            vendors_amount,
        })
    }

    pub fn find_port(&self, port: &u16) -> Option<&Vec<PortInfo>> {
        self.port_service.get(port)
    }

    pub fn find_vendor(&self, mac: &MacAddress) -> Option<Vendor> {
        dpi::analysis::vendor::lookup_vendor(&self.vendors, mac)
    }
}
