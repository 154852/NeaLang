use crate::{FunctionTranslationContext, LocalSymbol, TranslationContext};

impl TranslationContext {
    fn insert_call(&self, idx: ir::FunctionIndex, ftc: &mut FunctionTranslationContext, ins: &mut Vec<x86::Ins>) {
        // TODO: This push/pop is quite unfortuante, but sort of required without a bit of optimisation to move calls to be done earlier, while the stack is empty

        let params = ftc.unit().get_function(idx).signature().params().len();
        ftc.stack().pop_many(params);
        
        let old_stack_size = ftc.stack().size();
        for i in 0..old_stack_size {
            ins.push(x86::Ins::PushReg(ftc.stack().at(i).u32()));
        }

        // Move param values to new places on stack
        for (i, param) in ftc.unit().get_function(idx).signature().params().iter().enumerate() {
            ins.push(x86::Ins::MovRegReg(
                crate::registerify::reg_for_vt(param, self.mode, crate::registerify::SYS_V_ABI[i]),
                ftc.stack_ref().at_vt(ftc.stack_ref().size() + i, param),
            ));
        }

        ins.push(x86::Ins::CallGlobalSymbol(ftc.symbol_id_for_function(idx)));

        // Move return values to new places on stack
        for (i, ret) in ftc.unit().get_function(idx).signature().returns().iter().enumerate() {
            ins.push(x86::Ins::MovRegReg(
                ftc.stack_ref().at_vt(ftc.stack_ref().size() + i, ret),
                crate::registerify::reg_for_vt(ret, self.mode, crate::registerify::SYS_V_ABI_RET[i]),
            ));
        }

        let returns = ftc.unit().get_function(idx).signature().returns().len();
        ftc.stack().push_many(returns);

        for i in 0..old_stack_size {
            ins.push(x86::Ins::PopReg(ftc.stack_ref().at(old_stack_size - i - 1).u32()));
        }
    }

    fn addr_in_path(&self, path: &ir::ValuePath, ftc: &mut FunctionTranslationContext, ins: &mut Vec<x86::Ins>) {
        match path.origin() {
            ir::ValuePathOrigin::Local(local, _local_type) => {
                ins.push(x86::Ins::LeaRegMem(
                    ftc.stack().push_ptr(),
                    ftc.local_mem(*local)
                ));
            },
            ir::ValuePathOrigin::Global(global, _global_type) => {
                ins.push(x86::Ins::LeaRegGlobalSymbol(
                    ftc.stack().push_ptr(),
                    ftc.symbol_id_for_global(*global)
                ));
            },
            ir::ValuePathOrigin::Deref(_deref_type) => {
            },
        };

        for component in path.components() {
            match component {
                ir::ValuePathComponent::Slice(slice_type) => {
                    let slice = ftc.stack().pop_ptr();
                    let index = ftc.stack().pop_ptr();
                        
                    ins.push(x86::Ins::MovRegMem(
                        slice,
                        x86::Mem::new().base(slice.class()),
                    ));
    
                    let addr = ftc.stack().push_ptr();
    
                    ins.push(x86::Ins::LeaRegMem(
                        addr,
                        x86::Mem::new().base(slice.class()).index(index.class()).scale(match crate::registerify::size_for_st(slice_type, self.mode) {
                            1 => 0,
                            2 => 1,
                            4 => 2,
                            8 => 3,
                            _ => todo!()
                        }),
                    ));
                },
                ir::ValuePathComponent::Property(idx, compound_type, _prop_type) => {
                    ins.push(x86::Ins::LeaRegMem(
                        ftc.stack().peek_ptr(),
                        x86::Mem::new().base(ftc.stack().peek()).disp(
                            crate::registerify::offset_of_prop(compound_type, *idx, ftc.mode()) as i64,
                        ),
                    ));
                },
                ir::ValuePathComponent::Length => {
                    ins.push(x86::Ins::LeaRegMem(
                        ftc.stack().peek_ptr(),
                        x86::Mem::new().base(ftc.stack().peek()).disp(
                            self.mode.ptr_size() as i64
                        ),
                    ));
                },
            }
        }
    }

    pub(crate) fn translate_instruction_to(&self, ir_ins: &ir::Ins, ftc: &mut FunctionTranslationContext, ins: &mut Vec<x86::Ins>) {
        match ir_ins {
            ir::Ins::PushPath(path, _vt) => {
                self.addr_in_path(path, ftc, ins); // Push the Path onto the stack
            },
            ir::Ins::Push(vt) => {
                let addr = ftc.stack().pop();

                // Deref
                ins.push(x86::Ins::MovRegMem(
                    ftc.stack().push_vt(vt),
                    x86::Mem::new().base(addr)
                ));
            },
            ir::Ins::Pop(vt) => {
                let val = ftc.stack().pop_vt(vt);
                let addr = ftc.stack().pop();
                // TODO: Forbid writing to the length of a slice

                // Write
                ins.push(x86::Ins::MovMemReg(
                    x86::Mem::new().base(addr),
                    val
                ));
            },
            ir::Ins::Index(_) => {
                // Do nothing, will be handled in addr_in_path
            },
            ir::Ins::New(st) => {
                ins.push(x86::Ins::MovRegImm(
                    ftc.stack().push_vt(&ir::ValueType::Ref(Box::new(ir::StorableType::Slice(Box::new(st.clone()))))),
                    crate::registerify::size_for_st(st, self.mode) as u64
                ));

                self.insert_call(ftc.unit().find_function_index("nl_new_object").expect("No nl_new_object included"), ftc, ins);
            },
            ir::Ins::NewSlice(st) => {
                ins.push(x86::Ins::MovRegImm(
                    ftc.stack().push_vt(&ir::ValueType::Ref(Box::new(ir::StorableType::Slice(Box::new(st.clone()))))),
                    crate::registerify::size_for_st(st, self.mode) as u64
                ));

                self.insert_call(ftc.unit().find_function_index("nl_new_slice").expect("No nl_new_slice included"), ftc, ins);
            },
            ir::Ins::Convert(from, to) => {
                let size_a = crate::registerify::size_for_vt(from, self.mode);
                let size_b = crate::registerify::size_for_vt(to, self.mode);

                // Only need to do anything if promoting to a higher size
                if size_b > size_a {
                    if to.signed() {
                        ins.push(x86::Ins::MovsxRegReg(ftc.stack().peek_vt(to), ftc.stack().peek_vt(from)));
                    } else if size_a != 4 { // No need to zero extend from 32 bits, as this has already happened (I think?)
                        ins.push(x86::Ins::MovzxRegReg(ftc.stack().peek_vt(to), ftc.stack().peek_vt(from)));
                    }
                }
            },
            ir::Ins::Call(idx) => self.insert_call(*idx, ftc, ins),
            ir::Ins::Ret => {
                let rets_len = ftc.func().signature().returns().len();

                assert_eq!(ftc.stack().size(), rets_len);

                for (i, ret) in ftc.func().signature().returns().iter().enumerate() {
                    // TODO: There may be issues with multiple return values here, as rdx could be overwritten before it is read
                    ins.push(x86::Ins::MovRegReg(
                        crate::registerify::reg_for_vt(ret, self.mode, crate::registerify::SYS_V_ABI_RET[rets_len - 1 - i]),
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