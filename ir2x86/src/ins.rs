use crate::{FunctionTranslationContext, LocalSymbol, TranslationContext, registerify::{SYS_V_ABI, SYS_V_ABI_RET, reg_for_vt}};

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
            ir::Ins::Call(idx) => {
                // TODO: This push/pop is quite unfortuante, but sort of required without a bit of optimisation to move calls to be done earlier, while the stack is empty

                let params = ftc.unit().get_function(*idx).signature().params().len();
                ftc.stack().pop_many(params);
                
                let old_stack_size = ftc.stack().size();
                for i in 0..old_stack_size {
                    ins.push(x86::Ins::PushReg(ftc.stack().at(i).u32()));
                }

                // Move param values to new places on stack
                for (i, param) in ftc.unit().get_function(*idx).signature().params().iter().enumerate() {
                    ins.push(x86::Ins::MovRegReg(
                        reg_for_vt(*param, mode, SYS_V_ABI[i]),
                        ftc.stack_ref().at_vt(ftc.stack_ref().size() + i, *param),
                    ));
                }

                ins.push(x86::Ins::CallGlobalSymbol(*idx));

                // Move return values to new places on stack
                for (i, ret) in ftc.unit().get_function(*idx).signature().returns().iter().enumerate() {
                    ins.push(x86::Ins::MovRegReg(
                        ftc.stack_ref().at_vt(ftc.stack_ref().size() + i, *ret),
                        reg_for_vt(*ret, mode, SYS_V_ABI_RET[i]),
                    ));
                }

                let returns = ftc.unit().get_function(*idx).signature().returns().len();
                ftc.stack().push_many(returns);

                for i in 0..old_stack_size {
                    ins.push(x86::Ins::PopReg(ftc.stack_ref().at(old_stack_size - i - 1).u32()));
                }
            },
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

                // Root is always 0
                ins.push(x86::Ins::JumpLocalSymbol(0));
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
            ir::Ins::Mul(vt) => {
                if vt.signed() {
                    let b = ftc.stack().pop_vt(*vt);
                    let a = ftc.stack().peek_vt(*vt);
                    // a = a * b
                    ins.push(x86::Ins::IMulRegReg(
                        a, b,
                    ));
                } else {
                    todo!()
                }
            },
            ir::Ins::Div(_) => todo!(),
            ir::Ins::Sub(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = a + b
                ins.push(x86::Ins::SubRegReg(
                    a, b,
                ));
            },
            ir::Ins::Eq(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::Zero, a.class()));
            },
            ir::Ins::Ne(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::NotZero, a.class()));
            },
            ir::Ins::Lt(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::Less, a.class()));
            },
            ir::Ins::Le(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::LessOrEqual, a.class()));
            },
            ir::Ins::Gt(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::Greater, a.class()));
            },
            ir::Ins::Ge(vt) => {
                let b = ftc.stack().pop_vt(*vt);
                let a = ftc.stack().peek_vt(*vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::GreaterOrEqual, a.class()));
            },
            ir::Ins::Loop(body, condition, increment) => {
                let start = ftc.new_local_symbol();
                ins.push(x86::Ins::LocalSymbol(start));

                for inner_ins in condition {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }

                let cond = ftc.stack().pop_vt(ir::ValueType::Bool);
                ins.push(x86::Ins::TestRegReg(cond, cond));

                let final_end = ftc.new_local_symbol();
                ins.push(x86::Ins::JumpConditionalLocalSymbol(x86::Condition::Zero, final_end));

                let inc_start = ftc.new_local_symbol();

                ftc.local_symbols().push(LocalSymbol::Loop(
                    inc_start, final_end
                ));

                for inner_ins in body {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }
                
                ftc.local_symbols().pop();

                ins.push(x86::Ins::LocalSymbol(inc_start));

                for inner_ins in increment {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }

                ins.push(x86::Ins::JumpLocalSymbol(start));

                ins.push(x86::Ins::LocalSymbol(final_end));
            },
            ir::Ins::If(then) => {
                let cond = ftc.stack().pop_vt(ir::ValueType::Bool);
                ins.push(x86::Ins::TestRegReg(cond, cond));
                
                let end = ftc.new_local_symbol();
                ins.push(x86::Ins::JumpConditionalLocalSymbol(x86::Condition::Zero, end));

                ftc.local_symbols().push(LocalSymbol::If);
                for inner_ins in then {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }
                ftc.local_symbols().pop();

                ins.push(x86::Ins::LocalSymbol(end));
            },
            ir::Ins::IfElse(true_then, false_then) => {
                let cond = ftc.stack().pop_vt(ir::ValueType::Bool);
                ins.push(x86::Ins::TestRegReg(cond, cond));
                
                let end = ftc.new_local_symbol();
                let false_start = ftc.new_local_symbol();

                // if false, jump to false_start...
                ins.push(x86::Ins::JumpConditionalLocalSymbol(x86::Condition::Zero, false_start));

                // otherwise (if) true, continue...
                ftc.local_symbols().push(LocalSymbol::If);
                for inner_ins in true_then {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }
                ftc.local_symbols().pop();
                // then jump to the end
                ins.push(x86::Ins::JumpLocalSymbol(end));

                ins.push(x86::Ins::LocalSymbol(false_start));
                ftc.local_symbols().push(LocalSymbol::If);
                for inner_ins in false_then {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }
                ftc.local_symbols().pop();
                // just continue to end

                ins.push(x86::Ins::LocalSymbol(end));
            },
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