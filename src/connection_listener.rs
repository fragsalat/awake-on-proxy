use pnet::datalink::{interfaces, channel, Channel::Ethernet};
use pnet::packet::{Packet, MutablePacket};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use crate::config::Config;

fn handle_packet(config: &Config, data: &[u8]) {
    let ipPacket = Ipv4Packet::new(data).unwrap();
    let tcpPacket = TcpPacket::new(data).unwrap();
    let ethPacket = EthernetPacket::new(data).unwrap();
    let destination = ethPacket.get_destination();

    let sockAddr = SocketAddr::new(IpAddr::V4(ipPacket.get_destination()), tcpPacket.get_destination());

    let mapping = config.mappings
        .iter()
        .find(|mapping| {
            info!("{:#?} equals {:#?} = {}", mapping.target, sockAddr, mapping.target.eq(&sockAddr));
            mapping.target.eq(&sockAddr)
        });

    if let Some(mapping) = mapping {
        info!("Packet sent to {:#?} {:#?}:{} -> {:#?}:{:#?}", &destination, &ipPacket.get_source(), &tcpPacket.get_source(), &ipPacket.get_destination(), &tcpPacket.get_destination());
    }

//    if config.mappings.iter().any(|mapping| mapping.mac_address.eq(&destination)) {
//        info!("Packet sent to {:#?} {:#?}:{} -> {:#?}:{:#?}", &destination, &ipPacket.get_source(), &tcpPacket.get_source(), &ipPacket.get_destination(), &tcpPacket.get_destination());
//    }
//
//    let mapping = config.mappings
//        .iter()
//        .find(|mapping| mapping.mac_address.eq(destination));
}

pub fn start_listener(config: Config) -> Result<(), String> {
    let interface = interfaces().into_iter().find(|interface| {
        interface.ips.iter().any(|ip|
            ip.ip().ne(&Ipv4Addr::new(0, 0, 0, 0))
        )
    }).ok_or("Could not determine interface")?;

    let (mut tx, mut rx) = match channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unexpected channel type"),
        Err(error) => return Err(format!("Could not create channel: {}", error))
    };

    info!("Listening for packets on {:#?}", &interface);
    loop {
        match rx.next() {
            Ok(data) => handle_packet(&config, &data),
            Err(error) => error!("Failed to read packet: {}", error)
        }
    }
}