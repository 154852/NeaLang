use crate::{LocalSymbolStack, ins::X86ForIRIns, registerify::StackToReg};

pub trait X86ForIRFunction {
    fn build_x86(&self, mode: x86::Mode, unit: &ir::TranslationUnit) -> Vec<x86::Ins>;
}

impl X86ForIRFunction for ir::Function {
    fn build_x86(&self, mode: x86::Mode, unit: &ir::TranslationUnit) -> Vec<x86::Ins> {
        let mut x86_ins = Vec::new();

        let mut stack = StackToReg::new();
        let mut local_symbol_stack = LocalSymbolStack::new();

        if self.signature().params().len() == 0 {
            stack.set_no_params();
        } else {
            stack.push_many(self.signature().params().len());
        }

        if self.locals().len() > 0 {
            x86_ins.push(x86::Ins::PushReg(mode.base_ptr()));
            x86_ins.push(x86::Ins::MovRegReg(mode.base_ptr(), mode.stack_ptr()));
            x86_ins.push(x86::Ins::SubRegImm(mode.stack_ptr(), self.local_addr(mode, self.locals().len() - 1)));
        }

        for ins in self.code() {
            ins.build_x86(mode, &mut stack, &mut local_symbol_stack, unit, self, &mut x86_ins);
        }

        x86_ins.push(x86::Ins::LocalSymbol(local_symbol_stack.root()));
        if self.locals().len() > 0 {
            x86_ins.push(x86::Ins::MovRegReg(mode.stack_ptr(), mode.base_ptr()));
            x86_ins.push(x86::Ins::PopReg(mode.base_ptr()));
        }
        x86_ins.push(x86::Ins::Ret);

        x86_ins
    }
}

pub(crate) trait X86ForIRFunctionInternal {
    fn local_addr(&self, mode: x86::Mode, idx: ir::LocalIndex) -> u64;
    fn local_mem(&self, mode: x86::Mode, idx: ir::LocalIndex) -> x86::Mem;
}

impl X86ForIRFunctionInternal for ir::Function {
    fn local_addr(&self, mode: x86::Mode, idx: ir::LocalIndex) -> u64 {
        let mut addr = 0;

        assert!(self.locals().len() > idx);

        for i in self.locals().iter().take(idx + 1) {
            addr += i.value_type().bytes_size(mode.ptr_size() as u64);
        }

        addr
    }

    fn local_mem(&self, mode: x86::Mode, idx: ir::LocalIndex) -> x86::Mem {
        x86::Mem::new().base(x86::RegClass::Ebp).disp(-(self.local_addr(mode, idx) as i64))
    }
}

pub(crate) trait X86RegForValueType {
    fn x86_reg(&self, mode: x86::Mode, class: x86::RegClass) -> x86::Reg;
}

impl X86RegForValueType for ir::ValueType {
    fn x86_reg(&self, mode: x86::Mode, class: x86::RegClass) -> x86::Reg {
        match self {
            ir::ValueType::U8 | ir::ValueType::I8 => class.u8(),
            ir::ValueType::U16 | ir::ValueType::I16 => class.u16(),
            ir::ValueType::U32 | ir::ValueType::I32 => class.u32(),
            ir::ValueType::U64 | ir::ValueType::I64 => class.u64(),
            ir::ValueType::UPtr | ir::ValueType::IPtr => match mode {
                x86::Mode::X86 => class.u32(),
                x86::Mode::X8664 => class.u64(),
            },
        }
    }
}