use crate::{Reg, RegClass, Size};

#[derive(Debug, Clone)]
pub struct Mem {
    base: Option<RegClass>,
    index: Option<RegClass>,
    scale: u8,
    disp: i64
}

impl Mem {
    pub fn new() -> Mem {
        Mem {
            base: None,
            index: None,
            scale: 1,
            disp: 0
        }
    }

    pub fn base(mut self, base: RegClass) -> Self {
        self.base = Some(base);
        self
    }

    pub fn disp(mut self, disp: i64) -> Self {
        self.disp = disp;
        self
    }

    pub fn modrm_to_reg(&self, reg: Reg, data: &mut Vec<u8>) {
        self.modrm_with(reg.class().id(), data)
    }

    // http://www.cs.loyola.edu/~binkley/371/Encoding_Real_x86_Instructions.html
    pub fn modrm_with(&self, r: u8, data: &mut Vec<u8>) {
        if self.scale == 1 && self.index.is_none() {
            if let Some(base) = self.base {
                if self.disp == 0 {
                    if base == RegClass::Ebp {
                        data.push((0b01 << 6) | (r << 3) | base.id());
                        data.push(0);
                    } else if base == RegClass::Esp {
                        data.push((0b00 << 6) | (r << 3) | 0b100);
                        data.push((0b00 << 6) | (0b100 << 3) | 0b100);
                    } else {
                        data.push((0b00 << 6) | (r << 3) | base.id());
                    }
                } else {
                    let disp = self.disp as u64;
                    if self.disp >= -128 && self.disp <= 127 {
                        if base == RegClass::Esp {
                            data.push((0b01 << 6) | (r << 3) | 0b100);
                            data.push((0b00 << 6) | (0b100 << 3) | 0b100);
                        } else {
                            data.push((0b01 << 6) | (r << 3) | base.id());
                        }

                        data.push(disp as u8);
                    } else {
                        if base == RegClass::Esp {
                            data.push((0b10 << 6) | (r << 3) | 0b100);
                            data.push((0b00 << 6) | (0b100 << 3) | 0b100);
                        } else {
                            data.push((0b10 << 6) | (r << 3) | base.id());
                        }

                        data.extend(&(disp as u32).to_le_bytes());
                    }
                }
            } else {
                data.push((0b00 << 6) | (r << 3) | 0b101);
                data.extend(&(self.disp as u32).to_le_bytes());
            }
        } else {
            todo!();
        }
    }

    pub fn explicit_read_regs(&self) -> Vec<RegClass> {
        let mut regs = Vec::new();
        
        if let Some(base) = self.base {
            regs.push(base);
        }

        if let Some(index) = self.index {
            regs.push(index);
        }

        regs
    }
}

type Prefix = u8;

trait PrefixImpl {
    fn new() -> Self;
    
    fn operand_size_override() -> Self;

    fn w(self) -> Self;
    fn w_if(self, c: bool) -> Self;
    fn r(self) -> Self;
    fn r_if(self, c: bool) -> Self;
    fn x(self) -> Self;
    fn x_if(self, c: bool) -> Self;
    fn b(self) -> Self;
    fn b_if(self, c: bool) -> Self;
}

impl PrefixImpl for Prefix {
    fn new() -> Prefix { 0b01000000 }

    fn operand_size_override() -> Prefix { 0x66 }

    fn w(self) -> Prefix { self | 0b1000 }
    fn w_if(self, c: bool) -> Prefix { if c { self | 0b1000 } else { self } }
    fn r(self) -> Prefix { self | 0b0100 }
    fn r_if(self, c: bool) -> Prefix { if c { self | 0b0100 } else { self } }
    fn x(self) -> Prefix { self | 0b0010 }
    fn x_if(self, c: bool) -> Prefix { if c { self | 0b0010 } else { self } }
    fn b(self) -> Prefix { self | 0b0001 }
    fn b_if(self, c: bool) -> Prefix { if c { self | 0b0001 } else { self } }
}

pub(crate) struct Encoder {
    operand_size_override: bool,
    prefix: Option<Prefix>,
    opcode: Vec<u8>,
    operands: Vec<u8>,
    imm: Vec<u8>
}

impl Encoder {
    pub fn new(opcode: u8) -> Encoder {
        Encoder {
            opcode: vec![opcode],
            operand_size_override: false,
            prefix: None,
            operands: Vec::new(),
            imm: Vec::new()
        }
    }

    pub fn new_long<T: Into<Vec<u8>>>(opcode: T) -> Encoder {
        Encoder {
            opcode: opcode.into(),
            operand_size_override: false,
            prefix: None,
            operands: Vec::new(),
            imm: Vec::new()
        }
    }

