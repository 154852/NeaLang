pub mod elf;
pub mod elfbuilder;
pub mod macho;

// Alignment is the actual number (e.g. 1 << 12) not the order (in this case 12)
pub fn align_up<T: std::ops::Rem<Output = T> + std::ops::Sub<Output = T> + std::ops::Add<Output = T> + Eq + Default + Copy>(value: T, alignment: T) -> T {
    let modulo = value % alignment;
    if modulo == Default::default() { return value; }
    return value + alignment - modulo;
}

pub fn align_up_vec<T: Default + Clone>(vec: &mut Vec<T>, alignment: usize) {
    let modulo = vec.len() % alignment;
    if modulo != 0 {
        vec.extend(vec![Default::default(); alignment - modulo]);
    }
}

pub fn align_up_vec_offset<T: Default + Clone>(vec: &mut Vec<T>, offset: usize, alignment: usize) {
    let modulo = (vec.len() + offset) % alignment;
    if modulo != 0 {
        vec.extend(vec![Default::default(); alignment - modulo]);
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Endian {
    Little,
    Big
}

pub trait EncodableInt {
    fn size() -> usize;
    fn endian() -> Endian;
    fn u32(value: u32) -> [u8; 4];
    fn u16(value: u16) -> [u8; 2];
    fn encode(value: u64, arr: &mut Vec<u8>);
}

pub struct LittleEndian32;
impl EncodableInt for LittleEndian32 {
    fn size() -> usize {
        4
    }

    fn endian() -> Endian {
        Endian::Little
    }

    fn u32(value: u32) -> [u8; 4] {
        value.to_le_bytes()
    }

    fn u16(value: u16) -> [u8; 2] {
        value.to_le_bytes()
    }

    fn encode(value: u64, arr: &mut Vec<u8>) {
        arr.extend(&(value as u32).to_le_bytes());
    }
}

pub struct BigEndian32;
impl EncodableInt for BigEndian32 {
    fn size() -> usize {
        4
    }

    fn endian() -> Endian {
        Endian::Big
    }

    fn u32(value: u32) -> [u8; 4] {
        value.to_be_bytes()
    }

    fn u16(value: u16) -> [u8; 2] {
        value.to_be_bytes()
    }

    fn encode(value: u64, arr: &mut Vec<u8>) {
        arr.extend(&(value as u32).to_be_bytes());
    }
}

pub struct LittleEndian64;
impl EncodableInt for LittleEndian64 {
    fn size() -> usize {
        8
    }

    fn endian() -> Endian {
        Endian::Little
    }

    fn u32(value: u32) -> [u8; 4] {
        value.to_le_bytes()
    }

    fn u16(value: u16) -> [u8; 2] {
        value.to_le_bytes()
    }

    fn encode(value: u64, arr: &mut Vec<u8>) {
        arr.extend(&value.to_le_bytes());
    }
}

pub struct BigEndian64;
impl EncodableInt for BigEndian64 {
    fn size() -> usize {
        8
    }

    fn endian() -> Endian {
        Endian::Big
    }

    fn u32(value: u32) -> [u8; 4] {
        value.to_be_bytes()
    }

    fn u16(value: u16) -> [u8; 2] {
        value.to_be_bytes()
    }

    fn encode(value: u64, arr: &mut Vec<u8>) {
        arr.extend(&value.to_be_bytes());
    }
}