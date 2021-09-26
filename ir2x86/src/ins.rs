use crate::{LocalSymbolStack, registerify::{SYS_V_ABI_RET, StackToReg}, unit::{X86ForIRFunctionInternal, X86RegForValueType}};

pub(crate) trait X86ForIRIns {
    fn build_x86(&self, mode: x86::Mode, stack: &mut StackToReg, local_symbol_stack: &mut LocalSymbolStack, unit: &ir::TranslationUnit, function: &ir::Function, ins: &mut Vec<x86::Ins>);
}

impl X86ForIRIns for ir::Ins {
    fn build_x86(&self, mode: x86::Mode, stack: &mut StackToReg, local_symbol_stack: &mut LocalSymbolStack, _unit: &ir::TranslationUnit, function: &ir::Function, ins: &mut Vec<x86::Ins>) {
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
                    // TODO: There may be issues with multiple return values here, as rdx could be overwritten before it is read
                    ins.push(x86::Ins::MovRegReg(
                        ret.x86_reg(mode, SYS_V_ABI_RET[rets_len - 1 - i]),
                        ret.x86_reg(mode, stack.pop()),
                    ));
                }

                ins.push(x86::Ins::JumpLocalSymbol(local_symbol_stack.root()));
            },
            ir::Ins::Inc(vt, i) => {
                ins.push(x86::Ins::AddRegImm(
                    vt.x86_reg(mode, stack.peek()),
                    *i,
                ));
            },
            ir::Ins::Dec(vt, i) => {
                ins.push(x86::Ins::SubRegImm(
                    vt.x86_reg(mode, stack.peek()),
                    *i,
                ));
            },
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
            ir::Ins::Sub(vt) => {
                let b = stack.pop();
                let a = stack.peek();
                // a = a - b
                ins.push(x86::Ins::SubRegReg(
                    vt.x86_reg(mode, a),
                    vt.x86_reg(mode, b),
                ));
            },
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