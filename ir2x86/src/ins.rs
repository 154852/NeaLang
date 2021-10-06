use crate::{FunctionTranslationContext, LocalSymbol, TranslationContext};

impl TranslationContext {
    pub(crate) fn translate_instruction_to(&self, ir_ins: &ir::Ins, ftc: &mut FunctionTranslationContext, ins: &mut Vec<x86::Ins>) {
        let mode = ftc.mode();
        
        match ir_ins {
            ir::Ins::PushLocalValue(vt, idx) => {
                ins.push(x86::Ins::MovRegMem(
                    ftc.stack().push_vt(vt),
                    ftc.local_mem(*idx)
                ));
            },
            ir::Ins::PushLocalRef(st, idx) => {
                ins.push(x86::Ins::LeaRegMem(
                    ftc.stack().push_vt(&ir::ValueType::Ref(Box::new(st.clone()))),
                    ftc.local_mem(*idx)
                ));
            },
            ir::Ins::PopLocalValue(vt, idx) => {
                ins.push(x86::Ins::MovMemReg(
                    ftc.local_mem(*idx),
                    ftc.stack().pop_vt(vt),
                ));
            },
            ir::Ins::PopRef(vt) => {
                let val = ftc.stack().pop_vt(vt);
                let addr = ftc.stack().pop_vt(&ir::ValueType::Ref(Box::new(ir::StorableType::Value(vt.clone()))));
                ins.push(x86::Ins::MovMemReg(
                    x86::Mem::new().base(addr.class()),
                    val,
                ));
            },
            ir::Ins::PushProperty(ct, vt, idx) => {
                ins.push(x86::Ins::MovRegMem(
                    ftc.stack().peek_vt(vt),
                    x86::Mem::new().base(ftc.stack().peek()).disp(
                        crate::registerify::offset_of_prop(ct, *idx, ftc.mode()) as i64,
                    ),
                ));
            },
            ir::Ins::PushPropertyRef(ct, st, idx) => {
                ins.push(x86::Ins::LeaRegMem(
                    ftc.stack().peek_vt(&ir::ValueType::Ref(Box::new(st.clone()))),
                    x86::Mem::new().base(ftc.stack().peek()).disp(
                        crate::registerify::offset_of_prop(ct, *idx, ftc.mode()) as i64,
                    ),
                ));
            },
            // Slices are a packed struct, the ptr to the first element of the slice, followed by the length.
            ir::Ins::PushSliceLen(_) => {
                ins.push(x86::Ins::MovRegMem(
                    ftc.stack().peek_vt(&ir::ValueType::UPtr),
                    x86::Mem::new().base(ftc.stack().peek()).disp(
                        mode.ptr_size() as i64
                    ),
                ));
            },
            ir::Ins::PushSliceElement(st) => {
                let index = ftc.stack().pop_vt(&ir::ValueType::UPtr);
                let slice = ftc.stack().pop_vt(&ir::ValueType::Ref(Box::new(ir::StorableType::Slice(Box::new(st.clone())))));

                let data = ftc.stack().push().uptr(&mode);
                
                ins.push(x86::Ins::MovRegMem(
                    slice,
                    x86::Mem::new().base(slice.class()),
                ));

                ins.push(x86::Ins::MovRegMem(
                    data,
                    x86::Mem::new().base(slice.class()).index(index.class()).scale(match crate::registerify::size_for_st(st, mode) {
                        1 => 0,
                        2 => 1,
                        4 => 2,
                        8 => 3,
                        _ => todo!()
                    }),
                ));
            },
            ir::Ins::PushSliceElementRef(st) => {
                let index = ftc.stack().pop_vt(&ir::ValueType::UPtr);
                let slice = ftc.stack().pop_vt(&ir::ValueType::Ref(Box::new(ir::StorableType::Slice(Box::new(st.clone())))));

                let data = ftc.stack().push().uptr(&mode);
                
                ins.push(x86::Ins::MovRegMem(
                    slice,
                    x86::Mem::new().base(slice.class()),
                ));

                ins.push(x86::Ins::LeaRegMem(
                    data,
                    x86::Mem::new().base(slice.class()).index(index.class()).scale(match crate::registerify::size_for_st(st, mode) {
                        1 => 0,
                        2 => 1,
                        4 => 2,
                        8 => 3,
                        _ => todo!()
                    }),
                ));
            },
            ir::Ins::Convert(from, to) => {
                let size_a = crate::registerify::size_for_vt(from, mode);
                let size_b = crate::registerify::size_for_vt(to, mode);

                // Only need to do anything if promoting to a higher size
                if size_b > size_a {
                    if to.signed() {
                        ins.push(x86::Ins::MovsxRegReg(ftc.stack().peek_vt(to), ftc.stack().peek_vt(from)));
                    } else if size_a != 4 { // No need to zero extend from 32 bits, as this has already happened (I think?)
                        ins.push(x86::Ins::MovzxRegReg(ftc.stack().peek_vt(to), ftc.stack().peek_vt(from)));
                    }
                }
            },
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
                        crate::registerify::reg_for_vt(param, mode, crate::registerify::SYS_V_ABI[i]),
                        ftc.stack_ref().at_vt(ftc.stack_ref().size() + i, param),
                    ));
                }

                ins.push(x86::Ins::CallGlobalSymbol(*idx));

                // Move return values to new places on stack
                for (i, ret) in ftc.unit().get_function(*idx).signature().returns().iter().enumerate() {
                    ins.push(x86::Ins::MovRegReg(
                        ftc.stack_ref().at_vt(ftc.stack_ref().size() + i, ret),
                        crate::registerify::reg_for_vt(ret, mode, crate::registerify::SYS_V_ABI_RET[i]),
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
                        crate::registerify::reg_for_vt(ret, mode, crate::registerify::SYS_V_ABI_RET[rets_len - 1 - i]),
                        ftc.stack_ref().peek_at_vt(i, ret)
                    ));
                }

                ftc.stack().zero();

                // Root is always 0
                ins.push(x86::Ins::JumpLocalSymbol(0));
            },
            ir::Ins::Inc(vt, i) => {
                ins.push(x86::Ins::AddRegImm(
                    ftc.stack().peek_vt(vt),
                    *i,
                ));
            },
            ir::Ins::Dec(vt, i) => {
                ins.push(x86::Ins::SubRegImm(
                    ftc.stack().peek_vt(vt),
                    *i,
                ));
            },
            ir::Ins::Add(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                // a = a + b
                ins.push(x86::Ins::AddRegReg(
                    a, b,
                ));
            },
            ir::Ins::Mul(vt) => {
                if vt.signed() {
                    let b = ftc.stack().pop_vt(vt);
                    let a = ftc.stack().peek_vt(vt);
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
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                // a = a + b
                ins.push(x86::Ins::SubRegReg(
                    a, b,
                ));
            },
            ir::Ins::Eq(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::Zero, a.class()));
            },
            ir::Ins::Ne(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::NotZero, a.class()));
            },
            ir::Ins::Lt(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::Less, a.class()));
            },
            ir::Ins::Le(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::LessOrEqual, a.class()));
            },
            ir::Ins::Gt(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                // a = (a == b)
                ins.push(x86::Ins::CmpRegReg(a, b));
                ins.push(x86::Ins::ConditionalSet(x86::Condition::Greater, a.class()));
            },
            ir::Ins::Ge(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
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

                let cond = ftc.stack().pop_vt(&ir::ValueType::Bool);
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
                let cond = ftc.stack().pop_vt(&ir::ValueType::Bool);
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
                let cond = ftc.stack().pop_vt(&ir::ValueType::Bool);
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
                    ftc.stack().push_vt(vt),
                    *val
                ));
            },
            ir::Ins::Drop => {
                ftc.stack().pop();
            },
        }
    }
}