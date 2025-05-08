use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PortInfo {
    pub port: Port,
    pub service_name: String,
    pub transport_protocol: String,
    pub description: String,
}

pub type Port = u16;
pub type PortServiceTable = HashMap<Port, Vec<PortInfo>>;

pub fn read_database(path: PathBuf) -> io::Result<PortServiceTable> {
    let file = std::fs::File::open(path)?;
    let mut reader = csv::ReaderBuilder::new().from_reader(io::BufReader::new(file));

    let mut map = PortServiceTable::new();

    const CSV_FIELDS: usize = 12;
    const INDEX_SERVICE: usize = 0;
    const INDEX_PORT: usize = 1;
    const INDEX_TRANSPORT_PROTOCOL: usize = 2;
    const INDEX_DESCRIPTION: usize = 3;
    for result in reader.records() {
        let record = result?;
        debug_assert!(record.len() == CSV_FIELDS);

        let port = match record.get(INDEX_PORT) {
            Some(value) => match value.parse::<u16>() {
                Ok(value) => value,
                Err(_) => continue,
            },
            None => continue,
        };

        let service_name = match record.get(INDEX_SERVICE) {
            Some(value) => value.to_string(),
            None => "".to_string(),
        };
        let transport_protocol = match record.get(INDEX_TRANSPORT_PROTOCOL) {
            Some(value) => value.to_string(),
            None => "".to_string(),
        };
        let description = match record.get(INDEX_DESCRIPTION) {
            Some(value) => value.to_string(),
            None => "".to_string(),
        };

        let port_info = PortInfo {
            port,
            service_name,
            transport_protocol,
            description,
        };
        map.entry(port).or_default().push(port_info);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    const PATH: &str = "./resources/iana-port-service-database.csv";
    static TABLE: LazyLock<PortServiceTable> =
        LazyLock::new(|| read_database(PathBuf::from(PATH)).unwrap());

    #[test]
    fn test_kerberos() {
        const PORT: u16 = 88;

        let actual_port_info = TABLE.get(&PORT).unwrap();
        let expected_port_info = vec![
            PortInfo {
                port: 88,
                service_name: "kerberos".to_string(),
                transport_protocol: "tcp".to_string(),
                description: "Kerberos".to_string(),
            },
            PortInfo {
                port: 88,
                service_name: "kerberos".to_string(),
                transport_protocol: "udp".to_string(),
                description: "Kerberos".to_string(),
            },
        ];

        assert_eq!(actual_port_info, &expected_port_info);
    }

    #[test]
    fn test_http() {
        const PORT: u16 = 80;

        let actual_port_info = TABLE.get(&PORT).unwrap();
        let expected_port_info = vec![
            PortInfo {
                port: 80,
                service_name: "http".to_string(),
                transport_protocol: "tcp".to_string(),
                description: "World Wide Web HTTP".to_string(),
            },
            PortInfo {
                port: 80,
                service_name: "http".to_string(),
                transport_protocol: "udp".to_string(),
                description: "World Wide Web HTTP".to_string(),
            },
            PortInfo {
                port: 80,
                service_name: "www".to_string(),
                transport_protocol: "tcp".to_string(),
                description: "World Wide Web HTTP".to_string(),
            },
            PortInfo {
                port: 80,
                service_name: "www".to_string(),
                transport_protocol: "udp".to_string(),
                description: "World Wide Web HTTP".to_string(),
            },
            PortInfo {
                port: 80,
                service_name: "www-http".to_string(),
                transport_protocol: "tcp".to_string(),
                description: "World Wide Web HTTP".to_string(),
            },
            PortInfo {
                port: 80,
                service_name: "www-http".to_string(),
                transport_protocol: "udp".to_string(),
                description: "World Wide Web HTTP".to_string(),
            },
            PortInfo {
                port: 80,
                service_name: "http".to_string(),
                transport_protocol: "sctp".to_string(),
                description: "HTTP".to_string(),
            },
        ];

        assert_eq!(actual_port_info, &expected_port_info);
    }
}
