use x86;

pub(crate) const SYS_V_ABI: &[x86::RegClass] = &[
    x86::RegClass::Edi,
    x86::RegClass::Esi,
    x86::RegClass::Edx,
    x86::RegClass::Ecx,
    x86::RegClass::R8,
    x86::RegClass::R9,

    // TODO: Handle items on the stack

    x86::RegClass::R10,
    x86::RegClass::R11,
    x86::RegClass::R12,
    x86::RegClass::R13,
    x86::RegClass::R14,
    x86::RegClass::R15,
    x86::RegClass::Ebx,
    x86::RegClass::Eax,
];

pub(crate) const SYS_V_ABI_RET: &[x86::RegClass] = &[
    x86::RegClass::Eax,
    x86::RegClass::Edx,
    x86::RegClass::Ebx,

    // Non standard:

    x86::RegClass::R10,
    x86::RegClass::R11,
    x86::RegClass::R12,
    x86::RegClass::R13,
    x86::RegClass::R14,
    x86::RegClass::R15,
];

pub(crate) const SYS_V_CALLEE_SAVED: &[x86::RegClass] = &[
    x86::RegClass::Ebx, 
    x86::RegClass::R12,
    x86::RegClass::R13,
    x86::RegClass::R14,
    x86::RegClass::R15
];

pub trait CalleeSavedIndex {
    fn callee_saved_index(&self) -> Option<u8>;
}

impl CalleeSavedIndex for x86::RegClass {
    fn callee_saved_index(&self) -> Option<u8> {
        match self {
            x86::RegClass::Ebx => Some(0), 
            x86::RegClass::R12 => Some(1),
            x86::RegClass::R13 => Some(2),
            x86::RegClass::R14 => Some(3),
            x86::RegClass::R15 => Some(4),
            _ => None
        }
    }
}

pub(crate) struct StackToReg {
    idx: usize,
    mode: x86::Mode,
    clobbered: u8 // bitmap, bits 0-5 are registers in SYS_V_CALLEE_SAVED
}

impl StackToReg {
    pub fn new(mode: x86::Mode) -> StackToReg {
        StackToReg {
            idx: 0,
            mode,
            clobbered: 0
        }
    }

    pub fn clobbered(&self) -> u8 {
        self.clobbered
    }

    fn does_use_reg(&mut self, reg: x86::RegClass) {
        if let Some(idx) = reg.callee_saved_index() {
            self.clobbered |= 1 << idx;
        }
    }

    pub fn push(&mut self) -> x86::RegClass {
        self.idx += 1;

        let reg = SYS_V_ABI_RET[self.idx - 1];
        self.does_use_reg(reg);
        
        reg
    }

    pub fn pop(&mut self) -> x86::RegClass {
        self.idx -= 1;
        SYS_V_ABI_RET[self.idx]
    }

    pub fn peek(&self) -> x86::RegClass {
        SYS_V_ABI_RET[self.idx - 1]
    }
    
    pub fn peek_at(&self, off: usize) -> x86::RegClass {
        SYS_V_ABI_RET[self.idx - 1 - off]
    }

    pub(crate) fn at(&self, idx: usize) -> x86::RegClass {
        SYS_V_ABI_RET[idx]
    }

    pub fn push_many(&mut self, count: usize) {
        // Can't just do self.idx += count; since we need to check for clobbered registers
        for _ in 0..count {
            self.push();
        }
    }

    pub fn pop_many(&mut self, count: usize) {
        self.idx -= count;
    }

    pub fn zero(&mut self) {
        self.idx = 0;
    }

    pub fn size(&self) -> usize {
        self.idx
    }

    pub fn uses(&self, reg: x86::RegClass) -> bool {
        for i in 0..self.idx {
            if SYS_V_ABI_RET[i] == reg {
                return true;
            }
        }

        false
    }

    pub fn pop_vt(&mut self, vt: &ir::ValueType) -> x86::Reg {
        crate::util::reg_for_value_type(vt, self.mode, self.pop())
    }

    pub fn push_vt(&mut self, vt: &ir::ValueType) -> x86::Reg {
        crate::util::reg_for_value_type(vt, self.mode, self.push())
    }

    pub fn push_ptr(&mut self) -> x86::Reg {
        self.push().uptr(&self.mode)
    }

    pub fn pop_ptr(&mut self) -> x86::Reg {
        self.pop().uptr(&self.mode)
    }

    pub fn peek_ptr(&self) -> x86::Reg {
        self.peek().uptr(&self.mode)
    }

    pub fn peek_vt(&self, vt: &ir::ValueType) -> x86::Reg {
        crate::util::reg_for_value_type(vt, self.mode, self.peek())
    }

    pub fn peek_at_vt(&self, off: usize, vt: &ir::ValueType) -> x86::Reg {
        crate::util::reg_for_value_type(vt, self.mode, self.peek_at(off))
    }

    pub fn at_vt(&self, off: usize, vt: &ir::ValueType) -> x86::Reg {
        crate::util::reg_for_value_type(vt, self.mode, self.at(off))
    }
}