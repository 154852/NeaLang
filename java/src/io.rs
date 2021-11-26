use std::fs::OpenOptions;
use std::io;
use std::io::Read;

pub struct BinaryReader {
    raw: Vec<u8>,
    idx: usize
}

#[allow(dead_code)]
impl BinaryReader {
    pub fn open(path: &str) -> io::Result<BinaryReader> {
        let mut vec = Vec::new();

        OpenOptions::new()
            .read(true).write(false).create(false)
            .open(path)?.read_to_end(&mut vec)?;

        Ok(BinaryReader {
            raw: vec,
            idx: 0
        })
    }

    pub fn from(binary: Vec<u8>) -> BinaryReader {
        BinaryReader {
            raw: binary,
            idx: 0
        }
    }

    pub fn take(self) -> Vec<u8> {
        self.raw
    }

    pub fn next_u8(&mut self) -> Option<u8> {
        self.idx += 1;
        Some(*self.raw.get(self.idx - 1)?)
    }

    pub fn read_bytes(&mut self, data: &mut [u8]) -> bool {
        if self.idx + data.len() > self.raw.len() { return false; }
        data.copy_from_slice(&self.raw[self.idx..self.idx + data.len()]);
        self.idx += data.len();
        true
    }

    pub fn next_i8(&mut self) -> Option<i8> {
        Some(self.next_u8()? as i8)
    }

    pub fn next_u16(&mut self) -> Option<u16> {
        Some(((self.next_u8()? as u16) << 8) | self.next_u8()? as u16)
    }

    pub fn next_i16(&mut self) -> Option<i16> {
        Some(self.next_u16()? as i16)
    }

    pub fn next_u32(&mut self) -> Option<u32> {
        Some(((self.next_u16()? as u32) << 16) | self.next_u16()? as u32)
    }

    pub fn next_i32(&mut self) -> Option<i32> {
        Some(self.next_u32()? as i32)
    }

    pub fn next_u64(&mut self) -> Option<u64> {
        Some(((self.next_u32()? as u64) << 32) | self.next_u32()? as u64)
    }

    pub fn next_i64(&mut self) -> Option<i64> {
        Some(self.next_u64()? as i64)
    }

    pub fn next_f32(&mut self) -> Option<f32> {
        Some(self.next_u32()? as f32)
    }

    pub fn next_f64(&mut self) -> Option<f64> {
        Some(self.next_u64()? as f64)
    }

    pub fn tell(&self) -> usize {
        self.idx
    }

    pub fn finished(&self) -> bool {
        self.idx >= self.raw.len()
    }
}

pub struct BinaryWriter {
    raw: Vec<u8>,
}

#[allow(dead_code)]
impl BinaryWriter {
    pub fn new() -> BinaryWriter {
        BinaryWriter {
            raw: Vec::new(),
        }
    }

    pub fn take(self) -> Vec<u8> {
        self.raw
    }

    pub fn u8(&mut self, val: u8) { self.raw.push(val); }
    pub fn i8(&mut self, val: i8) { self.raw.push(val as u8); }
    pub fn u16(&mut self, val: u16) { self.raw.extend(val.to_be_bytes()); }
    pub fn i16(&mut self, val: i16) { self.raw.extend(val.to_be_bytes()); }
    pub fn u32(&mut self, val: u32) { self.raw.extend(val.to_be_bytes()); }
    pub fn i32(&mut self, val: i32) { self.raw.extend(val.to_be_bytes()); }
    pub fn u64(&mut self, val: u64) { self.raw.extend(val.to_be_bytes()); }
    pub fn i64(&mut self, val: i64) { self.raw.extend(val.to_be_bytes()); }

    pub fn f32(&mut self, val: f32) { self.raw.extend(val.to_be_bytes()); }
    pub fn f64(&mut self, val: f64) { self.raw.extend(val.to_be_bytes()); }

    pub fn bytes(&mut self, data: &[u8]) {
        self.raw.extend(data);
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }
}