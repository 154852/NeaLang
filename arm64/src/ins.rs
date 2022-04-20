use crate::reloc::{LocalSymbolID, RelocationType, GlobalSymbolID};

#[derive(Clone, Copy)]
pub struct Reg(pub u32);

impl Reg {
    pub fn u32(&self) -> u32 {
        self.0
    }

    pub fn sp() -> Reg {
        Reg(31)
    }

    pub fn zero() -> Reg {
        Reg(31)
    }

    pub fn lr() -> Reg {
        Reg(30)
    }

    pub fn fp() -> Reg {
        Reg(29)
    }
}

#[derive(Clone, Copy)]
pub enum SizeFlag {
    Size32 = 0,
    Size64 = 1
}

impl SizeFlag {
    pub fn is_32(&self) -> bool {
        return matches!(self, SizeFlag::Size32);
    }

    pub fn is_64(&self) -> bool {
        return matches!(self, SizeFlag::Size64);
    }

    pub fn divisor(&self) -> i32 {
        4 << *self as u32
    }
}

#[derive(Clone, Copy)]
pub enum ShiftMode {
    LogicalLeft = 0,
    LogicalRight = 1,
    ArithmeticRight = 2,
    RotateRight = 3
}

pub enum InsRelocMode {
    Branch26,
    Branch19Shift5,
    Page21,
    PageOff12
}

pub struct InsReloc {
    mode: InsRelocMode,
    symbol: RelocationType
}

impl InsReloc {
    pub fn symbol(&self) -> &RelocationType {
        &self.symbol
    }

    pub fn mode(&self) -> &InsRelocMode {
        &self.mode
    }
}

pub struct InsEncodeResult {
    val: u32,
    reloc: Option<InsReloc>
}

type Res = InsEncodeResult;

impl InsEncodeResult {
    fn val(val: u32) -> InsEncodeResult {
        InsEncodeResult {
            val,
            reloc: None
        }
    }

    fn reloc(mut self, type_: InsRelocMode, symbol: RelocationType) -> InsEncodeResult {
        self.reloc = Some(InsReloc { mode: type_, symbol });
        self
    }

    pub fn get(&self) -> u32 {
        self.val
    }

    pub fn get_reloc(self) -> Option<InsReloc> {
        self.reloc
    }
}

#[derive(Clone, Copy)]
pub enum Condition {
    Eq = 0,
    Ne,
    Cs,
    Cc,
    Mi,
    Pl,
    Vs,
    Vc,
    Hi,
    Ls,
    Ge,
    Lt,
    Gt,
    Le,
    Al
}

