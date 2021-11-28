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