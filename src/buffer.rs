use byteorder::{BigEndian, ReadBytesExt};

pub struct ByteBuffer {
    buffer: Vec<u8>,
    position: usize,
}

impl ByteBuffer {
    pub fn new(buffer: Vec<u8>) -> Self {
        ByteBuffer { buffer, position: 0 }
    }

    pub fn read_bytes(&mut self, count: usize) -> &[u8] {
        let start = self.position;
        let end = start + count;
        if end > self.buffer.len() {
            panic!("Attempt to read beyond buffer length");
        }
        self.position = end;
        &self.buffer[start..end]
    }

    pub fn skip_bytes(&mut self, count: usize) {
        let end = self.position + count;
        if end > self.buffer.len() {
            panic!("Attempt to skip beyond buffer length");
        }
        self.position = end;
    }

    pub fn read_u8(&mut self) -> u8 {
        let bytes = self.read_bytes(1);
        bytes[0]
    }

    pub fn read_u16(&mut self) -> u16 {
        let bytes = self.read_bytes(2);
        (&bytes[..]).read_u16::<BigEndian>().expect("Failed to read u16")
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes = self.read_bytes(4);
        (&bytes[..]).read_u32::<BigEndian>().expect("Failed to read u32")
    }

    pub fn read_i8(&mut self) -> i8 {
        let bytes = self.read_bytes(1);
        bytes[0] as i8
    }

    pub fn read_i16(&mut self) -> i16 {
        let bytes = self.read_bytes(2);
        (&bytes[..]).read_i16::<BigEndian>().expect("Failed to read i16")
    }

    pub fn read_i32(&mut self) -> i32 {
        let bytes = self.read_bytes(4);
        (&bytes[..]).read_i32::<BigEndian>().expect("Failed to read i32")
    }

    pub fn read_tag(&mut self) -> [u8; 4] {
        let bytes = self.read_bytes(4);
        [bytes[0], bytes[1], bytes[2], bytes[3]]
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    pub fn current_position(&self) -> usize {
        self.position
    }

    pub fn read_array<T: Readable>(&mut self, count: usize) -> Vec<T> {
        let mut array = Vec::with_capacity(count);
        for _ in 0..count {
            array.push(T::read_from_buffer(self));
        }
        array
    }
}

pub trait Readable: Sized {
    fn read_from_buffer(buffer: &mut ByteBuffer) -> Self;
}

impl Readable for u8 {
    fn read_from_buffer(buffer: &mut ByteBuffer) -> Self {
        buffer.read_u8()
    }
}

impl Readable for u16 {
    fn read_from_buffer(buffer: &mut ByteBuffer) -> Self {
        buffer.read_u16()
    }
}

impl Readable for u32 {
    fn read_from_buffer(buffer: &mut ByteBuffer) -> Self {
        buffer.read_u32()
    }
}

impl Readable for i8 {
    fn read_from_buffer(buffer: &mut ByteBuffer) -> Self {
        buffer.read_i8()
    }
}

impl Readable for i16 {
    fn read_from_buffer(buffer: &mut ByteBuffer) -> Self {
        buffer.read_i16()
    }
}

impl Readable for i32 {
    fn read_from_buffer(buffer: &mut ByteBuffer) -> Self {
        buffer.read_i32()
    }
}
