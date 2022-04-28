use crate::{FunctionTranslationContext, TranslationContext, util::size_for_value_type, LocalSymbol};

impl TranslationContext {
    fn insert_call(&self, idx: ir::FunctionIndex, ftc: &mut FunctionTranslationContext, insns: &mut Vec<arm64::Ins>) {
        // TODO: This push/pop is quite unfortuante, but sort of required without a bit of optimisation to move calls to be done earlier, while the stack is empty

        let params = ftc.unit().get_function(idx).unwrap().signature().param_count();
        ftc.stack().pop_many(params);
        
        let old_stack_size = ftc.stack().size();
        let mut ajd_stack_size = old_stack_size as u32 * 8;
        if ajd_stack_size & 15 != 0 {
            ajd_stack_size += 16 - (ajd_stack_size as u32 & 15);
        }

        insns.push(arm64::Ins::SubImm {
            size: arm64::SizeFlag::Size64,
            shift: arm64::ImmShift::Shift0,
            dest: arm64::Reg::sp(),
            src: arm64::Reg::sp(),
            val: ajd_stack_size
        });
        for i in 0..old_stack_size {
            insns.push(arm64::Ins::Stur {
                size: arm64::SizeFlag::Size64,
                base: arm64::Reg::sp(),
                offset: i as i32 * 8,
                src: ftc.stack().at(i)
            });
        }

        // Move param values to new places on stack
        for (i, _) in ftc.unit().get_function(idx).unwrap().signature().params().iter().enumerate() {
            insns.push(arm64::Ins::Mov {
                size: arm64::SizeFlag::Size64,
                dest: arm64::Reg(i as u32),
                src: ftc.stack_ref().at(ftc.stack_ref().size() + i)
            });
        }

        insns.push(arm64::Ins::BranchLinkGlobalSymbol(ftc.symbol_id_for_function(idx)));

        // Move return values to new places on stack
        for (i, _) in ftc.unit().get_function(idx).unwrap().signature().returns().iter().enumerate() {
            insns.push(arm64::Ins::Mov {
                size: arm64::SizeFlag::Size64,
                src: arm64::Reg(i as u32),
                dest: ftc.stack_ref().at(ftc.stack_ref().size() + i)
            });
        }

        let returns = ftc.unit().get_function(idx).unwrap().signature().return_count();
        ftc.stack().push_many(returns);

        for i in 0..old_stack_size {
            insns.push(arm64::Ins::Ldur {
                size: arm64::SizeFlag::Size64,
                base: arm64::Reg::sp(),
                offset: i as i32 * 8,
                dest: ftc.stack().at(i)
            });
        }

        insns.push(arm64::Ins::AddImm {
            size: arm64::SizeFlag::Size64,
            shift: arm64::ImmShift::Shift0,
            dest: arm64::Reg::sp(),
            src: arm64::Reg::sp(),
            val: ajd_stack_size
        });
    }

    fn addr_in_path(&self, path: &ir::ValuePath, ftc: &mut FunctionTranslationContext, insns: &mut Vec<arm64::Ins>) {
        match path.origin() {
            ir::ValuePathOrigin::Local(local, _local_type) => {
                insns.push(arm64::Ins::SubImm {
                    size: arm64::SizeFlag::Size64,
                    dest: ftc.stack().push(),
                    shift: arm64::ImmShift::Shift0,
                    src: arm64::Reg::fp(),
                    val: ftc.local_addr(*local)
                });
            },
            ir::ValuePathOrigin::Global(global, _global_type) => {
                let addr = ftc.stack().push();
                insns.push(arm64::Ins::AdrpGlobalSymbol(ftc.symbol_id_for_global(*global), addr));
                insns.push(arm64::Ins::AddPageOffGlobalSymbol { src: addr, dest: addr, symbol: ftc.symbol_id_for_global(*global) });
            },
            ir::ValuePathOrigin::Deref(_deref_type) => {
                // Do nothing, address is already there
            },
        };

        for component in path.components() {
            match component {
                ir::ValuePathComponent::Slice(slice_type) => {
                    let slice = ftc.stack().peek_at(0);
                    let index = ftc.stack().peek_at(1);

                    let tmp = ftc.stack().push();
                        
                    insns.push(arm64::Ins::Ldur {
                        size: arm64::SizeFlag::Size64,
                        dest: slice,
                        base: slice,
                        offset: 0
                    });

                    insns.push(arm64::Ins::MovZ {
                        size: arm64::SizeFlag::Size64,
                        dest: tmp,
                        val: crate::util::size_for_storable_type(slice_type) as u32,
                        shift: 0
                    });
                    
                    ftc.stack().pop_many(3);
                    let dest = ftc.stack().push();
    
                    insns.push(arm64::Ins::MAdd {
                        size: arm64::SizeFlag::Size64,
                        addend: slice,
                        mul1: index,
                        mul2: tmp,
                        dest
                    });
                },
                ir::ValuePathComponent::Property(idx, compound_type, _prop_type) => {
                    let addr = ftc.stack().peek();
                    
                    insns.push(arm64::Ins::AddImm {
                        size: arm64::SizeFlag::Size64,
                        dest: addr,
                        src: addr,
                        val: crate::util::offset_of_compound_property(compound_type, *idx) as u32,
                        shift: arm64::ImmShift::Shift0
                    });
                },
                ir::ValuePathComponent::Length => {
                    let addr = ftc.stack().peek();

                    insns.push(arm64::Ins::AddImm {
                        size: arm64::SizeFlag::Size64,
                        dest: addr,
                        src: addr,
                        val: 8,
                        shift: arm64::ImmShift::Shift0
                    });
                },
            }
        }
    }