    /// Forces the presence of an empty REX prefix
    pub fn rex(mut self) -> Self {
        self.prefix = Some(Prefix::new());
        self
    }

    /// Sets the 0x66 prefix, meaning that an otherwise 32 bit instruction operates on 16 bits instead
    pub fn opsize_override(mut self) -> Self {
        self.operand_size_override = true;
        self
    }

    /// Sets W prefix for 64 bit instructions
    pub fn long(mut self) -> Self {
        self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).w());
        self
    }

    /// Single byte immediate
    pub fn imm8(mut self, value: u8) -> Self {
        self.imm.push(value);
        self
    }

    /// 16 bit immediate
    pub fn imm16(mut self, value: u16) -> Self {
        self.imm.extend(&value.to_le_bytes());
        self
    }

    /// 32 bit immediate
    pub fn imm32(mut self, value: u32) -> Self {
        self.imm.extend(&value.to_le_bytes());
        self
    }

    /// 64 bit immediate
    pub fn imm64(mut self, value: u64) -> Self {
        self.imm.extend(&value.to_le_bytes());
        self
    }

    /// Immediate with given size, but cap at 32 bits
    pub fn immn(self, value: u32, size: Size) -> Self {
        match size {
            Size::Byte => self.imm8(value as u8),
            Size::Word => self.imm16(value as u16),
            Size::Double | Size::Quad => self.imm32(value),
        }
    }

    /// Immediate with given size
    pub fn immnq(self, value: u64, size: Size) -> Self {
        match size {
            Size::Byte => self.imm8(value as u8),
            Size::Word => self.imm16(value as u16),
            Size::Double => self.imm32(value as u32),
            Size::Quad => self.imm64(value)
        }
    }

    /// For instruction such as push, where the target register is added to the opcode
    pub fn offset(mut self, reg: Reg) -> Self {
        assert!(self.opcode.len() == 1);

        self.opcode[0] += reg.class().id();
        
        if reg.class().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).b()) }
        if matches!(reg.size(), Size::Quad) { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).w()) }
        if matches!(reg.size(), Size::Word) { self.operand_size_override = true; }

        self
    }

    /// Two registers in a single mod rm byte
    pub fn rr(mut self, a: Reg, b: Reg) -> Self {
        self.operands.push(Reg::modrm_reg_addressing(a, b));
        
        if a.class().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).r()) }
        if b.class().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).b()) }
        
        if matches!(a.size(), Size::Byte) && a.class().byte_forces_rex() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new())); }
        if matches!(a.size(), Size::Quad) { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).w()) }
        if matches!(a.size(), Size::Word) { self.operand_size_override = true; }

        self
    }

    /// For /x, where a single register and some disambiguating bits are used
    pub fn rn(mut self, a: Reg, b: u8) -> Self {
        self.operands.push(a.modrm_reg_addressing_single(b));
        
        if a.class().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).b()) }
        
        if matches!(a.size(), Size::Byte) && a.class().byte_forces_rex() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new())); }
        if matches!(a.size(), Size::Quad) { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).w()) }
        if matches!(a.size(), Size::Word) { self.operand_size_override = true; }

        self
    }

    /// A register and memory
    pub fn rm(mut self, a: Reg, b: &Mem) -> Self {
        b.modrm_to_reg(a, &mut self.operands);
        
        if a.class().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).r()) }

        if b.base.is_some() && b.base.unwrap().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).b()) }
        if b.index.is_some() && b.index.unwrap().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).x()) }
        
        if matches!(a.size(), Size::Byte) && a.class().byte_forces_rex() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new())); }
        if matches!(a.size(), Size::Quad) { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).w()) }
        if matches!(a.size(), Size::Word) { self.operand_size_override = true; }

        self
    }

    /// Memory and a register, encoded differently only in the opcode, so just calls rm
    pub fn mr(self, a: &Mem, b: Reg) -> Self {
        self.rm(b, a)
    }

    /// Like rn, but with memory
    pub fn mn(mut self, size: Size, a: &Mem, x: u8) -> Self {
        a.modrm_with(x, &mut self.operands);

        if a.base.is_some() && a.base.unwrap().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).b()) }
        if a.index.is_some() && a.index.unwrap().is_rn() { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).x()) }
        
        if matches!(size, Size::Quad) { self.prefix = Some(self.prefix.unwrap_or_else(|| Prefix::new()).w()) }
        if matches!(size, Size::Word) { self.operand_size_override = true; }

        self
    }

    /// Encode and write to data
    pub fn to(&self, data: &mut Vec<u8>) {
        if self.operand_size_override { data.push(Prefix::operand_size_override()); }
        if let Some(prefix) = self.prefix { data.push(prefix); }
        data.extend(&self.opcode);
        data.extend(&self.operands);
        data.extend(&self.imm);
    }
}