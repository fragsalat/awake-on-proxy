#[macro_use]
extern crate log;
use log4rs::init_file;
use crate::config::Config;
use crate::proxy::start_proxies;

mod config;
mod proxy;

fn main() {
    init_file("./assets/log4rs.yaml", Default::default())
        .or_else(|_| init_file("/etc/awake-on-proxy/log4rs.yaml", Default::default()))
        .unwrap();

    info!("Starting proxy server");

    let config = Config::from_file();

    start_proxies(config.mappings);
}
