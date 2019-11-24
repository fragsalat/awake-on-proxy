use crate::config::{Config, ProxyMapping};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use std::thread;
use std::io::{BufReader, Read, Write, BufWriter};

fn awake_target(mapping: &ProxyMapping) -> Result<TcpStream, String> {
    let magic = wakey::WolPacket::from_string(&mapping.mac_address, ':');

    magic.send_magic().map_err(|error| {
        format!("Failed to send Wake on Lan package to {}: {}", &mapping.target, error)
    })?;

    info!("Awake sent, re-trying to connect to {}", &mapping.target);

    for _ in 0..mapping.awake_delay {
        match TcpStream::connect_timeout(&mapping.get_target_addr()?, Duration::from_secs(1)) {
            Ok(stream) => return Ok(stream),
            Err(_) => {
                debug!("System is not up yet, trying again soon");
                sleep(Duration::from_secs(1));
            }
        }
    }

    Err(format!("Could not connect to {} after awake", &mapping.target))
}

fn forward(direction: &str, input: &mut BufReader<TcpStream>, output: &mut BufWriter<TcpStream>) {
    loop {
        let mut buffer = [0;1024];
        debug!("{} Reading", direction);
        match input.read(&mut buffer) {
            Ok(bytes) => {
                debug!("Read {} bytes", bytes);
                debug!("{}", buffer.iter().map(|b| format!("{:x?}", b)).collect::<Vec<String>>().join(""));
                if bytes < 1  {
                    break;
                }
                match output.write_all(&mut buffer) {
                    Ok(_) => {
                        debug!("Forwarded {:#?} bytes", bytes);
                        output.flush();
                        if bytes < 1024 {
                            break;
                        }
                    },
                    Err(error) => panic!("Could not forward: {}", error)
                }
            },
            Err(error) => panic!("Could not read: {}", error)
        }
    }
}

fn sync(input: TcpStream, output: TcpStream) -> Result<(), String> {
    Ok(())
}

fn handle_connection(mapping: ProxyMapping, incoming: TcpStream) -> Result<(), String> {
    info!("Incomming connection from {}", incoming.peer_addr().map_err(|e| format!("{}", e))?);
    // Try to connect to target
    let outgoing = TcpStream::connect_timeout(
        &mapping.get_target_addr()?,
        Duration::from_secs(1)
    )
        .or_else(|_| {
            info!("Can not connect to target {}, trying to awake it", &mapping.target);
            awake_target(&mapping)
        })?;


    // forward tcp steam
    debug!("Start sync");
    let incoming_clone = incoming.try_clone()
        .map_err(|error| format!("Couldn't clone {}", error))?;
    let mut input_read = BufReader::new(incoming);
    let mut input_write = BufWriter::new(incoming_clone);

    let outgoing_clone = outgoing.try_clone()
        .map_err(|error| format!("Couldn't clone {}", error))?;
    let mut output_read = BufReader::new(outgoing);
    let mut output_write = BufWriter::new(outgoing_clone);
    debug!("spawn sync");

    loop {
        debug!("Forward");
        forward("forward", &mut input_read, &mut output_write);
        debug!("Backward");
        forward("backward",&mut output_read, &mut input_write);

        break;
    }

    Ok(())
}

pub fn start_proxy(config: Config) -> Result<Vec<JoinHandle<()>>, String> {
    let mut threads = Vec::new();
    for mapping in config.mappings {
        let local_addr = format!("0.0.0.0:{}", mapping.local_port);
        let addr = local_addr.parse::<SocketAddr>()
            .expect("Could not parse local address");
        let listener = TcpListener::bind(&addr)
            .map_err(|error|
                format!("Could not bind to {}: {:#?}", local_addr, error.to_string())
            )?;

        info!("Proxying 0.0.0.0:{} to {}", mapping.local_port, mapping.target);

        threads.push(thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let map = mapping.clone();
                        thread::spawn(|| {
                            if let Err(error) = handle_connection(map, stream) {
                                error!("{}", error);
                            }
                        });
                    },
                    Err(error) => error!("Connection failure: {}", error)
                }
            }
        }));
    }
    Ok(threads)
}