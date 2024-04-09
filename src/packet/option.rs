use crate::buffer::{ByteReader, ByteWriter};
use std::str;
use std::{net::Ipv4Addr, time::Duration};

const END_OPTION: u8 = 255;
const PAD_OPTION: u8 = 0;

/// https://datatracker.ietf.org/doc/html/rfc1533
#[derive(Debug)]
pub enum DHCPOption {
    Unimplemented {
        option_code: u8,
        len: u8,
    },
    /// option code 1
    Subnet {
        subnet: Ipv4Addr,
    },
    /// option code 3
    Router {
        routers: Vec<Ipv4Addr>,
    },
    /// option code 6
    DomainNameServer {
        dns_servers: Vec<Ipv4Addr>,
    },
    /// option 50
    RequestedIp(Ipv4Addr),
    /// option 51
    IpLeasetime {
        secs: u32,
    },
    /// option 52
    OptionOverload(OptionOverload),
    /// option 53
    DHCPMessageType(DHCPMessageType),
    /// option 54
    ServerIdentifier(Ipv4Addr),
    /// option 55
    ParameterRequest {
        requested_options: Vec<u8>,
    },
    /// option 56
    Message(String),
    /// option 57
    DHCPMessageSize(u16),
    /// option 58
    RenewalTime {
        secs: u32,
    },
    /// option 59
    RebindingTime {
        secs: u32,
    },
}

impl DHCPOption {
    pub fn from_bytes_many(buffer: &[u8]) -> Result<Vec<Self>, OptionParseErr> {
        let mut start_index_option = 0;

        let mut options: Vec<DHCPOption> = Vec::new();

        loop {
            let result = DHCPOption::from_bytes(&buffer[start_index_option..])?;
            println!("opt: {:?}", result);

            match result {
                OptionsParseResult::End => break,
                OptionsParseResult::Padding => start_index_option += 1,
                OptionsParseResult::Done(option, len) => {
                    options.push(option);
                    start_index_option += 2 + len;
                }
            }
        }

        Ok(options)
    }

    pub fn to_bytes_many(options: &[DHCPOption], buffer: &mut [u8]) -> usize {
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
            DHCPOption::Subnet { subnet } => {
                buffer.write_tag(1);
                buffer.write_len(4);
                buffer.write_slice(2, &subnet.octets());
                4
            }
            DHCPOption::Router { routers } => {
                buffer.write_tag(0);
                buffer.write_len((routers.len() as u8) * 4);
                routers
                    .iter()
                    .enumerate()
                    .for_each(|(i, ipv4)| buffer.write_slice(i * 4 + 2, &ipv4.octets()));
                routers.len() * 4
            }
            DHCPOption::DomainNameServer { dns_servers } => {
                buffer.write_tag(6);
                buffer.write_len((dns_servers.len() as u8) * 4);
                dns_servers
                    .iter()
                    .enumerate()
                    .for_each(|(i, ipv4)| buffer.write_slice(i * 4 + 2, &ipv4.octets()));

                dns_servers.len() * 4
            },
            DHCPOption::RequestedIp(ip) => {
                buffer.write_tag(50);
                buffer.write_len(4);
                buffer.write_slice(2, &ip.octets());
                4
            },
            DHCPOption::IpLeasetime { secs } => {
                buffer.write_tag(51);
                buffer.write_len(4);
                buffer.write_u32(2, *secs);
                4
            },
            DHCPOption::OptionOverload(overload) => {
                buffer.write_tag(52);
                buffer.write_len(1);
                buffer[2] = *overload as u8;
                1
            },
            DHCPOption::DHCPMessageType(message_type) => {
                buffer.write_tag(53);
                buffer.write_len(1);
                buffer[2] = *message_type as u8;
                1
            },
            DHCPOption::ParameterRequest { requested_options } => {
                buffer.write_tag(55);
                buffer.write_len(requested_options.len() as u8);
                buffer.write_slice(2, &requested_options);
                requested_options.len()
            },
            DHCPOption::Message(message) => {
                buffer.write_tag(56);
                buffer.write_len(message.len() as u8);
                buffer.write_slice(2, message.as_bytes());
                message.len()
            },
            DHCPOption::DHCPMessageSize(size) => {
                buffer.write_tag(57);
                buffer.write_len(2);
                buffer.write_u16(2, *size);
                2
            },
            DHCPOption::RenewalTime { secs } => {
                buffer.write_tag(58);
                buffer.write_len(4);
                buffer.write_u32(2, *secs);
                4
            },
            DHCPOption::RebindingTime { secs } => {
                buffer.write_tag(59);
                buffer.write_len(4);
                buffer.write_u32(2, *secs);
                4
            },
            _ => 0,
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
            1 => DHCPOption::Subnet {
                subnet: Ipv4Addr::from(bytes.read_u32(2)),
            },
            3 => DHCPOption::Router {
                routers: bytes.read_u32_many(2, len).map(Ipv4Addr::from).collect(),
            },
            6 => DHCPOption::DomainNameServer {
                dns_servers: bytes.read_u32_many(2, len).map(Ipv4Addr::from).collect(),
            },
            50 => DHCPOption::RequestedIp(Ipv4Addr::from(bytes.read_u32(2))),
            51 => DHCPOption::IpLeasetime {
                secs: bytes.read_u32(2),
            },
            52 => DHCPOption::OptionOverload(
                OptionOverload::try_from(bytes[2]).map_err(|_| OptionParseErr::OptionOverLoad)?,
            ),
            53 => DHCPOption::DHCPMessageType(
                DHCPMessageType::try_from(bytes[2]).map_err(|_| OptionParseErr::DHCPMessageType)?,
            ),
            55 => DHCPOption::ParameterRequest {
                requested_options: Vec::from(&bytes[2..(2 + len)]),
            },
            56 => DHCPOption::Message(
                str::from_utf8(&bytes[2..(2 + len)])
                    .map_err(|_| OptionParseErr::StringErr)?
                    .to_string(),
            ),
            57 => DHCPOption::DHCPMessageSize(bytes.read_u16(2)),
            58 => DHCPOption::RenewalTime {
                secs: bytes.read_u32(2),
            },
            59 => DHCPOption::RebindingTime {
                secs: bytes.read_u32(2),
            },
            option_code => DHCPOption::Unimplemented {
                option_code,
                len: len as u8,
            },
        };
        Ok(OptionsParseResult::Done(option, len))
    }
}

#[derive(Debug)]
pub enum LeaseTime {
    Infinite,
    Finite(Duration),
}

impl From<u32> for LeaseTime {
    fn from(value: u32) -> Self {
        match value {
            0xffffffff => LeaseTime::Infinite,
            secs => LeaseTime::Finite(Duration::from_secs(secs as u64)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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
