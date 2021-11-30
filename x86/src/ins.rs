use std::collections::HashMap;
use crate::{Encoder, GlobalSymbolID, LocalSymbolID, Mem, Reg, RegClass, Relocation, Size};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Condition {
    Zero,
    NotZero,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual
}

impl Condition {
    fn base(&self) -> u8 {
        match self {
            Condition::Zero => 0x4,
            Condition::NotZero => 0x5,
            Condition::Less => 0xc,
            Condition::LessOrEqual => 0xe,
            Condition::Greater => 0xf,
            Condition::GreaterOrEqual => 0xd,
        }
    }
}

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

    /// A <- A & B
    AndRegReg(Reg, Reg),
    /// A <- A & B
    AndRegMem(Reg, Mem),
    /// A <- A & B
    AndMemReg(Mem, Reg),
    /// A <- A & B
    AndRegImm(Reg, u64),
    /// A <- A & B
    AndMemImm(Size, Mem, u64),

    // Call A
    CallGlobalSymbol(GlobalSymbolID),

    // cwd / cdq / cqo
    Cdq(Size),

    // If Condition Then A <- B
    CMovRegReg(Condition, Reg, Reg),
    // If Condition Then A <- B
    CMovRegMem(Condition, Reg, Mem),

    /// Cmp A, B
    CmpRegReg(Reg, Reg),
    /// Cmp A, B
    CmpRegImm(Reg, u64),
    /// Cmp A, B
    CmpRegMem(Reg, Mem),
    /// Cmp A, B
    CmpMemReg(Mem, Reg),

    /// If Condition Then A = 1
    ConditionalSet(Condition, RegClass),

    // eax <- eax / A
    IDivReg(Reg),
    // eax <- eax / A
    IDivMem(Size, Mem),

    // A <- A * B
    IMulRegReg(Reg, Reg),
    // A <- A * B
    IMulRegMem(Reg, Mem),

    /// Jump A
    JumpLocalSymbol(LocalSymbolID),
    /// If Condition Then Jump A
    JumpConditionalLocalSymbol(Condition, LocalSymbolID),

    /// A <- &B
    LeaRegMem(Reg, Mem),
    /// A <- &B
    LeaRegGlobalSymbol(Reg, GlobalSymbolID),

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

    // A <- Sign extended B
    MovsxRegReg(Reg, Reg),
    // A <- Sign extended B
    MovsxRegMem(Size, Reg, Mem),

    // A <- Zero extended B
    MovzxRegReg(Reg, Reg),
    // A <- Zero extended B
    MovzxRegMem(Size, Reg, Mem),

    /// A <- A | B
    OrRegReg(Reg, Reg),
    /// A <- A | B
    OrRegMem(Reg, Mem),
    /// A <- A | B
    OrMemReg(Mem, Reg),
    /// A <- A | B
    OrRegImm(Reg, u64),
    /// A <- A | B
    OrMemImm(Size, Mem, u64),

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

    /// Test A, B
    TestRegReg(Reg, Reg),
    /// Test A, B
    TestMemReg(Mem, Reg),
    /// Test A, B
    TestRegImm(Reg, u64),
}

