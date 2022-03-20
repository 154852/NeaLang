use crate::{FunctionTranslationContext, LocalSymbol, TranslationContext};

impl TranslationContext {
    fn insert_call(&self, idx: ir::FunctionIndex, ftc: &mut FunctionTranslationContext, insns: &mut Vec<x86::Ins>) {
        // TODO: This push/pop is quite unfortuante, but sort of required without a bit of optimisation to move calls to be done earlier, while the stack is empty

        let params = ftc.unit().get_function(idx).unwrap().signature().param_count();
        ftc.stack().pop_many(params);
        
        let old_stack_size = ftc.stack().size();
        for i in 0..old_stack_size {
            insns.push(x86::Ins::PushReg(ftc.stack().at(i).u32()));
        }

        // Move param values to new places on stack
        for (i, param) in ftc.unit().get_function(idx).unwrap().signature().params().iter().enumerate() {
            insns.push(x86::Ins::MovRegReg(
                crate::util::reg_for_value_type(param, self.mode, crate::registerify::SYS_V_ABI[i]),
                ftc.stack_ref().at_vt(ftc.stack_ref().size() + i, param),
            ));
        }

        // FIXME: 32 bit
        if ftc.should_align_16_byte() && old_stack_size % 2 != 0 {
            insns.push(x86::Ins::SubRegImm(x86::Reg::Rsp, 8));
        }

        insns.push(x86::Ins::CallGlobalSymbol(ftc.symbol_id_for_function(idx)));

        if ftc.should_align_16_byte() && old_stack_size % 2 != 0 {
            insns.push(x86::Ins::AddRegImm(x86::Reg::Rsp, 8));
        }

        // Move return values to new places on stack
        for (i, ret) in ftc.unit().get_function(idx).unwrap().signature().returns().iter().enumerate() {
            insns.push(x86::Ins::MovRegReg(
                ftc.stack_ref().at_vt(ftc.stack_ref().size() + i, ret),
                crate::util::reg_for_value_type(ret, self.mode, crate::registerify::SYS_V_ABI_RET[i]),
            ));
        }

        let returns = ftc.unit().get_function(idx).unwrap().signature().return_count();
        ftc.stack().push_many(returns);

        for i in 0..old_stack_size {
            insns.push(x86::Ins::PopReg(ftc.stack_ref().at(old_stack_size - i - 1).u32()));
        }
    }

