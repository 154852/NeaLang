use crate::{FunctionTranslationContext, TranslationContext, registerify::{SYS_V_ABI_RET, reg_for_vt}};

impl TranslationContext {
    pub(crate) fn translate_instruction_to(&self, ir_ins: &ir::Ins, ftc: &mut FunctionTranslationContext, ins: &mut Vec<x86::Ins>) {
        let mode = ftc.mode();
        
        match ir_ins {
            ir::Ins::PushLocal(vt, idx) => {
                ins.push(x86::Ins::MovRegMem(
                    ftc.stack().push_vt(*vt),
                    ftc.local_mem(*idx)
                ));
            },
            ir::Ins::PopLocal(vt, idx) => {
                ins.push(x86::Ins::MovMemReg(
                    ftc.local_mem(*idx),
                    ftc.stack().pop_vt(*vt),
                ));
            },
            ir::Ins::PushGlobal(_, _, _) => todo!(),
            ir::Ins::PopGlobal(_, _, _) => todo!(),
            ir::Ins::Call(_) => todo!(),
            ir::Ins::Ret => {
                let rets_len = ftc.func().signature().returns().len();

                assert_eq!(ftc.stack().size(), rets_len);

                for (i, ret) in ftc.func().signature().returns().iter().enumerate() {
                    // TODO: There may be issues with multiple return values here, as rdx could be overwritten before it is read
                    ins.push(x86::Ins::MovRegReg(
                        reg_for_vt(*ret, mode, SYS_V_ABI_RET[rets_len - 1 - i]),
                        ftc.stack_ref().peek_at_vt(i, *ret)
                    ));
                }

                ftc.stack().zero();

                ins.push(x86::Ins::JumpLocalSymbol(ftc.local_symbols().root()));
            },
            ir::Ins::Inc(vt, i) => {
                ins.push(x86::Ins::AddRegImm(
                    ftc.stack().peek_vt(*vt),
                    *i,
                ));
            },
            ir::Ins::Dec(vt, i) => {
                ins.push(x86::Ins::SubRegImm(
                    ftc.stack().peek_vt(*vt),
                    *i,
                ));
            },
            ir::Ins::Add(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = a + b
                ins.push(x86::Ins::AddRegReg(
                    a, b,
                ));
            },
            ir::Ins::Mul(_) => todo!(),
            ir::Ins::Div(_) => todo!(),
            ir::Ins::Sub(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = a + b
                ins.push(x86::Ins::SubRegReg(
                    a, b,
                ));
            },
            ir::Ins::Loop(_, _, _) => todo!(),
            ir::Ins::If(_) => todo!(),
            ir::Ins::IfElse(_, _) => todo!(),
            ir::Ins::Break(_) => todo!(),
            ir::Ins::Continue(_) => todo!(),
            ir::Ins::PushLiteral(vt, val) => {
                ins.push(x86::Ins::MovRegImm(
                    ftc.stack().push_vt(*vt),
                    *val
                ));
            },
            ir::Ins::Drop => {
                ftc.stack().pop();
            },
        }
    }
}