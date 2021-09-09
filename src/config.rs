extern crate serde;

use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use serde::Deserialize;


#[derive(Deserialize, Clone)]
pub struct ProxyMapping {
    pub local_port: u16,
    pub target_address: SocketAddr,
    pub mac_address: String,
    pub awake_delay: u64
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub mappings: Vec<ProxyMapping>
}

impl Config {
    pub fn from_file() -> Self {
        let mut config_file = File::open("./assets/config.json")
            .or(File::open("/etc/awake-on-proxy/config.json"))
            .expect("Could not find /etc/awake-on-proxy/config.json or ./asstes/config.json");

        let mut config = String::new();
        config_file.read_to_string(&mut config)
            .expect("Could not read config file");

        serde_json::from_str(&config).expect("Could not parse config file")
    }
}
