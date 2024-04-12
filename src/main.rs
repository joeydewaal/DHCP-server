#![allow(dead_code)]
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};

use chrono::Utc;
use packet::Packet;
use standard::CLIENT_PORT;

use crate::{
    packet::{DHCPMessageType, DHCPOption},
    standard::{BROADCAST_ADDR, SERVER_PORT},
};
mod buffer;
mod leases;
mod packet;
mod standard;

fn main() {
    let server = UdpSocket::bind((BROADCAST_ADDR, SERVER_PORT)).unwrap();
    server.set_broadcast(true).unwrap();

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

        let response = handle_packet(packet, src);

        let Some(resp) = response else {
            continue;
        };

        let len = resp.write_to_bytes(&mut buff);

        let mut response_addr = src.ip();
        if resp.is_broadcast() {
            response_addr = IpAddr::from(BROADCAST_ADDR);
        }
        // let _ = server
        //     .send_to(&buff[0..len], (response_addr, CLIENT_PORT))
        //     .unwrap();
    }
}

fn handle_packet(mut packet: Packet, _src: SocketAddr) -> Option<Packet> {
    println!("------ {:?}", Utc::now() + chrono::Duration::hours(2) );
    packet.print();
    println!();
    // println!(
    //     "DHCP message type: {:?} from: {:?}",
    //     packet.dhcp_message_type, src
    // );

    match packet.dhcp_message_type {
        DHCPMessageType::DHCPDISCOVER => {
            packet.into_response(DHCPMessageType::DHCPOFFER);
            packet.yiaddr = Ipv4Addr::new(192, 168, 0, 66);
            packet.options.push(packet::DHCPOption::Subnet (
                Ipv4Addr::new(255, 255, 255, 0),
            ));
            packet.options.push(DHCPOption::IpLeasetime (
                Duration::from_secs(99).into(),
            ));
            packet
                .options
                .push(DHCPOption::ServerIdentifier(Ipv4Addr::new(192, 168, 0, 15)));

            return Some(packet);
        }
        _ => (),
    }

    None
}
