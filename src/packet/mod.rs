use std::fmt::Debug;
use std::net::Ipv4Addr;

mod option;

pub use option::DHCPOption;
pub use option::DHCPMessageType;

use crate::buffer::ByteWriter;

const MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];

// #[derive(Debug)]
pub struct Packet {
    op: MessageType,
    htype: u8,
    hlen: u8,
    hops: u8,

    xid: u32,
    secs: u16,
    /// als 1e bit 1 is moet dit terug gestuurd worden als broadcast bericht
    /// Als deze 0 is dan moet dit bericht terug gestuurd worden naar het aders in yiaddr
    flags: u16,
    ciaddr: Ipv4Addr,

    /// 'your' (client) IP address.
    yiaddr: Ipv4Addr,
    /// IP address of next server to use in bootstrap;
    /// returned in DHCPOFFER, DHCPACK by server.
    siaddr: Ipv4Addr,
    /// Relay agent IP address, used in booting via a relay agent.
    giaddr: Ipv4Addr,
    /// Client hardware address
    chaddr: [u8; 16],
    /// Optional server host name, null terminated string.
    sname: [u8; 64],
    /// Boot file name, null terminated string; "generic"
    /// name or null in DHCPDISCOVER, fully qualified
    file: [u8; 128],
    pub options: Vec<DHCPOption>,
}

impl Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Packet")
            .field("op", &self.op)
            .field("htype", &self.htype)
            .field("hlen", &self.htype)
            .field("hops", &self.hops)
            .field("xid", &self.hops)
            .field("secs", &self.secs)
            .field("flags", &self.flags)
            .field("ciaddr", &self.ciaddr)
            .field("yiaddr", &self.yiaddr)
            .field("siaddr", &self.siaddr)
            .field("giaddr", &self.giaddr)
            .field("chaddr", &self.chaddr)
            .field("options", &self.options)
            .finish()
    }
}

impl Packet {
    pub fn new_request() -> Self {
        Self {
            op: MessageType::BOOTREQUEST,
            htype: 1,
            hlen: 6,
            hops: 0,
            xid: 666,
            secs: 128,
            flags: 1 << 15,
            ciaddr: Ipv4Addr::from([0, 0, 0, 0]),
            yiaddr: Ipv4Addr::from([0, 0, 0, 0]),
            siaddr: Ipv4Addr::from([0, 0, 0, 0]),
            giaddr: Ipv4Addr::from([0, 0, 0, 0]),
            chaddr: [222, 173, 192, 222, 202, 254, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            sname: [0; 64],
            file: [0; 128],
            options: Vec::new(),
        }
    }

    pub fn write_to_bytes(&self, buffer: &mut [u8]) -> usize {
        buffer[0] = self.op.into();
        buffer[1] = self.htype;
        buffer[2] = self.hlen;
        buffer[3] = self.hops;
        buffer.write_u32(4, self.xid);
        buffer.write_u16(8, self.secs);
        buffer.write_u16(10, self.flags);
        buffer.write_slice(12, &self.ciaddr.octets());
        buffer.write_slice(16, &self.yiaddr.octets());
        buffer.write_slice(20, &self.siaddr.octets());
        buffer.write_slice(24, &self.giaddr.octets());
        buffer.write_slice(28, &self.chaddr);
        buffer.write_slice(44, &self.sname);
        buffer.write_slice(108, &self.file);
        buffer.write_slice(236, &MAGIC_COOKIE);

        let mut len = 240;
        len += DHCPOption::to_bytes_many(&self.options, &mut buffer[240..]);
        len
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = ();
    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        let mut chaddr: [u8; 16] = [0; 16];
        chaddr.copy_from_slice(&buffer[28..44]);

        let mut sname = [0; 64];
        sname.copy_from_slice(&buffer[44..108]);

        let mut file = [0; 128];
        file.copy_from_slice(&buffer[108..236]);

        assert!(buffer[236..240] == MAGIC_COOKIE);

        // println!("rest: {:?}", &buffer[240..]);

        let packet = Packet {
            op: MessageType::try_from(buffer[0]).unwrap(),
            htype: buffer[1],
            hlen: buffer[2],
            hops: buffer[3],
            xid: u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            secs: u16::from_be_bytes([buffer[8], buffer[9]]),
            flags: u16::from_be_bytes([buffer[10], buffer[11]]),
            ciaddr: Ipv4Addr::from([buffer[12], buffer[13], buffer[14], buffer[15]]),
            yiaddr: Ipv4Addr::from([buffer[16], buffer[17], buffer[18], buffer[19]]),
            siaddr: Ipv4Addr::from([buffer[20], buffer[21], buffer[22], buffer[23]]),
            giaddr: Ipv4Addr::from([buffer[24], buffer[25], buffer[26], buffer[27]]),
            chaddr,
            sname,
            file,
            options: DHCPOption::from_bytes_many(&buffer[240..]).unwrap(),
        };
        Ok(packet)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    BOOTREQUEST = 1,
    BOOTREPLY = 2,
}

impl TryFrom<u8> for MessageType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => MessageType::BOOTREQUEST,
            2 => MessageType::BOOTREPLY,
            _ => return Err(()),
        })
    }
}

impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::BOOTREQUEST => 1,
            MessageType::BOOTREPLY => 2
        }
    }
}
