use std::fmt::Debug;
use std::net::Ipv4Addr;

mod header;
mod option;
mod time;

pub use option::DHCPMessageType;
pub use option::DHCPOption;

use crate::buffer::ByteWriter;
use crate::standard::MAGIC_COOKIE;

use self::header::MessageType;

#[derive(Debug)]
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
    pub yiaddr: Ipv4Addr,
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

    /// Alle DHCP berichten zouden deze option moeten hebben
    pub dhcp_message_type: DHCPMessageType,
}

// impl Debug for Packet {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Packet")
//             .field("DHCP_message_type", &self.dhcp_message_type)
//             .field("op", &self.op)
//             .field("htype", &self.htype)
//             .field("hlen", &self.htype)
//             .field("hops", &self.hops)
//             .field("xid", &self.hops)
//             .field("secs", &self.secs)
//             .field("flags", &self.flags)
//             .field("ciaddr", &self.ciaddr)
//             .field("yiaddr", &self.yiaddr)
//             .field("siaddr", &self.siaddr)
//             .field("giaddr", &self.giaddr)
//             // .field("chaddr", &self.chaddr)
//             .field("options", &self.options)
//             .finish()
//     }
// }

impl Packet {
    pub fn new_request(dhcp_message_type: DHCPMessageType) -> Self {
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
            dhcp_message_type,
        }
    }

    pub fn print(&self) {
        println!("op:\t\t\t{:?}", self.op);
        println!("dhcp_message_type:\t{:?}", self.dhcp_message_type);
        println!("htype:\t\t\t{:?}", self.htype);
        println!("hlen:\t\t\t{:?}", self.op);
        println!("xid:\t\t\t{:?}", self.xid);
        println!("secs:\t\t\t{:?}", std::time::Duration::from_secs(self.secs as u64));
        println!("flags:\t\t\t{:b}", self.flags);
        println!("ciaddr:\t\t\t{:?}", self.ciaddr);
        println!("yiaddr:\t\t\t{:?}", self.yiaddr);
        println!("siaddr:\t\t\t{:?}", self.siaddr);
        println!("giaddr:\t\t\t{:?}", self.giaddr);
        println!("chaddr: {:?}", self.chaddr);
        println!("sname: {:?}", self.sname);
        println!("file: {:?}", self.file);
        println!("--- options ---");
        self.options.iter().for_each(|opt| println!("{opt:?}"));
    }

    pub fn into_response(&mut self, dhcp_message_type: DHCPMessageType) {
        self.op = MessageType::BOOTREPLY;
        self.options.clear();
        self.dhcp_message_type = dhcp_message_type;
    }

    pub fn is_broadcast(&self) -> bool {
        (self.flags & (1 << 15)) != 0
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
        buffer[240] = 53;
        buffer[241] = 1;
        buffer[242] = self.dhcp_message_type as u8;

        let mut len = 243;
        len += DHCPOption::to_bytes_many(&self.options, &mut buffer[len..]);
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

        let mut options = DHCPOption::from_bytes_many(&buffer[240..]).unwrap();

        let Some((i, dhcp_message_type)) = options.iter().enumerate().find_map(|(i, opt)| match opt {
            DHCPOption::DHCPMessageType(message_type) => Some((i,*message_type)),
            _ => None,
        }) else {
            return Err(());
        };
        options.swap_remove(i);


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
            options,
            dhcp_message_type,
        };
        Ok(packet)
    }
}
