#![allow(dead_code)]

use std::net::UdpSocket;

use packet::{DHCPOption, Packet};
mod buffer;
mod packet;

fn main() {
    let server = UdpSocket::bind("255.255.255.255:68").unwrap();
    // server.connect("192.168.255.255:67").unwrap();
    server.set_broadcast(true).unwrap();

    let mut packet = Packet::new_request();
    packet.options.push(DHCPOption::DHCPMessageType(
        packet::DHCPMessageType::DHCPDISCOVER,
    ));
    packet.options.push(DHCPOption::ParameterRequest {
        requested_options: [
            252, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 67, 66,
        ]
        .into(),
    });
    packet.options.push(DHCPOption::IpLeasetime { secs: 1 });

    let mut buff1  = [0; 4096];
    let len = packet.write_to_bytes(&mut buff1);
    // println!("bytes: {:?}\nlen: {}", &buff1[0..len], len);
    println!("sent: {}", server.send_to(&buff1[..len], "255.255.255.255:67").unwrap());


    let mut buff  = [0; 4096];
    // let server = UdpSocket::bind("255.255.255.255:67").unwrap();
    let (len, src) = server.recv_from(&mut buff).unwrap();
    // println!("bytes: {:?}\nlen: {}", &buff[0..len], len);
    let packet = Packet::try_from(&buff[..len]).unwrap();
    println!("len: {} src: {:?}", len, src);
    println!("packet: {:?}", packet);
}
