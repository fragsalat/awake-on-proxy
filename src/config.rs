use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ProxyMapping {
    pub local_port: u16,
    pub target: String,
    pub mac_address: String,
    pub awake_delay: u64
}

impl ProxyMapping {
    pub fn get_target_addr(&self) -> Result<SocketAddr, String> {
        self.target.parse::<SocketAddr>().map_err(|error|
            format!("Could not parse address {}: {}", self.target, error)
        )
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub mappings: Vec<ProxyMapping>
}

impl Config {
    pub fn from_file() -> Self {
        let mut config_file = File::open("./config.json")
            .or(File::open("/etc/wake-proxy/config.json"))
            .expect("Could not find /etc/wake-proxy/config.json or ./proxy-config.json");

        let mut config = String::new();
        config_file.read_to_string(&mut config)
            .expect("Could not read config file");

        let r: Config = serde_json::from_str(&config).expect("Could not parse config file");
        r
    }
}