impl Condition {
    pub fn inv(&self) -> Condition {
        match self {
            Condition::Eq => Condition::Ne,
            Condition::Ne => Condition::Eq,
            Condition::Cs => Condition::Cc,
            Condition::Cc => Condition::Cs,
            Condition::Mi => Condition::Pl,
            Condition::Pl => Condition::Mi,
            Condition::Vs => Condition::Vc,
            Condition::Vc => Condition::Vs,
            Condition::Hi => Condition::Ls,
            Condition::Ls => Condition::Hi,
            Condition::Ge => Condition::Lt,
            Condition::Lt => Condition::Ge,
            Condition::Gt => Condition::Le,
            Condition::Le => Condition::Gt,
            Condition::Al => panic!(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum IndexMode {
    PostIndex = 1,
    PreIndex = 3,
    SignedOffset = 2
}

#[derive(Clone, Copy)]
pub enum ImmShift {
    Shift0 = 0,
    Shift12 = 1
}
pub enum Ins {
    LocalSymbol(LocalSymbolID),
    OrrImm {
        size: SizeFlag,
        shift_mode: ShiftMode,
        src1: Reg, src2: Reg,
        dest: Reg, shift: u32
    },
    Mov {
        size: SizeFlag,
        src: Reg, dest: Reg
    },
    BranchLocalSymbol(LocalSymbolID),
    ConditionalBranchLocalSymbol(LocalSymbolID, Condition),
    BranchLinkGlobalSymbol(GlobalSymbolID),
    Ret(Reg),
    Stp {
        size: SizeFlag,
        mode: IndexMode,
        src1: Reg, src2: Reg,
        base: Reg, offset: i32
    },
    Ldp {
        size: SizeFlag,
        mode: IndexMode,
        dest1: Reg, dest2: Reg,
        base: Reg, offset: i32
    },
    SubImm {
        size: SizeFlag,
        src: Reg, dest: Reg,
        val: u32, shift: ImmShift
    },
    AddImm {
        size: SizeFlag,
        src: Reg, dest: Reg,
        val: u32, shift: ImmShift
    },
    AddPageOffGlobalSymbol {
        src: Reg, dest: Reg,
        symbol: GlobalSymbolID
    },
    MovZ {
        size: SizeFlag,
        dest: Reg,
        val: u32, shift: u32
    },
    Stur {
        size: SizeFlag,
        src: Reg,
        base: Reg, offset: i32
    },
    Sturb {
        src: Reg,
        base: Reg, offset: i32
    },
    Sturh {
        src: Reg,
        base: Reg, offset: i32
    },
    Ldur {
        size: SizeFlag,
        dest: Reg,
        base: Reg, offset: i32
    },
    Ldurb {
        dest: Reg,
        base: Reg, offset: i32
    },
    Ldurh {
        dest: Reg,
        base: Reg, offset: i32
    },
    MAdd {
        size: SizeFlag,
        dest: Reg,
        mul1: Reg, mul2: Reg, addend: Reg,
    },
    AdrpGlobalSymbol(GlobalSymbolID, Reg),
    AddShifted {
        size: SizeFlag,
        shift_mode: ShiftMode,
        dest: Reg,
        src: Reg, shifted_src: Reg,
        shift: u32
    },
    AndShifted {
        size: SizeFlag,
        shift_mode: ShiftMode,
        dest: Reg,
        src: Reg, shifted_src: Reg,
        shift: u32
    },
    OrrShifted {
        size: SizeFlag,
        shift_mode: ShiftMode,
        dest: Reg,
        src: Reg, shifted_src: Reg,
        shift: u32
    },
    SubShifted {
        size: SizeFlag,
        shift_mode: ShiftMode,
        dest: Reg,
        src: Reg, shifted_src: Reg,
        shift: u32
    },
    SubsShifted {
        size: SizeFlag,
        shift_mode: ShiftMode,
        dest: Reg,
        src: Reg, shifted_src: Reg,
        shift: u32
    },
    SDiv {
        size: SizeFlag,
        divisor: Reg,
        divided: Reg,
        dest: Reg
    },
    UDiv {
        size: SizeFlag,
        divisor: Reg,
        divided: Reg,
        dest: Reg
    },
    CSInc {
        size: SizeFlag,
        dest: Reg,
        cond: Condition,
        true_reg: Reg,
        inc_reg: Reg
    }
}

fn orr(size: SizeFlag, shift_mode: ShiftMode, src1: Reg, src2: Reg, shift: u32, dest: Reg) -> Res {
    Res::val(((size as u32) << 31) | (0b0101010 << 24) | ((shift_mode as u32) << 22) | (0 << 21) | (src2.u32() << 16) | (shift << 10) | (src1.u32() << 5) | (dest.u32() << 0))
}

fn dp(size: SizeFlag, mode: IndexMode, src1: Reg, src2: Reg, base: Reg, offset: i32, load: u32) -> Res {
    Res::val(((size as u32) << 31) | (0b010100 << 25) | ((mode as u32) << 23) | (load << 22) | (((offset / size.divisor()) & 1023) << 15) as u32 | (src2.u32() << 10) | (base.u32() << 5) | (src1.u32() << 0))
}

fn arith_imm(opcode: u32, size: SizeFlag, src: Reg, dest: Reg, val: u32, shift: ImmShift) -> Res {
    Res::val(((size as u32) << 31) | (opcode << 23) | ((shift as u32) << 22) | (val << 10) | (src.u32() << 5) | (dest.u32() << 0))
}

fn ldstr_ur(opcode: u32, size: SizeFlag, src: Reg, base: Reg, offset: i32) -> Res {
    Res::val((opcode << 21) | ((size as u32) << 30) | (((offset & 511) as u32) << 12) | (base.u32() << 5) | src.u32())
}

impl Ins {
    pub fn encode(&self) -> InsEncodeResult {
        match *self {
            Ins::LocalSymbol(_) => unreachable!(),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/ORR--immediate---Bitwise-OR--immediate--?lang=en
            Ins::OrrImm { size, shift_mode, src1, src2, dest, shift } => orr(size, shift_mode, src1, src2, shift, dest),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/MOV--bitmask-immediate---Move--bitmask-immediate---an-alias-of-ORR--immediate--?lang=en
            Ins::Mov { size, src, dest } => orr(size, ShiftMode::LogicalLeft, Reg::zero(), src, 0, dest),
            
            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/B--Branch-?lang=en
            Ins::BranchLocalSymbol(symbol) => Res::val(0b000101 << 26).reloc(InsRelocMode::Branch26, RelocationType::LocalFunctionSymbol(symbol)),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/B-cond--Branch-conditionally-?lang=en
            Ins::ConditionalBranchLocalSymbol(symbol, condition) => Res::val((0b01010100 << 24) | (0 << 4) | ((condition as u32) << 0)).reloc(InsRelocMode::Branch19Shift5, RelocationType::LocalFunctionSymbol(symbol)),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/BL--Branch-with-Link-?lang=en
            Ins::BranchLinkGlobalSymbol(symbol) => Res::val(0b100101 << 26).reloc(InsRelocMode::Branch26, RelocationType::RelativeGlobalSymbol(symbol)),
            
            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/RET--Return-from-subroutine-?lang=en
            Ins::Ret(reg) => Res::val((0b1101011001011111000000 << 10) | (reg.u32() << 5) | (0b0000 << 0)),
            
            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/STP--Store-Pair-of-Registers-?lang=en
            Ins::Stp { size, mode, src1, src2, base, offset } => dp(size, mode, src1, src2, base, offset, 0),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/LDP--Load-Pair-of-Registers-?lang=en
            Ins::Ldp { size, mode, dest1: src1, dest2: src2, base, offset } => dp(size, mode, src1, src2, base, offset, 1),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/SUB--immediate---Subtract--immediate--?lang=en
            Ins::SubImm { size, src, dest, val, shift } => arith_imm(0b10100010, size, src, dest, val, shift),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/ADD--immediate---Add--immediate--?lang=en
            Ins::AddImm { size, src, dest, val, shift } => arith_imm(0b00100010, size, src, dest, val, shift),

            Ins::AddPageOffGlobalSymbol { src, dest, symbol } => arith_imm(0b00100010, SizeFlag::Size64, src, dest, 0, ImmShift::Shift0).reloc(InsRelocMode::PageOff12, RelocationType::RelativeGlobalSymbol(symbol)),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/MOVZ--Move-wide-with-zero-?lang=en
            Ins::MovZ { size, dest, val, shift } => Res::val(((size as u32) << 31) | (0b10100101 << 23) | ((shift / 16) << 21) | (val << 5) | (dest.u32() << 0)),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/STUR--Store-Register--unscaled--?lang=en
            Ins::Stur { size, src, base, offset } => ldstr_ur(0b10111000000, size, src, base, offset),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/STURB--Store-Register-Byte--unscaled--?lang=en
            Ins::Sturb { src, base, offset } => ldstr_ur(0b00111000000, SizeFlag::Size32, src, base, offset),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/STURH--Store-Register-Halfword--unscaled--?lang=en
            Ins::Sturh { src, base, offset } => ldstr_ur(0b01111000000, SizeFlag::Size32, src, base, offset),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/LDUR--Load-Register--unscaled--?lang=en
            Ins::Ldur { size, dest, base, offset } => ldstr_ur(0b10111000010, size, dest, base, offset),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/LDURB--Load-Register-Byte--unscaled--?lang=en
            Ins::Ldurb { dest, base, offset } => ldstr_ur(0b00111000010, SizeFlag::Size32, dest, base, offset),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/LDURH--Load-Register-Halfword--unscaled--?lang=en
            Ins::Ldurh { dest, base, offset } => ldstr_ur(0b01111000010, SizeFlag::Size32, dest, base, offset),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/MADD--Multiply-Add-?lang=en
            Ins::MAdd { size, dest, mul1, mul2, addend } => Res::val(((size as u32) << 31) | (0b0011011000 << 21) | (mul2.u32() << 16) | (addend.u32() << 10) | (mul1.u32() << 5) | dest.u32()),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/ADRP--Form-PC-relative-address-to-4KB-page-?lang=en
            Ins::AdrpGlobalSymbol(sym, dest) => Res::val((0b10010000 << 24) | dest.u32()).reloc(InsRelocMode::Page21, RelocationType::RelativeGlobalSymbol(sym)),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/ADD--shifted-register---Add--shifted-register--?lang=en
            Ins::AddShifted { size, shift_mode, src, shifted_src, dest, shift } => Res::val(((size as u32) << 31) | (0b0001011 << 24) | ((shift_mode as u32) << 22) | (src.u32() << 16) | (shift << 10) | (shifted_src.u32() << 5) | dest.u32()),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/AND--shifted-register---Bitwise-AND--shifted-register--?lang=en
            Ins::AndShifted { size, shift_mode, src, shifted_src, dest, shift } => Res::val(((size as u32) << 31) | (0b0001010 << 24) | ((shift_mode as u32) << 22) | (src.u32() << 16) | (shift << 10) | (shifted_src.u32() << 5) | dest.u32()),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/ORR--shifted-register---Bitwise-OR--shifted-register--?lang=en
            Ins::OrrShifted { size, shift_mode, src, shifted_src, dest, shift } => Res::val(((size as u32) << 31) | (0b0101010 << 24) | ((shift_mode as u32) << 22) | (src.u32() << 16) | (shift << 10) | (shifted_src.u32() << 5) | dest.u32()),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/SUB--shifted-register---Subtract--shifted-register--?lang=en
            Ins::SubShifted { size, shift_mode, src, shifted_src, dest, shift } => Res::val(((size as u32) << 31) | (0b1001011 << 24) | ((shift_mode as u32) << 22) | (src.u32() << 16) | (shift << 10) | (shifted_src.u32() << 5) | dest.u32()),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/SUBS--shifted-register---Subtract--shifted-register---setting-flags-?lang=en
            Ins::SubsShifted { size, shift_mode, src, shifted_src, dest, shift } => Res::val(((size as u32) << 31) | (0b1101011 << 24) | ((shift_mode as u32) << 22) | (src.u32() << 16) | (shift << 10) | (shifted_src.u32() << 5) | dest.u32()),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/SDIV--Signed-Divide-?lang=en
            Ins::SDiv { size, divisor, divided, dest } => Res::val(((size as u32) << 31) | (0b0011010110 << 21) | (divisor.u32() << 16) | (0b000011 << 10) | (divided.u32() << 5) | dest.u32()),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/UDIV--Unsigned-Divide-?lang=en
            Ins::UDiv { size, divisor, divided, dest } => Res::val(((size as u32) << 31) | (0b0011010110 << 21) | (divisor.u32() << 16) | (0b000010 << 10) | (divided.u32() << 5) | dest.u32()),

            // https://developer.arm.com/documentation/ddi0596/2021-12/Base-Instructions/CSINC--Conditional-Select-Increment-?lang=en
            Ins::CSInc { size, dest, cond, true_reg, inc_reg } => Res::val(((size as u32) << 31) | (0b0011010100 << 21) | (inc_reg.u32() << 16) | ((cond as u32) << 12) | (0b01 << 10) | ((true_reg.u32()) << 5) | dest.u32()),
        }
    }
}