    fn addr_in_path(&self, path: &ir::ValuePath, ftc: &mut FunctionTranslationContext, insns: &mut Vec<x86::Ins>) {
        match path.origin() {
            ir::ValuePathOrigin::Local(local, _local_type) => {
                insns.push(x86::Ins::LeaRegMem(
                    ftc.stack().push_ptr(),
                    ftc.local_mem(*local)
                ));
            },
            ir::ValuePathOrigin::Global(global, _global_type) => {
                insns.push(x86::Ins::LeaRegGlobalSymbol(
                    ftc.stack().push_ptr(),
                    ftc.symbol_id_for_global(*global)
                ));
            },
            ir::ValuePathOrigin::Deref(_deref_type) => {
                // Do nothing, address is already there
            },
        };

        for component in path.components() {
            match component {
                ir::ValuePathComponent::Slice(slice_type) => {
                    let slice = ftc.stack().pop_ptr();
                    let index = ftc.stack().pop_ptr();
                        
                    insns.push(x86::Ins::MovRegMem(
                        slice,
                        x86::Mem::new().base(slice.class()),
                    ));
    
                    let addr = ftc.stack().push_ptr();
    
                    insns.push(x86::Ins::LeaRegMem(
                        addr,
                        x86::Mem::new().base(slice.class()).index(index.class()).scale(match crate::util::size_for_storable_type(slice_type, self.mode) {
                            1 => 0,
                            2 => 1,
                            4 => 2,
                            8 => 3,
                            _ => todo!() // Use multiplication
                        }),
                    ));
                },
                ir::ValuePathComponent::Property(idx, compound_type, _prop_type) => {
                    insns.push(x86::Ins::LeaRegMem(
                        ftc.stack().peek_ptr(),
                        x86::Mem::new().base(ftc.stack().peek()).disp(
                            crate::util::offset_of_compound_property(compound_type, *idx, ftc.mode()) as i64,
                        ),
                    ));
                },
                ir::ValuePathComponent::Length => {
                    insns.push(x86::Ins::LeaRegMem(
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
                    crate::util::size_for_storable_type(st, self.mode) as u64
                ));

                self.insert_call(ftc.unit().find_alloc().expect("No alloc implementation included"), ftc, ins);
            },
            ir::Ins::NewSlice(st) => {
                ins.push(x86::Ins::MovRegImm(
                    ftc.stack().push_vt(&ir::ValueType::Ref(Box::new(ir::StorableType::Slice(Box::new(st.clone()))))),
                    crate::util::size_for_storable_type(st, self.mode) as u64
                ));

                self.insert_call(ftc.unit().find_alloc_slice().expect("No alloc slice implementation included"), ftc, ins);
            },
            ir::Ins::Free(st) => {
                ins.push(x86::Ins::MovRegImm(
                    ftc.stack().push_ptr(),
                    crate::util::size_for_storable_type(st, self.mode) as u64
                ));
                self.insert_call(ftc.unit().find_free().expect("No free implementation included"), ftc, ins);
            },
            ir::Ins::FreeSlice(st) => {
                ins.push(x86::Ins::MovRegImm(
                    ftc.stack().push_ptr(),
                    crate::util::size_for_storable_type(st, self.mode) as u64
                ));
                self.insert_call(ftc.unit().find_free_slice().expect("No free slice implementation included"), ftc, ins);
            },
            ir::Ins::Convert(from, to) => {
                let size_a = crate::util::size_for_value_type(from, self.mode);
                let size_b = crate::util::size_for_value_type(to, self.mode);

                // Only need to do anything if promoting to a higher size
                if size_b > size_a {
                    if to.is_signed() {
                        ins.push(x86::Ins::MovsxRegReg(ftc.stack().peek_vt(to), ftc.stack().peek_vt(from)));
                    } else if size_a != 4 { // No need to zero extend from 32 bits, as this has already happened (I think?)
                        ins.push(x86::Ins::MovzxRegReg(ftc.stack().peek_vt(to), ftc.stack().peek_vt(from)));
                    }
                }
            },
            ir::Ins::Call(idx) => self.insert_call(*idx, ftc, ins),
            ir::Ins::Ret => {
                let rets_len = ftc.func().signature().return_count();

                assert_eq!(ftc.stack().size(), rets_len);

                for (i, ret) in ftc.func().signature().returns().iter().enumerate() {
                    // TODO: There may be issues with multiple return values here, as rdx could be overwritten before it is read
                    ins.push(x86::Ins::MovRegReg(
                        crate::util::reg_for_value_type(ret, self.mode, crate::registerify::SYS_V_ABI_RET[rets_len - 1 - i]),
                        ftc.stack_ref().peek_at_vt(i, ret)
                    ));
                }

                ftc.stack().zero();

                // Root is always 0
                ins.push(x86::Ins::JumpLocalSymbol(x86::LocalSymbolID::new(0)));
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
                if vt.is_signed() {
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
            ir::Ins::Div(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                
                ins.push(x86::Ins::PushReg(x86::Reg::Rax));
                
                let uses_rdx = ftc.stack().uses(x86::RegClass::Edx);
                if uses_rdx { ins.push(x86::Ins::PushReg(x86::Reg::Rdx)); }

                let uses_rcx = ftc.stack().uses(x86::RegClass::Ecx);
                if uses_rcx { ins.push(x86::Ins::PushReg(x86::Reg::Rcx)); }

                // ecx = b
                ins.push(x86::Ins::MovRegReg(
                    crate::util::reg_for_value_type(vt, self.mode, x86::RegClass::Ecx), b
                ));

                // eax = a 
                ins.push(x86::Ins::MovRegReg(
                    crate::util::reg_for_value_type(vt, self.mode, x86::RegClass::Eax), a
                ));
                ins.push(x86::Ins::Cdq(a.size()));
                // eax = eax / ecx
                if vt.is_signed() {
                        ins.push(x86::Ins::IDivReg(
                        crate::util::reg_for_value_type(vt, self.mode, x86::RegClass::Ecx)
                    ));
                } else {
                    ins.push(x86::Ins::DivReg(
                        crate::util::reg_for_value_type(vt, self.mode, x86::RegClass::Ecx)
                    ));
                }

                // a = eax
                ins.push(x86::Ins::MovRegReg(
                    a, crate::util::reg_for_value_type(vt, self.mode, x86::RegClass::Eax)
                ));

                if uses_rcx {
                    if a.class() == x86::RegClass::Ecx {
                        ins.push(x86::Ins::AddRegImm(x86::Reg::Rsp, self.mode.ptr_size() as u64));
                    } else {
                        ins.push(x86::Ins::PopReg(x86::Reg::Rcx));
                    }
                }

                if uses_rdx {
                    if a.class() == x86::RegClass::Edx {
                        ins.push(x86::Ins::AddRegImm(x86::Reg::Rsp, self.mode.ptr_size() as u64));
                    } else {
                        ins.push(x86::Ins::PopReg(x86::Reg::Rdx));
                    }
                }

                if a.class() == x86::RegClass::Eax {
                    ins.push(x86::Ins::AddRegImm(x86::Reg::Rsp, self.mode.ptr_size() as u64));
                } else {
                    ins.push(x86::Ins::PopReg(x86::Reg::Rax));
                }
            },
            ir::Ins::Sub(vt) => {
                let b = ftc.stack().pop_vt(vt);
                let a = ftc.stack().peek_vt(vt);
                // a = a + b
                ins.push(x86::Ins::SubRegReg(
                    a, b,
                ));
            },
            ir::Ins::Neg(vt) => {
                let a = ftc.stack().peek_vt(vt);
                // a = -a
                ins.push(x86::Ins::NegReg(
                    a
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
            ir::Ins::BoolAnd => {
                let b = ftc.stack().pop().u8();
                let a = ftc.stack().peek().u8();
                // a = a & b
                ins.push(x86::Ins::AndRegReg(
                    a, b,
                ));
            },
            ir::Ins::BoolOr => {
                let b = ftc.stack().pop().u8();
                let a = ftc.stack().peek().u8();
                // a = a + b
                ins.push(x86::Ins::OrRegReg(
                    a, b,
                ));
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
            ir::Ins::If(then, condition) => {
                for inner_ins in condition {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }

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
            ir::Ins::IfElse(true_then, false_then, condition) => {
                for inner_ins in condition {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }

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