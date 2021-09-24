mod registerify;

use registerify::StackToReg;
use x86::{self};
use ir;

use crate::registerify::SYS_V_ABI_RET;

struct LocalSymbolStack {
    next: usize
}

impl LocalSymbolStack {
    fn new() -> LocalSymbolStack {
        LocalSymbolStack {
            next: 1
        }
    }

    fn root(&self) -> x86::LocalSymbolID {
        0
    }
}

trait X86ForIRIns {
    fn build_x86(&self, mode: x86::Mode, stack: &mut StackToReg, local_symbol_stack: &mut LocalSymbolStack, unit: &ir::TranslationUnit, function: &ir::Function, ins: &mut Vec<x86::Ins>);
}

impl X86ForIRIns for ir::Ins {
    fn build_x86(&self, mode: x86::Mode, stack: &mut StackToReg, local_symbol_stack: &mut LocalSymbolStack, unit: &ir::TranslationUnit, function: &ir::Function, ins: &mut Vec<x86::Ins>) {
        match self {
            ir::Ins::PushLocal(vt, idx) => {
                ins.push(x86::Ins::MovRegMem(
                    vt.x86_reg(mode, stack.push()),
                    function.local_mem(mode, *idx)
                ));
            },
            ir::Ins::PopLocal(vt, idx) => {
                ins.push(x86::Ins::MovMemReg(
                    function.local_mem(mode, *idx),
                    vt.x86_reg(mode, stack.pop())
                ));
            },
            ir::Ins::PushGlobal(_, _, _) => todo!(),
            ir::Ins::PopGlobal(_, _, _) => todo!(),
            ir::Ins::Call(_) => todo!(),
            ir::Ins::Ret => {
                let rets_len = function.signature().returns().len();

                assert_eq!(stack.size(), rets_len);

                for (i, ret) in function.signature().returns().iter().enumerate() {
                    // TODO: There may be issues with multiple return values here, as rdi could be overwritten before it is read
                    ins.push(x86::Ins::MovRegReg(
                        ret.x86_reg(mode, SYS_V_ABI_RET[rets_len - 1 - i]),
                        ret.x86_reg(mode, stack.pop()),
                    ));
                }

                ins.push(x86::Ins::JumpLocalSymbol(local_symbol_stack.root()));
            },
            ir::Ins::Inc(_, _) => todo!(),
            ir::Ins::Dec(_, _) => todo!(),
            ir::Ins::Add(vt) => {
                let b = stack.pop();
                let a = stack.peek();
                // a = a + b
                ins.push(x86::Ins::AddRegReg(
                    vt.x86_reg(mode, a),
                    vt.x86_reg(mode, b),
                ));
            },
            ir::Ins::Mul(_) => todo!(),
            ir::Ins::Div(_) => todo!(),
            ir::Ins::Sub(_) => todo!(),
            ir::Ins::Loop(_, _, _) => todo!(),
            ir::Ins::If(_) => todo!(),
            ir::Ins::IfElse(_, _) => todo!(),
            ir::Ins::Break(_) => todo!(),
            ir::Ins::Continue(_) => todo!(),
            ir::Ins::PushLiteral(vt, val) => {
                ins.push(x86::Ins::MovRegImm(
                    vt.x86_reg(mode, stack.push()),
                    *val
                ));
            },
            ir::Ins::Drop => {
                stack.pop();
            },
        }
    }
}

pub trait X86ForIRFunction {
    fn build_x86(&self, mode: x86::Mode, unit: &ir::TranslationUnit) -> Vec<x86::Ins>;
}

impl X86ForIRFunction for ir::Function {
    fn build_x86(&self, mode: x86::Mode, unit: &ir::TranslationUnit) -> Vec<x86::Ins> {
        let mut x86_ins = Vec::new();

        let mut stack = StackToReg::new();
        let mut local_symbol_stack = LocalSymbolStack::new();

        stack.push_many(self.signature().params().len());

        x86_ins.push(x86::Ins::PushReg(mode.base_ptr()));
        x86_ins.push(x86::Ins::MovRegReg(mode.base_ptr(), mode.stack_ptr()));
        x86_ins.push(x86::Ins::SubRegImm(mode.stack_ptr(), self.local_addr(mode, self.locals().len() - 1)));

        for ins in self.code() {
            ins.build_x86(mode, &mut stack, &mut local_symbol_stack, unit, self, &mut x86_ins);
        }

        x86_ins.push(x86::Ins::LocalSymbol(local_symbol_stack.root()));
        x86_ins.push(x86::Ins::MovRegReg(mode.stack_ptr(), mode.base_ptr()));
        x86_ins.push(x86::Ins::PopReg(mode.base_ptr()));
        x86_ins.push(x86::Ins::Ret);

        x86_ins
    }
}

trait X86ForIRFunctionInternal {
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

trait X86RegForValueType {
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