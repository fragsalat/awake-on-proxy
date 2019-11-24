#[macro_use]
extern crate log;
use env_logger::Env;
use crate::config::Config;
use crate::proxy::start_proxy;

mod config;
mod proxy;

fn main() {
    env_logger
    ::from_env(Env::default().default_filter_or("INFO".to_string()))
        .init();

    let config = Config::from_file();

    match start_proxy(config) {
        Err(error) => error!("Could not start proxy: {}", error),
        Ok(threads) => {
            info!("Proxy started");
            for thread in threads {
                thread.join().unwrap();
            }
        },
    }

    info!("Hello, world!");
}

/*
[[mappings]]
local_port = 9981
target = "nas:9981"
max_address = "b4:2e:99:82:34:5a"
awake_delay = 10

[[mappings]]

*/