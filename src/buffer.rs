pub trait ByteReader {
    fn read_u32(&self, start: usize) -> u32;
    fn read_u32_many(&self, start: usize, len: usize) -> impl Iterator<Item = u32>;
    fn read_u16(&self, start: usize) -> u16;
    // fn read<T>(&self, start: usize) -> T;
}

impl ByteReader for &[u8] {
    fn read_u32(&self, start: usize) -> u32 {
        u32::from_be_bytes([
            self[start],
            self[start + 1],
            self[start + 2],
            self[start + 3],
        ])
    }

    fn read_u32_many(&self, start: usize, len: usize) -> impl Iterator<Item = u32> {
        (start..(len * 4)).step_by(4).map(|i| self.read_u32(i))
    }

    fn read_u16(&self, start: usize) -> u16 {
        u16::from_be_bytes([self[start], self[start + 1]])
    }
}

pub trait ByteWriter {
    fn write_u32(&mut self, start: usize, value: u32);
    fn write_u16(&mut self, start: usize, value: u16);
    fn write_slice(&mut self, start: usize, value: &[u8]);
    fn write_tag(&mut self, tag: u8);
    fn write_len(&mut self, len: u8);
}

impl ByteWriter for [u8] {
    fn write_u32(&mut self, start: usize, value: u32) {
        let bytes = value.to_be_bytes();
        self[start..(start + 4)].copy_from_slice(&bytes);
    }

    fn write_u16(&mut self, start: usize, value: u16) {
        let bytes = value.to_be_bytes();
        self[start..(start + 2)].copy_from_slice(&bytes);
    }

    fn write_slice(&mut self, start: usize, value: &[u8]) {
        self[start..(start + value.len())].copy_from_slice(value);
    }

    fn write_tag(&mut self, tag: u8) {
        self[0] = tag;
    }

    fn write_len(&mut self, len: u8) {
        self[1] = len;
    }
}
