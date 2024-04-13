use crate::buffer::ByteReader;
use std::collections::HashSet;
use std::hash::Hash;
use std::net::Ipv4Addr;
use std::str;

use self::bytes::OptionToByte;

use super::time::LeaseTime;

mod bytes;

const END_OPTION: u8 = 255;
const PAD_OPTION: u8 = 0;

/// https://datatracker.ietf.org/doc/html/rfc1533
#[derive(Debug, Clone)]
pub enum DHCPOption {
    Unimplemented {
        option_code: u8,
        len: u8,
    },
    /// option code 1
    Subnet(Ipv4Addr),
    /// option code 3
    Router(Vec<Ipv4Addr>),
    /// option code 6
    DomainNameServer(Vec<Ipv4Addr>),
    /// option 12
    HostName(String),
    /// option 15
    DomainName(String),
    /// option 50
    RequestedIp(Ipv4Addr),
    /// option 51
    IpLeasetime(LeaseTime),
    /// option 52
    OptionOverload(OptionOverload),
    /// option 53
    DHCPMessageType(DHCPMessageType),
    /// option 54
    ServerIdentifier(Ipv4Addr),
    /// option 55
    ParameterRequest(Vec<u8>),
    /// option 56
    Message(String),
    /// option 57
    DHCPMessageSize(u16),
    /// option 58
    RenewalTime(LeaseTime),
    /// option 59
    RebindingTime(LeaseTime),
    /// option 60
    ClassIdentifier(Vec<u8>),
    /// option 61
    ClientIdentifier(Vec<u8>),
}

impl PartialEq for DHCPOption {
    fn eq(&self, other: &Self) -> bool {
        self.get_option_id() == other.get_option_id()
    }
}

impl Eq for DHCPOption { }

impl Hash for DHCPOption {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u8(self.get_option_id());
    }
}

impl DHCPOption {
    pub fn get_option_id(&self) -> u8 {
        match self {
            DHCPOption::Subnet(_) => 1,
            DHCPOption::Router(_) => 3,
            DHCPOption::DomainNameServer(_) => 6,
            DHCPOption::HostName(_) => 12,
            DHCPOption::DomainName(_) => 15,
            DHCPOption::RequestedIp(_) => 50,
            DHCPOption::IpLeasetime(_) => 51,
            DHCPOption::OptionOverload(_) => 52,
            DHCPOption::DHCPMessageType(_) => 53,
            DHCPOption::ServerIdentifier(_) => 54,
            DHCPOption::ParameterRequest(_) => 55,
            DHCPOption::Message(_) => 56,
            DHCPOption::DHCPMessageSize(_) => 57,
            DHCPOption::RenewalTime(_) => 58,
            DHCPOption::RebindingTime(_) => 59,
            DHCPOption::ClassIdentifier(_) => 60,
            DHCPOption::ClientIdentifier(_) => 61,
            DHCPOption::Unimplemented { option_code, len: _ } => *option_code
        }
    }

    pub fn from_bytes_many(buffer: &[u8]) -> Result<HashSet<Self>, OptionParseErr> {
        let mut start_index_option = 0;
        let mut options: HashSet<DHCPOption> = HashSet::new();

        loop {
            let result = DHCPOption::from_bytes(&buffer[start_index_option..])?;

            match result {
                OptionsParseResult::End => break,
                OptionsParseResult::Padding => start_index_option += 1,
                OptionsParseResult::Done(option, len) => {
                    options.insert(option);
                    start_index_option += 2 + len;
                }
            }
        }

        Ok(options)
    }

    pub fn to_bytes_many(options: &HashSet<DHCPOption>, buffer: &mut [u8]) -> usize {
        let mut index = 0;
        for option in options {
            index += option.write_bytes(&mut buffer[index..]) + 2; // len van data + tag + lengte
                                                                   // zelf
        }
        buffer[index] = END_OPTION;
        index += 1;
        index
    }

    fn write_bytes(&self, buffer: &mut [u8]) -> usize {
        match self {
            DHCPOption::Subnet(subnet) => subnet.write_option_bytes(1, buffer),
            DHCPOption::Router(routers) => routers.write_option_bytes(3, buffer),
            DHCPOption::DomainNameServer(dns_servers) => dns_servers.write_option_bytes(6, buffer),
            DHCPOption::HostName(hostname) => hostname.write_option_bytes(12, buffer),
            DHCPOption::DomainName(domainname) => domainname.write_option_bytes(15, buffer),
            DHCPOption::RequestedIp(ip) => ip.write_option_bytes(50, buffer),
            DHCPOption::IpLeasetime(secs) => secs.write_option_bytes(51, buffer),
            DHCPOption::OptionOverload(overload) => overload.write_option_bytes(52, buffer),
            DHCPOption::DHCPMessageType(message_type) => {
                message_type.write_option_bytes(53, buffer)
            }
            DHCPOption::ServerIdentifier(ip) => ip.write_option_bytes(54, buffer),
            DHCPOption::ParameterRequest(requested_options) => {
                requested_options.write_option_bytes(55, buffer)
            }
            DHCPOption::Message(message) => message.write_option_bytes(56, buffer),
            DHCPOption::DHCPMessageSize(size) => size.write_option_bytes(57, buffer),
            DHCPOption::RenewalTime(secs) => secs.write_option_bytes(58, buffer),
            DHCPOption::RebindingTime(secs) => secs.write_option_bytes(59, buffer),
            DHCPOption::ClientIdentifier(id) => id.write_option_bytes(60, buffer),
            DHCPOption::ClassIdentifier(id) => id.write_option_bytes(61, buffer),
            DHCPOption::Unimplemented {
                option_code: _,
                len: _,
            } => 0,
        }
    }
}