    pub(crate) fn translate_instruction_to(&self, ir_ins: &ir::Ins, ftc: &mut FunctionTranslationContext, ins: &mut Vec<arm64::Ins>) {
        macro_rules! cmp {
            ($vt:expr, $ftc:expr, $ins:expr, $cond:ident) => {
                let rhs = $ftc.stack().pop();
                let lhs = $ftc.stack().peek();

                $ins.push(match size_for_value_type($vt) {
                    8 => arm64::Ins::SubsShifted {
                        size: arm64::SizeFlag::Size64,
                        shift_mode: arm64::ShiftMode::LogicalLeft,
                        dest: arm64::Reg::zero(),
                        src: rhs,
                        shifted_src: lhs,
                        shift: 0
                    },
                    _ => arm64::Ins::SubsShifted {
                        size: arm64::SizeFlag::Size32,
                        shift_mode: arm64::ShiftMode::LogicalLeft,
                        dest: arm64::Reg::zero(),
                        src: rhs,
                        shifted_src: lhs,
                        shift: 0
                    }
                });
                
                ins.push(arm64::Ins::CSInc {
                    size: arm64::SizeFlag::Size64,
                    cond: arm64::Condition::$cond.inv(),
                    inc_reg: arm64::Reg::zero(),
                    true_reg: arm64::Reg::zero(),
                    dest: lhs
                });
            }
        }

        match ir_ins {
            ir::Ins::PushPath(path, _vt) => {
                self.addr_in_path(path, ftc, ins); // Push the Path onto the stack
            },
            ir::Ins::Push(vt) => {
                let val = ftc.stack().peek();

                // Deref
                ins.push(match crate::util::size_for_value_type(vt) {
                    8 =>
                        arm64::Ins::Ldur {
                            size: arm64::SizeFlag::Size64,
                            base: val,
                            offset: 0,
                            dest: val
                        },
                    4 =>
                        arm64::Ins::Ldur {
                            size: arm64::SizeFlag::Size32,
                            base: val,
                            offset: 0,
                            dest: val
                        },
                    2 => 
                        arm64::Ins::Ldurh {
                            base: val,
                            offset: 0,
                            dest: val
                        },
                    1 =>
                        arm64::Ins::Ldurb {
                            base: val,
                            offset: 0,
                            dest: val
                        },
                    _ => unreachable!()
                });
            },
            ir::Ins::Pop(vt) => {
                let val = ftc.stack().pop();
                let addr = ftc.stack().pop();

                // Deref
                ins.push(match crate::util::size_for_value_type(vt) {
                    8 =>
                        arm64::Ins::Stur {
                            size: arm64::SizeFlag::Size64,
                            base: addr,
                            offset: 0,
                            src: val
                        },
                    4 =>
                        arm64::Ins::Stur {
                            size: arm64::SizeFlag::Size32,
                            base: addr,
                            offset: 0,
                            src: val
                        },
                    2 =>
                        arm64::Ins::Sturh {
                            base: addr,
                            offset: 0,
                            src: val
                        },
                    1 =>
                        arm64::Ins::Sturb {
                            base: addr,
                            offset: 0,
                            src: val
                        },
                    _ => unreachable!()
                });
            },
            ir::Ins::Index(_) => {
                // Do nothing, will be handled in addr_in_path
            },
            ir::Ins::New(st) => {
                ins.push(arm64::Ins::MovZ {
                    size: arm64::SizeFlag::Size64,
                    dest: ftc.stack().push(),
                    val: crate::util::size_for_storable_type(st) as u32,
                    shift: 0
                });

                self.insert_call(ftc.unit().find_alloc().expect("No alloc implementation included"), ftc, ins);
            },
            ir::Ins::NewSlice(st) => {
                ins.push(arm64::Ins::MovZ {
                    size: arm64::SizeFlag::Size64,
                    dest: ftc.stack().push(),
                    val: crate::util::size_for_storable_type(st) as u32,
                    shift: 0
                });

                self.insert_call(ftc.unit().find_alloc_slice().expect("No alloc implementation included"), ftc, ins);
            },
            ir::Ins::Free(st) => {
                ins.push(arm64::Ins::MovZ {
                    size: arm64::SizeFlag::Size64,
                    dest: ftc.stack().push(),
                    val: crate::util::size_for_storable_type(st) as u32,
                    shift: 0
                });

                self.insert_call(ftc.unit().find_free().expect("No free implementation included"), ftc, ins);
            },
            ir::Ins::FreeSlice(st) => {
                ins.push(arm64::Ins::MovZ {
                    size: arm64::SizeFlag::Size64,
                    dest: ftc.stack().push(),
                    val: crate::util::size_for_storable_type(st) as u32,
                    shift: 0
                });

                self.insert_call(ftc.unit().find_free_slice().expect("No free slice implementation included"), ftc, ins);
            },
            ir::Ins::Convert(_from, _to) => {
                // println!("WARNING Ignoring Convert");
            },
            ir::Ins::Call(idx) => self.insert_call(*idx, ftc, ins),
            ir::Ins::Ret => {
                ftc.stack().zero();
                ins.push(arm64::Ins::BranchLocalSymbol(arm64::LocalSymbolID::new(0)));
            },
            ir::Ins::Inc(_vt, i) => {
                ins.push(arm64::Ins::AddImm {
                    size: arm64::SizeFlag::Size64,
                    src: ftc.stack().peek(),
                    dest: ftc.stack().peek(),
                    val: *i as u32,
                    shift: arm64::ImmShift::Shift0
                });
            },
            ir::Ins::Dec(_vt, i) => {
                ins.push(arm64::Ins::SubImm {
                    size: arm64::SizeFlag::Size64,
                    src: ftc.stack().peek(),
                    dest: ftc.stack().peek(),
                    val: *i as u32,
                    shift: arm64::ImmShift::Shift0
                });
            },
            ir::Ins::Add(_vt) => {
                let rhs = ftc.stack().pop();
                let lhs = ftc.stack().peek();

                ins.push(arm64::Ins::AddShifted {
                    size: arm64::SizeFlag::Size64,
                    shift_mode: arm64::ShiftMode::LogicalLeft,
                    shift: 0,
                    dest: lhs,
                    src: rhs,
                    shifted_src: lhs
                });
            },
            ir::Ins::Sub(_vt) => {
                let rhs = ftc.stack().pop();
                let lhs = ftc.stack().peek();

                ins.push(arm64::Ins::SubShifted {
                    size: arm64::SizeFlag::Size64,
                    shift_mode: arm64::ShiftMode::LogicalLeft,
                    shift: 0,
                    dest: lhs,
                    src: rhs,
                    shifted_src: lhs
                });
            },
            ir::Ins::Mul(_vt) => {
                let rhs = ftc.stack().pop();
                let lhs = ftc.stack().peek();

                ins.push(arm64::Ins::MAdd {
                    size: arm64::SizeFlag::Size64,
                    dest: lhs,
                    mul1: lhs,
                    mul2: rhs,
                    addend: arm64::Reg::zero()
                });
            },
            ir::Ins::Div(vt) => {
                let rhs = ftc.stack().pop();
                let lhs = ftc.stack().peek();

                if vt.is_signed() {
                    ins.push(arm64::Ins::SDiv {
                        size: arm64::SizeFlag::Size64,
                        dest: lhs,
                        divided: lhs,
                        divisor: rhs
                    });
                } else {
                    ins.push(arm64::Ins::UDiv {
                        size: arm64::SizeFlag::Size64,
                        dest: lhs,
                        divided: lhs,
                        divisor: rhs
                    });
                }
            },
            ir::Ins::Neg(_vt) => {
                ins.push(arm64::Ins::SubShifted {
                    size: arm64::SizeFlag::Size64,
                    shift_mode: arm64::ShiftMode::LogicalLeft,
                    shift: 0,
                    dest: ftc.stack().peek(),
                    src: ftc.stack().peek(),
                    shifted_src: arm64::Reg::zero()
                });
            },
            ir::Ins::Eq(vt) => {
                cmp!(vt, ftc, ins, Eq);
            },
            ir::Ins::Ne(vt) => {
                cmp!(vt, ftc, ins, Ne);
            },
            ir::Ins::Lt(vt) => {
                cmp!(vt, ftc, ins, Lt);
            },
            ir::Ins::Le(vt) => {
                cmp!(vt, ftc, ins, Le);
            },
            ir::Ins::Gt(vt) => {
                cmp!(vt, ftc, ins, Gt);
            },
            ir::Ins::Ge(vt) => {
                cmp!(vt, ftc, ins, Ge);
            },
            ir::Ins::BoolAnd => {
                let rhs = ftc.stack().pop();
                let lhs = ftc.stack().peek();

                ins.push(arm64::Ins::AndShifted {
                    size: arm64::SizeFlag::Size64,
                    shift_mode: arm64::ShiftMode::LogicalLeft,
                    shift: 0,
                    dest: lhs,
                    src: rhs,
                    shifted_src: lhs
                });
            },
            ir::Ins::BoolOr => {
                let rhs = ftc.stack().pop();
                let lhs = ftc.stack().peek();

                ins.push(arm64::Ins::OrrShifted {
                    size: arm64::SizeFlag::Size64,
                    shift_mode: arm64::ShiftMode::LogicalLeft,
                    shift: 0,
                    dest: lhs,
                    src: rhs,
                    shifted_src: lhs
                });
            },
            ir::Ins::Loop(body, condition, increment) => {
                let start = ftc.new_local_symbol();
                ins.push(arm64::Ins::LocalSymbol(start));

                for inner_ins in condition {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }

                let cond = ftc.stack().pop();
                ins.push(arm64::Ins::SubsShifted {
                    size: arm64::SizeFlag::Size32,
                    shift_mode: arm64::ShiftMode::LogicalLeft,
                    dest: arm64::Reg::zero(),
                    src: cond,
                    shifted_src: arm64::Reg::zero(),
                    shift: 0
                });

                let final_end = ftc.new_local_symbol();
                ins.push(arm64::Ins::ConditionalBranchLocalSymbol(final_end, arm64::Condition::Eq));

                let inc_start = ftc.new_local_symbol();

                ftc.local_symbols().push(LocalSymbol::Loop(
                    inc_start, final_end
                ));

                for inner_ins in body {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }
                
                ftc.local_symbols().pop();

                ins.push(arm64::Ins::LocalSymbol(inc_start));

                for inner_ins in increment {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }

                ins.push(arm64::Ins::BranchLocalSymbol(start));

                ins.push(arm64::Ins::LocalSymbol(final_end));
            },
            ir::Ins::If(then, condition) => {
                for inner_ins in condition {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }

                let cond = ftc.stack().pop();
                ins.push(arm64::Ins::SubsShifted {
                    size: arm64::SizeFlag::Size32,
                    shift_mode: arm64::ShiftMode::LogicalLeft,
                    dest: arm64::Reg::zero(),
                    src: cond,
                    shifted_src: arm64::Reg::zero(),
                    shift: 0
                });
                
                let end = ftc.new_local_symbol();
                ins.push(arm64::Ins::ConditionalBranchLocalSymbol(end, arm64::Condition::Eq));

                ftc.local_symbols().push(LocalSymbol::If);
                for inner_ins in then {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }
                ftc.local_symbols().pop();

                ins.push(arm64::Ins::LocalSymbol(end));
            },
            ir::Ins::IfElse(true_then, false_then, condition) => {
                for inner_ins in condition {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }

                let cond = ftc.stack().pop();
                ins.push(arm64::Ins::SubsShifted {
                    size: arm64::SizeFlag::Size32,
                    shift_mode: arm64::ShiftMode::LogicalLeft,
                    dest: arm64::Reg::zero(),
                    src: cond,
                    shifted_src: arm64::Reg::zero(),
                    shift: 0
                });
                
                let end = ftc.new_local_symbol();
                let false_start = ftc.new_local_symbol();

                // if false, jump to false_start...
                ins.push(arm64::Ins::ConditionalBranchLocalSymbol(false_start, arm64::Condition::Eq));

                // otherwise (if) true, continue...
                ftc.local_symbols().push(LocalSymbol::If);
                for inner_ins in true_then {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }
                ftc.local_symbols().pop();
                // then jump to the end
                ins.push(arm64::Ins::BranchLocalSymbol(end));

                ins.push(arm64::Ins::LocalSymbol(false_start));
                ftc.local_symbols().push(LocalSymbol::If);
                for inner_ins in false_then {
                    self.translate_instruction_to(inner_ins, ftc, ins);
                }
                ftc.local_symbols().pop();
                // just continue to end

                ins.push(arm64::Ins::LocalSymbol(end));
            },
            ir::Ins::Break(_) => todo!(),
            ir::Ins::Continue(_) => todo!(),
            ir::Ins::PushLiteral(_vt, val) => {
                ins.push(arm64::Ins::MovZ {
                    size: arm64::SizeFlag::Size64,
                    dest: ftc.stack().push(),
                    shift: 0,
                    val: *val as u32
                });
            },
            ir::Ins::Drop => {
                ftc.stack().pop();
            },
        }
    }
}