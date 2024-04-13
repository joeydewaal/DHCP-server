#![allow(dead_code)]
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use chrono::Utc;
use leases::LeaseRange;
use packet::Packet;
use standard::CLIENT_PORT;

use crate::{
    handlers::{on_dhcp_discover, on_dhcp_request},
    packet::DHCPMessageType,
    standard::{BROADCAST_ADDR, SERVER_PORT},
};

mod buffer;
mod handlers;
mod leases;
mod packet;
mod standard;

fn main() {
    let server = UdpSocket::bind((BROADCAST_ADDR, SERVER_PORT)).unwrap();
    server.set_broadcast(true).unwrap();
    let mut server_state = LeaseRange::new(
        Ipv4Addr::new(192, 168, 56, 2),
        Ipv4Addr::new(192, 168, 56, 255),
    );
    loop {
        let mut buff = [0; 4096];
        let (len, src) = server.recv_from(&mut buff).unwrap();
        println!("source: {src:?}");

        let packet = match Packet::try_from(&buff[..len]) {
            Ok(packet) => packet,
            Err(error) => {
                println!("ERR: {error:?}");
                continue;
            }
        };

        let response = handle_packet(packet, src, &mut server_state);

        let Some(resp) = response else {
            continue;
        };

        let len = resp.write_to_bytes(&mut buff);

        let mut response_addr = src.ip();
        if resp.is_broadcast() {
            response_addr = IpAddr::from(BROADCAST_ADDR);
        }
        println!("sent: {response_addr:?}");
        let _ = server
            .send_to(&buff[0..len], (response_addr, CLIENT_PORT))
            .unwrap();
    }
}

fn handle_packet(packet: Packet, _src: SocketAddr, data: &mut LeaseRange) -> Option<Packet> {
    println!("------ {:?}", Utc::now() + chrono::Duration::hours(2));
    packet.print();
    println!();

    match packet.dhcp_message_type {
        DHCPMessageType::DHCPDISCOVER => return on_dhcp_discover(packet, data).ok(),
        DHCPMessageType::DHCPREQUEST => return on_dhcp_request(packet, data).ok(),
        _ => (),
    }

    None
}
