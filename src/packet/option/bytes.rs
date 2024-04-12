use std::net::Ipv4Addr;

use crate::{buffer::ByteWriter, packet::time::LeaseTime};

use super::{DHCPMessageType, OptionOverload};

pub trait OptionToByte {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize;
}

impl OptionToByte for Ipv4Addr {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize {
        buffer.write_tag(tag);
        buffer.write_len(4);
        buffer.write_slice(2, &self.octets());
        4
    }
}


impl OptionToByte for Vec<Ipv4Addr> {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize {
        buffer.write_tag(tag);
        buffer.write_len(self.len() as u8 * 4);
        self
            .iter()
            .enumerate()
            .for_each(|(i, ipv4)| buffer.write_slice(i * 4 + 2, &ipv4.octets()));
        self.len() * 4
    }
}

impl OptionToByte for String {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize {
        buffer.write_tag(tag);
        let bytes = self.as_bytes();
        buffer.write_len(bytes.len() as u8);
        buffer.write_slice(2, &bytes);
        bytes.len()
    }
}

impl OptionToByte for LeaseTime {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize {
        buffer.write_tag(tag);
        buffer.write_len(4);
        buffer.write_u32(2, self.to_bytes());
        4
    }
}

impl OptionToByte for OptionOverload {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize {
        buffer.write_tag(tag);
        buffer.write_len(1);
        buffer[2] = (*self).into();
        1
    }
}

impl OptionToByte for DHCPMessageType {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize {
        buffer.write_tag(tag);
        buffer.write_len(1);
        buffer[2] = (*self).into();
        1
    }
}

impl OptionToByte for Vec<u8> {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize {
        buffer.write_tag(tag);
        buffer.write_len(self.len() as u8);
        buffer.write_slice(2, &self);
        self.len()
    }
}

impl OptionToByte for u16 {
    fn write_option_bytes(&self, tag: u8, buffer: &mut [u8]) -> usize {
        buffer.write_tag(tag);
        buffer.write_len(2);
        buffer.write_u16(2, *self);
        2
    }
}
