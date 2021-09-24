use std::collections::HashMap;
use crate::{Encoder, LocalSymbolID, Mem, Reg, Size, UnfilledLocalSymbol};

pub enum Ins {
    LocalSymbol(LocalSymbolID),

    /// A <- A + B
    AddRegReg(Reg, Reg),
    /// A <- A + B
    AddRegMem(Reg, Mem),
    /// A <- A + B
    AddMemReg(Mem, Reg),
    /// A <- A + B
    AddRegImm(Reg, u64),
    /// A <- A + B
    AddMemImm(Size, Mem, u64),

    /// Jump A
    JumpLocalSymbol(LocalSymbolID),

    /// A <- B
    MovRegReg(Reg, Reg),
    /// A <- B
    MovRegMem(Reg, Mem),
    /// A <- B
    MovMemReg(Mem, Reg),
    /// A <- B
    MovRegImm(Reg, u64),
    /// A <- B
    MovMemImm(Size, Mem, u64),

    /// Pop A
    PopReg(Reg),
    /// Pop A
    PopMem(Mem),

    /// Push A
    PushReg(Reg),
    /// Push A
    PushMem(Mem),
    /// Push A
    PushImm(u64),

    /// Return
    Ret,

    /// A <- A - B
    SubRegReg(Reg, Reg),
    /// A <- A - B
    SubRegMem(Reg, Mem),
    /// A <- A - B
    SubMemReg(Mem, Reg),
    /// A <- A - B
    SubRegImm(Reg, u64),
    /// A <- A - B
    SubMemImm(Size, Mem, u64),
}

impl Ins {
    pub fn encode(&self, data: &mut Vec<u8>, local_symbols: &mut HashMap<LocalSymbolID, usize>, unfilled_local_symbols: &mut Vec<UnfilledLocalSymbol>) {
        match *self {
            Ins::LocalSymbol(id) => {
                local_symbols.insert(id, data.len());
            },

            // https://www.felixcloutier.com/x86/add
            Ins::AddRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x00 } else { 0x01 }).rr(b, a).to(data),
            Ins::AddRegMem(r, ref m) => Encoder::new(if r.size() == Size::Byte { 0x02 } else { 0x03 }).rm(r, m).to(data),
            Ins::AddMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x00 } else { 0x01 }).rm(r, m).to(data),
            Ins::AddRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0x80 } else { 0x81 }).rn(r, 0).immn(i as u32, r.size()).to(data),
            Ins::AddMemImm(s, ref m, i) => Encoder::new(if s == Size::Byte { 0x80 } else { 0x81 }).mn(s, m, 0).immn(i as u32, s).to(data),

            // https://www.felixcloutier.com/x86/jmp
            Ins::JumpLocalSymbol(id) => {
                Encoder::new(0xe9).imm32(0).to(data);
                unfilled_local_symbols.push(UnfilledLocalSymbol::new(id, data.len() - 4, -4));
            },

            // https://www.felixcloutier.com/x86/mov
            Ins::MovRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x88 } else { 0x89 }).rr(b, a).to(data),
            Ins::MovRegMem(r, ref m) => Encoder::new(if r.size() == Size::Byte { 0x8a } else { 0x8b }).rm(r, m).to(data),
            Ins::MovMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x88 } else { 0x89 }).rm(r, m).to(data),
            Ins::MovRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0xb0 } else { 0xb8 }).offset(r).immnq(i, r.size()).to(data),
            Ins::MovMemImm(s, ref m, i) => Encoder::new(if s == Size::Byte { 0xc6 } else { 0xc7 }).mn(s, m, 0).immn(i as u32, s).to(data),

            // https://www.felixcloutier.com/x86/pop
            Ins::PopReg(r) => Encoder::new(0x58).offset(r.class().u32()).to(data),
            Ins::PopMem(ref m) => Encoder::new(0x8f).mn(Size::Double, m, 0).to(data),

            // https://www.felixcloutier.com/x86/push
            Ins::PushReg(r) => Encoder::new(0x50).offset(r.class().u32()).to(data),
            Ins::PushMem(ref m) => Encoder::new(0xff).mn(Size::Double, m, 6).to(data),
            Ins::PushImm(i) => Encoder::new(0x68).imm32(i as u32).to(data), // TODO: This will change between x86/x86-64

            // https://www.felixcloutier.com/x86/ret
            Ins::Ret => Encoder::new(0xc3).to(data),

            // https://www.felixcloutier.com/x86/sub
            Ins::SubRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x28 } else { 0x29 }).rr(b, a).to(data),
            Ins::SubRegMem(r, ref m) => Encoder::new(if r.size() == Size::Byte { 0x2a } else { 0x2b }).rm(r, m).to(data),
            Ins::SubMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x28 } else { 0x29 }).rm(r, m).to(data),
            Ins::SubRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0x80 } else { 0x81 }).rn(r, 5).immn(i as u32, r.size()).to(data),
            Ins::SubMemImm(s, ref m, i) => Encoder::new(if s == Size::Byte { 0x80 } else { 0x81 }).mn(s, m, 5).immn(i as u32, s).to(data),
        }
    }
}