impl Ins {
    pub fn encode(&self, data: &mut Vec<u8>, local_symbols: &mut HashMap<LocalSymbolID, usize>, unfilled_local_symbols: &mut Vec<Relocation>) {
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

            // https://www.felixcloutier.com/x86/and
            Ins::AndRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x20 } else { 0x21 }).rr(b, a).to(data),
            Ins::AndRegMem(r, ref m) => Encoder::new(if r.size() == Size::Byte { 0x22 } else { 0x23 }).rm(r, m).to(data),
            Ins::AndMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x20 } else { 0x21 }).rm(r, m).to(data),
            Ins::AndRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0x80 } else { 0x81 }).rn(r, 4).immn(i as u32, r.size()).to(data),
            Ins::AndMemImm(s, ref m, i) => Encoder::new(if s == Size::Byte { 0x80 } else { 0x81 }).mn(s, m, 4).immn(i as u32, s).to(data),

            // https://www.felixcloutier.com/x86/call
            Ins::CallGlobalSymbol(id) => {
                Encoder::new(0xe8).imm32(0).to(data);
                unfilled_local_symbols.push(Relocation::new_global_relative(id, data.len() - 4, -4));
            },

            // https://www.felixcloutier.com/x86/cwd:cdq:cqo
            Ins::Cdq(s) => match s {
                Size::Byte => panic!("Cannot CDQ byte"),
                Size::Word | Size::Double => Encoder::new(0x99).to(data),
                Size::Quad => Encoder::new(0x99).long().to(data),
            },

            // https://www.felixcloutier.com/x86/cmovcc
            Ins::CMovRegReg(c, a, b) => Encoder::new_long([0x0f, 0x40 + c.base()]).rr(a, b).to(data),
            Ins::CMovRegMem(c, r, ref m) => Encoder::new_long([0x0f, 0x40 + c.base()]).rm(r, m).to(data),

            // https://www.felixcloutier.com/x86/cmp
            Ins::CmpRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x3a } else { 0x3b }).rr(a, b).to(data),
            Ins::CmpRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0x80 } else { 0x81 }).rn(r, 7).immn(i as u32, r.size()).to(data),
            Ins::CmpRegMem(r, ref m) => Encoder::new(if r.size() == Size::Byte { 0x3a } else { 0x3b }).rm(r, m).to(data),
            Ins::CmpMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x38 } else { 0x39 }).rm(r, m).to(data),

            // https://www.felixcloutier.com/x86/setcc
            Ins::ConditionalSet(c, r) => Encoder::new_long([0x0f, 0x90 + c.base()]).rn(r.u8(), 0).to(data),

            // https://www.felixcloutier.com/x86/idiv
            Ins::IDivReg(a) => Encoder::new(if a.size() == Size::Byte { 0xf6 } else { 0xf7 }).rn(a, 7).to(data),
            Ins::IDivMem(s, ref m) => Encoder::new(if s == Size::Byte { 0xf6 } else { 0xf7 }).mn(s, m, 7).to(data),

            // https://www.felixcloutier.com/x86/imul
            Ins::IMulRegReg(a, b) => Encoder::new_long([0x0f, 0xaf]).rr(a, b).to(data),
            Ins::IMulRegMem(r, ref m) => Encoder::new_long([0x0f, 0xaf]).rm(r, m).to(data),

            // https://www.felixcloutier.com/x86/jmp
            Ins::JumpLocalSymbol(id) => {
                Encoder::new(0xe9).imm32(0).to(data);
                unfilled_local_symbols.push(Relocation::new_local_branch(id, data.len() - 4, -4));
            },

            // https://www.felixcloutier.com/x86/jcc
            Ins::JumpConditionalLocalSymbol(c, id) => {
                Encoder::new_long([0x0f, 0x80 + c.base()]).imm32(0).to(data);
                unfilled_local_symbols.push(Relocation::new_local_branch(id, data.len() - 4, -4));
            },

            // https://www.felixcloutier.com/x86/lea
            Ins::LeaRegMem(r, ref m) => Encoder::new(0x8d).rm(r, m).to(data),
            Ins::LeaRegGlobalSymbol(r, idx) => {
                Encoder::new(0x8d).rm(r, &Mem::new().base(RegClass::Eip).disp(0)).to(data);
                unfilled_local_symbols.push(Relocation::new_global_relative(idx, data.len() - 4, -4));
            },

            // https://www.felixcloutier.com/x86/mov
            Ins::MovRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x88 } else { 0x89 }).rr(b, a).to(data),
            Ins::MovRegMem(r, ref m) => Encoder::new(if r.size() == Size::Byte { 0x8a } else { 0x8b }).rm(r, m).to(data),
            Ins::MovMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x88 } else { 0x89 }).rm(r, m).to(data),
            Ins::MovRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0xb0 } else { 0xb8 }).offset(r).immnq(i, r.size()).to(data),
            Ins::MovMemImm(s, ref m, i) => Encoder::new(if s == Size::Byte { 0xc6 } else { 0xc7 }).mn(s, m, 0).immn(i as u32, s).to(data),

            // https://www.felixcloutier.com/x86/movsx:movsxd
            Ins::MovsxRegReg(a, b) => match b.size() {
                Size::Byte => Encoder::new_long([0x0f, 0xbe]).rr(a, b).to(data),
                Size::Word => Encoder::new_long([0x0f, 0xbf]).rr(a, b).to(data),
                Size::Double => Encoder::new(0x63).rr(a, b).to(data),
                _ => panic!("Cannot sign extend from 64 bits")
            },
            Ins::MovsxRegMem(s, r, ref m) => match s {
                Size::Byte => Encoder::new_long([0x0f, 0xbe]).rm(r, m).to(data),
                Size::Word => Encoder::new_long([0x0f, 0xbf]).rm(r, m).to(data),
                Size::Double => Encoder::new(0x63).rm(r, m).to(data),
                _ => panic!("Cannot sign extend from 64 bits")
            },

            // https://www.felixcloutier.com/x86/movzx
            Ins::MovzxRegReg(a, b) => match b.size() {
                Size::Byte => Encoder::new_long([0x0f, 0xb6]).rr(a, b).to(data),
                Size::Word => Encoder::new_long([0x0f, 0xb7]).rr(a, b).to(data),
                _ => panic!("Cannot zero extend from 8 or 16 bits")
            },
            Ins::MovzxRegMem(s, r, ref m) => match s {
                Size::Byte => Encoder::new_long([0x0f, 0xb6]).rm(r, m).to(data),
                Size::Word => Encoder::new_long([0x0f, 0xb7]).rm(r, m).to(data),
                _ => panic!("Cannot zero extend from 8 or 16 bits")
            },

            // https://www.felixcloutier.com/x86/or
            Ins::OrRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x08 } else { 0x09 }).rr(b, a).to(data),
            Ins::OrRegMem(r, ref m) => Encoder::new(if r.size() == Size::Byte { 0x0a } else { 0x0b }).rm(r, m).to(data),
            Ins::OrMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x08 } else { 0x09 }).rm(r, m).to(data),
            Ins::OrRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0x80 } else { 0x81 }).rn(r, 1).immn(i as u32, r.size()).to(data),
            Ins::OrMemImm(s, ref m, i) => Encoder::new(if s == Size::Byte { 0x80 } else { 0x81 }).mn(s, m, 1).immn(i as u32, s).to(data),

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

            // https://www.felixcloutier.com/x86/test
            Ins::TestRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x84 } else { 0x85 }).rr(a, b).to(data),
            Ins::TestMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x84 } else { 0x85 }).mr(m, r).to(data),
            Ins::TestRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0xf6 } else { 0xf7 }).rn(r, 0).immn(i as u32, r.size()).to(data),
        }
    }
}