#[derive(Debug)]
pub enum OptionsParseResult {
    Padding,
    End,
    Done(DHCPOption, usize),
}

#[derive(Debug)]
pub enum OptionParseErr {
    OptionOverLoad,
    DHCPMessageType,
    StringErr,
}

impl From<std::str::Utf8Error> for OptionParseErr {
    fn from(_value: std::str::Utf8Error) -> Self {
        OptionParseErr::StringErr
    }
}

impl DHCPOption {
    fn from_bytes(bytes: &[u8]) -> Result<OptionsParseResult, OptionParseErr> {
        let tag = bytes[0];

        if tag == END_OPTION {
            return Ok(OptionsParseResult::End);
        } else if tag == PAD_OPTION {
            return Ok(OptionsParseResult::Padding);
        }

        let len = bytes[1] as usize;

        let option = match tag {
            1 => DHCPOption::Subnet(Ipv4Addr::from(bytes.read_u32(2))),
            3 => DHCPOption::Router(
                bytes
                    .read_u32_many(2, len / 4)
                    .map(Ipv4Addr::from)
                    .collect(),
            ),
            6 => DHCPOption::DomainNameServer(
                bytes
                    .read_u32_many(2, len / 4)
                    .map(Ipv4Addr::from)
                    .collect(),
            ),
            12 => DHCPOption::HostName(str::from_utf8(&bytes[2..(2 + len)])?.to_string()),
            15 => DHCPOption::DomainName(str::from_utf8(&bytes[2..(2 + len)])?.to_string()),
            50 => DHCPOption::RequestedIp(Ipv4Addr::from(bytes.read_u32(2))),
            51 => DHCPOption::IpLeasetime(bytes.read_u32(2).into()),
            52 => DHCPOption::OptionOverload(
                OptionOverload::try_from(bytes[2]).map_err(|_| OptionParseErr::OptionOverLoad)?,
            ),
            53 => DHCPOption::DHCPMessageType(
                DHCPMessageType::try_from(bytes[2]).map_err(|_| OptionParseErr::DHCPMessageType)?,
            ),
            54 => DHCPOption::ServerIdentifier(Ipv4Addr::from(bytes.read_u32(2))),
            55 => DHCPOption::ParameterRequest(Vec::from(&bytes[2..(2 + len)])),
            56 => DHCPOption::Message(str::from_utf8(&bytes[2..(2 + len)])?.to_string()),
            57 => DHCPOption::DHCPMessageSize(bytes.read_u16(2)),
            58 => DHCPOption::RenewalTime(bytes.read_u32(2).into()),
            59 => DHCPOption::RebindingTime(bytes.read_u32(2).into()),
            60 => DHCPOption::ClassIdentifier(bytes[2..(2 + len)].to_vec()),
            61 => DHCPOption::ClientIdentifier(bytes[2..(2 + len)].to_vec()),
            option_code => DHCPOption::Unimplemented {
                option_code,
                len: len as u8,
            },
        };
        Ok(OptionsParseResult::Done(option, len))
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DHCPMessageType {
    DHCPDISCOVER = 1,
    DHCPOFFER = 2,
    DHCPREQUEST = 3,
    DHCPDECLINE = 4,
    DHCPACK = 5,
    DHCPNAK = 6,
    DHCPRELEASE = 7,
}

impl TryFrom<u8> for DHCPMessageType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => DHCPMessageType::DHCPDISCOVER,
            2 => DHCPMessageType::DHCPOFFER,
            3 => DHCPMessageType::DHCPREQUEST,
            4 => DHCPMessageType::DHCPDECLINE,
            5 => DHCPMessageType::DHCPACK,
            6 => DHCPMessageType::DHCPNAK,
            7 => DHCPMessageType::DHCPRELEASE,
            _ => return Err(()),
        })
    }
}

impl From<DHCPMessageType> for u8 {
    fn from(value: DHCPMessageType) -> Self {
        value as u8
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum OptionOverload {
    File = 1,
    Sname = 2,
    Both = 3,
}

impl TryFrom<u8> for OptionOverload {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => OptionOverload::File,
            2 => OptionOverload::Sname,
            3 => OptionOverload::Both,
            _ => return Err(()),
        })
    }
}

impl From<OptionOverload> for u8 {
    fn from(value: OptionOverload) -> Self {
        value as u8
    }
}


#[cfg(test)]
mod test {
    use std::{collections::HashSet, net::Ipv4Addr};

    use super::DHCPOption;

    #[test]
    fn testing() {
        let ip1 = Ipv4Addr::new(192, 168, 0, 1);
        let ip2 = Ipv4Addr::new(192, 168, 0, 2);

        let opt1 = DHCPOption::ServerIdentifier(ip1);
        let opt2 = DHCPOption::ServerIdentifier(ip2);

        let mut set: HashSet<DHCPOption> = HashSet::new();
        set.insert(DHCPOption::ServerIdentifier(ip1));

        assert!(!set.insert(DHCPOption::ServerIdentifier(ip2)));

        assert!(set.insert(DHCPOption::Subnet(ip1)));
        assert!(!set.insert(DHCPOption::Subnet(ip2)));
    }
}
