use std::net::Ipv4Addr;

pub const CLIENT_PORT: u16 = 68;
pub const SERVER_PORT: u16 = 67;

pub const BROADCAST_ADDR: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 255);
pub const MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];
