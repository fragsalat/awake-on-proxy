use std::net::{TcpListener, TcpStream, Shutdown, SocketAddr};
use std::thread::{spawn, JoinHandle, sleep};
use std::io::{Read, Write};
use crate::config::ProxyMapping;
use std::time::Duration;

fn awake_target(mapping: &ProxyMapping) -> Result<TcpStream, String> {
    let magic = wakey::WolPacket::from_string(&mapping.mac_address, ':');

    magic.send_magic().map_err(|error| {
        format!("Failed to send Wake on Lan package to {}: {}", &mapping.target_address.to_string(), error)
    })?;

    info!("Awake sent, re-trying to connect to {}", &mapping.target_address.to_string());

    for _ in 0..mapping.awake_delay {
        match TcpStream::connect_timeout(&mapping.target_address, Duration::from_secs(1)) {
            Ok(stream) => return Ok(stream),
            Err(_) => {
                info!("System is not up yet, trying again soon");
                sleep(Duration::from_secs(1));
            }
        }
    }

    Err(format!("Could not connect to {} after awake", &mapping.target_address.to_string()))
}

fn pipe(incoming: &mut TcpStream, outgoing: &mut TcpStream) -> Result<(), String> {
    let mut buffer = [0; 1024];
    loop {
        match incoming.read(&mut buffer) {
            Ok(bytes_read) => {
                debug!("Read {} bytes", bytes_read);
                // Socket is disconnected => Shutdown the other socket as well
                if bytes_read == 0 {
                    outgoing.shutdown(Shutdown::Both);
                    break;
                }
                if outgoing.write(&buffer[..bytes_read]).is_ok() {
                    outgoing.flush();
                }
            },
            Err(error) => return Err(format!("Could not read data: {}", error))
        }
    }
    Ok(())
}

fn proxy_connection(mut incoming: TcpStream, mapping: &ProxyMapping) -> Result<(), String> {
    info!("Client connected from: {:#?}", incoming.peer_addr().unwrap().to_string());

    let mut outgoing = TcpStream::connect_timeout(&mapping.target_address, Duration::from_secs(2))
        .or_else(|error| {
            info!("Could not establish connection to {}: {}", mapping.target_address, error);
            awake_target(&mapping)
        })?;

    let mut incoming_clone = incoming.try_clone().map_err(|e| e.to_string())?;
    let mut outgoing_clone = outgoing.try_clone().map_err(|e| e.to_string())?;

    // Pipe for- and backward asynchronously
    let forward = spawn(move || pipe(&mut incoming, &mut outgoing));
    let backward = spawn(move || pipe(&mut outgoing_clone, &mut incoming_clone));

    info!("Proxying data...");
    forward.join().map_err(|_| format!("Forward thread failed"))?;
    backward.join().map_err(|_| format!("Backward thread failed"))?;

    info!("Socket closed");

    Ok(())
}

fn proxy(mapping: ProxyMapping) -> JoinHandle<()> {
    let local_address = SocketAddr::from(([0, 0, 0, 0], mapping.local_port));
    let listener = TcpListener::bind(local_address)
        .expect("Could not bind listener");
    info!("Proxying 0.0.0.0:{} -> {}", mapping.local_port, mapping.target_address.to_string());
    // One thread per port listener
    spawn(move || {
        for socket in listener.incoming() {
            let socket = match socket {
                Ok(s) => s,
                Err(error) => {
                    error!("Could not handle connection: {}", error);
                    return;
                }
            };
            let mapping =  mapping.clone();
            // One thread per connection
            spawn(move || {
                if let Err(error) = proxy_connection(socket, &mapping) {
                    error!("{}", error);
                }
            });
        }
    })
}

pub fn start_proxies(mappings: Vec<ProxyMapping>) {
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for mapping in mappings {
        &handles.push(proxy(mapping));
    }

    for handle in handles {
        handle.join();
    }
}
