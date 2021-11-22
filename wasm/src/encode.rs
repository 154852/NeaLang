pub trait WasmEncodable {
    fn wasm_encode(&self, data: &mut Vec<u8>);
}

impl<T: WasmEncodable> WasmEncodable for Vec<T> {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        self.len().wasm_encode(data);
        for x in self.iter() {
            x.wasm_encode(data);
        }
    }
}

impl WasmEncodable for u64 {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        let mut value = *self;
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;

            if value == 0 {
                data.push(byte);
                break;
            }

            data.push(byte | 0x80);
        }
    }
}

impl WasmEncodable for u32 {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        (*self as u64).wasm_encode(data)
    }
}

impl WasmEncodable for i64 {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        let mut value = *self as i32;
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;

            if (value == 0 && (byte & 0x40 == 0)) || (value == -1 && (byte & 0x40 != 0)) {
                data.push(byte);
                break;
            }
            
            data.push(byte | 0x80);
        }
    }
}

impl WasmEncodable for i32 {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        (*self as i64).wasm_encode(data)
    }
}

impl WasmEncodable for usize {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        (*self as u32).wasm_encode(data)
    }
}

impl WasmEncodable for isize {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        (*self as u32).wasm_encode(data)
    }
}

impl WasmEncodable for String {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        self.len().wasm_encode(data);
        data.extend(self.as_bytes());
    }
}

impl WasmEncodable for f32 {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        data.extend(self.to_le_bytes());
    }
}

impl WasmEncodable for f64 {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        data.extend(self.to_le_bytes());
    }
}

impl WasmEncodable for u8 {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        data.push(*self);
    }
}