use std::collections::HashSet;
use std::fmt::Debug;
use std::io::Cursor;
use std::net::Ipv4Addr;

mod header;
mod option;
mod time;

use byteorder::NetworkEndian;
use byteorder::ReadBytesExt;
pub use option::DHCPMessageType;
pub use option::DHCPOption;
pub use time::LeaseTime;
pub use option::OptionParseErr;

use crate::buffer::ByteWriter;
use crate::error::DHCPError;
use crate::standard::MAGIC_COOKIE;

use self::header::MessageType;

#[derive(Debug, Clone)]
pub struct Packet {
    op: MessageType,
    htype: u8,
    hlen: u8,
    hops: u8,

    pub xid: u32,
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
    options: HashSet<DHCPOption>,

    /// Alle DHCP berichten zouden deze option moeten hebben
    pub dhcp_message_type: DHCPMessageType,
}

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
            options: HashSet::new(),
            dhcp_message_type,
        }
    }

    /// voegt een optie toe, als deze nog niet bestaat
    pub fn add_option(&mut self, option: DHCPOption) -> bool {
        self.options.insert(option)
    }

    /// voegt optie toe en overschrijft als deze er al is
    pub fn override_option(&mut self, option: DHCPOption) -> Option<DHCPOption> {
        self.options.replace(option)
    }

    pub fn options_cloned(&self) -> HashSet<DHCPOption> {
        self.options.clone()
    }

    pub fn get_requested_ip(&self) -> Option<Ipv4Addr> {
        self.options
            .get(&DHCPOption::RequestedIp(Ipv4Addr::new(0, 0, 0, 0)))
            .map(|ip| match ip {
                DHCPOption::RequestedIp(ip) => ip,
                _ => unreachable!(),
            })
            .copied()
    }

    pub fn get_leasetime(&self) -> Option<LeaseTime> {
        self.options
            .get(&DHCPOption::IpLeasetime(LeaseTime::Infinite))
            .map(|time| match time {
                DHCPOption::IpLeasetime(leasetime) => leasetime,
                _ => unreachable!(),
            })
            .copied()
    }

    pub fn print(&self) {
        // println!("op:\t\t\t{:?}", self.op);
        println!("dhcp_message_type:\t{:?}", self.dhcp_message_type);
        // println!("htype:\t\t\t{:?}", self.htype);
        // println!("hlen:\t\t\t{:?}", self.op);
        println!("xid:\t\t\t{:?}", self.xid);
        println!(
            "secs:\t\t\t{:?}",
            std::time::Duration::from_secs(self.secs as u64)
        );
        println!("flags:\t\t\t{:b}", self.flags);
        println!("ciaddr:\t\t\t{:?}", self.ciaddr);
        println!("yiaddr:\t\t\t{:?}", self.yiaddr);
        println!("siaddr:\t\t\t{:?}", self.siaddr);
        println!("giaddr:\t\t\t{:?}", self.giaddr);
        println!("chaddr: {:?}", self.chaddr);
        // println!("sname: {:?}", self.sname);
        // println!("file: {:?}", self.file);
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
    type Error = DHCPError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        let mut buffer = Cursor::new(buffer);

        let op = MessageType::try_from(buffer.read_u8()?).unwrap();
        let htype = buffer.read_u8()?;
        let hlen = buffer.read_u8()?;
        let hops = buffer.read_u8()?;

        let xid = buffer.read_u32::<NetworkEndian>()?;
        let secs = buffer.read_u16::<NetworkEndian>()?;

        let flags = buffer.read_u16::<NetworkEndian>()?;

        fn read_ipaddr(buffer: &mut Cursor<&[u8]>) -> Result<Ipv4Addr, DHCPError> {
            Ok(Ipv4Addr::from([
                buffer.read_u8()?,
                buffer.read_u8()?,
                buffer.read_u8()?,
                buffer.read_u8()?,
            ]))
        }

        let ciaddr = read_ipaddr(&mut buffer)?;
        let yiaddr = read_ipaddr(&mut buffer)?;
        let siaddr = read_ipaddr(&mut buffer)?;
        let giaddr = read_ipaddr(&mut buffer)?;

        let mut chaddr: [u8; 16] = [0; 16];
        chaddr.copy_from_slice(&buffer.get_ref()[28..44]);

        let mut sname = [0; 64];
        sname.copy_from_slice(&buffer.get_ref()[44..108]);

        let mut file = [0; 128];
        file.copy_from_slice(&buffer.get_ref()[108..236]);

        assert!(buffer.get_ref()[236..240] == MAGIC_COOKIE);

        let mut options = DHCPOption::from_bytes_many(&buffer.get_ref()[240..])?;

        let Some(dhcp_message_type) = options.iter().find_map(|opt| match opt {
            DHCPOption::DHCPMessageType(message_type) => Some(*message_type),
            _ => None,
        }) else {
            return Err(DHCPError::Protocol(
                "No DHCP message type in options".into(),
            ));
        };
        options.remove(&DHCPOption::DHCPMessageType(dhcp_message_type));

        Ok(Packet {
            op,
            htype,
            hlen,
            hops,
            xid,
            secs,
            flags,
            ciaddr,
            yiaddr,
            siaddr,
            giaddr,
            chaddr,
            sname,
            file,
            options,
            dhcp_message_type,
        })
    }
}
