use crate::{TranslationContext};

enum Path {
    Local(usize, java::Descriptor),
    Global(usize),
    /// Assumes an index followed by a slice
    Slice(java::Descriptor),
    /// Assumes a reference
    Prop(usize, java::Descriptor),
    /// Assumes a slice
    Length,
    // Reference already on stack
    Ref
}

impl Path {
    fn push(&self, insns: &mut InstructionTarget)  {
        match self {
            Path::Local(idx, desc) => {
                insns.push(java::opt::ins::load(*idx, desc));
            },
            Path::Global(idx) => {
                insns.push(java::Ins::GetStatic { index: *idx })
            },
            Path::Slice(desc) => {
                insns.push(java::opt::ins::aload(desc));
            },
            Path::Prop(idx, _desc) => {
                insns.push(java::Ins::GetField { index: *idx });
            },
            Path::Length => {
                insns.push(java::Ins::ArrayLength);
            },
            Path::Ref => {}
        }
    }

    fn pop(&self, insns: &mut InstructionTarget)  {
        match self {
            Path::Local(idx, desc) => {
                insns.push(java::opt::ins::store(*idx, desc));
            },
            Path::Global(idx) => {
                insns.push(java::Ins::PutStatic { index: *idx })
            },
            Path::Slice(desc) => {
                insns.push(java::opt::ins::astore(desc));
            },
            Path::Prop(idx, _desc) => {
                insns.push(java::Ins::PutField { index: *idx });
            },
            Path::Length => {
                panic!("Attempt to write slice length")
            },
            Path::Ref => panic!("Attempt to write to reference")
        }
    }
}

pub(crate) struct PathStack {
    paths: Vec<Path>
}

impl PathStack {
    pub(crate) fn new() -> PathStack {
        PathStack {
            paths: Vec::new()
        }
    }

    fn push(&mut self, path: Path) {
        self.paths.push(path);
    }

    fn pop(&mut self) -> Path {
        self.paths.pop().expect("Path stack underflow")
    }
}

pub(crate) struct InstructionTarget {
    insns: Vec<java::Ins>,
    size: usize
}

impl InstructionTarget {
    pub(crate) fn new(size: usize) -> InstructionTarget {
        InstructionTarget {
            insns: Vec::new(),
            size
        }
    }

    fn push(&mut self, ins: java::Ins) {
        self.size += ins.size(self.size);
        self.insns.push(ins);
    }

    fn extend(&mut self, target: InstructionTarget, overlap: usize) {
        self.size += target.size - overlap;
        self.insns.extend(target.insns);
    }

    fn tell(&self) -> usize {
        self.size
    }

    pub fn take(self) -> Vec<java::Ins> {
        self.insns
    }
}

pub(crate) struct StackMapBuilder<'a> {
    func: &'a ir::Function,
    map: java::StackMapTable,
    offset: usize,
    first_unused_local: usize,
    previous_first_unused_local: usize
}

impl<'a> StackMapBuilder<'a> {
    pub(crate) fn new(func: &'a ir::Function) -> StackMapBuilder<'a> {
        StackMapBuilder {
            func,
            map: java::StackMapTable::new(),
            offset: 0,
            first_unused_local: func.signature().param_count(),
            previous_first_unused_local: func.signature().param_count()
        }
    }

    fn accessed_local(&mut self, local: ir::LocalIndex) {
        if local.idx() >= self.first_unused_local {
            self.first_unused_local = local.idx() + 1;
        }
    }

    fn empty_stack_target(&mut self, addr: usize, class: &mut java::ClassFile) {
        if self.first_unused_local != self.previous_first_unused_local {
            let mut locals = Vec::new();
            for i in self.previous_first_unused_local..self.first_unused_local {
                locals.push(crate::util::verification_type_for_storable(self.func.locals()[i].local_type(), class));
            }

            self.push_at(addr, java::StackMapFrame::AppendFrame {
                offset: self.delta_to(addr),
                locals
            });

            self.previous_first_unused_local = self.first_unused_local;
        } else {
            self.push_at(addr, java::StackMapFrame::SameFrameExtended {
                offset: self.delta_to(addr),
            });
        }
    }

    fn single_stack_target(&mut self, addr: usize, el: java::VerificationTypeInfo, class: &mut java::ClassFile) {
        if self.first_unused_local != self.previous_first_unused_local {
            let mut locals = Vec::new();
            for i in self.previous_first_unused_local..self.first_unused_local {
                locals.push(crate::util::verification_type_for_storable(self.func.locals()[i].local_type(), class));
            }

            self.push_at(addr, java::StackMapFrame::FullFrame {
                offset: self.delta_to(addr),
                locals,
                stack: vec![el]
            });

            self.previous_first_unused_local = self.first_unused_local;
        } else {
            self.push_at(addr, java::StackMapFrame::SameLocalsOneStackExtended {
                offset: self.delta_to(addr),
                stack: el
            });
        }
    }

    fn push_at(&mut self, offset: usize, frame: java::StackMapFrame) {
        self.map.add_entry(frame);
        self.offset = offset;
    }

    fn delta_to(&self, offset: usize) -> u16 {
        if self.offset == 0 {
            offset as u16
        } else {
            (offset - self.offset - 1) as u16
        }
    }

    pub(crate) fn take(self) -> java::StackMapTable {
        self.map
    }
}

impl<'a> TranslationContext<'a> {
    pub(crate) fn translate_ins(&self, func: &ir::Function, ins: &ir::Ins, path_stack: &mut PathStack, insns: &mut InstructionTarget, stack_map: &mut StackMapBuilder, class: &mut java::ClassFile) {
        match ins {
            ir::Ins::PushPath(value_path, _) => {
                let mut path = match value_path.origin() {
                    ir::ValuePathOrigin::Local(idx, st) => {
                        stack_map.accessed_local(*idx);
                        Path::Local(idx.idx(), crate::util::storable_type_to_descriptor(st, &class))
                    },
                    ir::ValuePathOrigin::Global(idx, st) => {
                        let field_idx = class.const_field(
                            &class.name().to_string(),
                            &crate::util::field_name_for_global(self.unit().get_global(*idx).unwrap(), *idx),
                            &crate::util::storable_type_to_descriptor(st, class).to_string()
                        );
                        Path::Global(field_idx)
                    },
                    ir::ValuePathOrigin::Deref(_) => {
                        Path::Ref
                    },
                };
                
                for component in value_path.components() {
                    path.push(insns);

                    match component {
                        ir::ValuePathComponent::Slice(st) => {
                            insns.push(java::Ins::Swap);
                            path = Path::Slice(crate::util::storable_type_to_descriptor(st, &class));
                        },
                        ir::ValuePathComponent::Property(prop_idx, ctr, _) => {
                            match ctr.content() {
                                ir::CompoundContent::Struct(struc) => {
                                    let prop = struc.prop(*prop_idx).unwrap();
                                    let desc = crate::util::storable_type_to_descriptor(prop.prop_type(), &class);

                                    let field_ref_idx = class.const_field(&crate::util::class_name_for_compound(class, ctr), prop.name(), &desc.to_string());

                                    path = Path::Prop(field_ref_idx, desc);
                                },
                            }
                        },
                        ir::ValuePathComponent::Length => {
                            path = Path::Length;
                        }
                    }
                }
            
                path_stack.push(path);
            },
            ir::Ins::Push(_) => {
                path_stack.pop().push(insns);
            },
            ir::Ins::Pop(_) => {
                path_stack.pop().pop(insns);
            },
            ir::Ins::Index(_) => {
                // Do nothing
            },
            ir::Ins::New(st) => {
                let name = match st {
                    ir::StorableType::Compound(ctr) => {
                        crate::util::class_name_for_compound(class, ctr)
                    },
                    ir::StorableType::Value(_) => panic!("Cannot currently create reference to value"),
                    ir::StorableType::Slice(_) => todo!(),
                    ir::StorableType::SliceData(_) => panic!(),
                };

                insns.push(java::Ins::New { index: class.const_class(&name) });
                insns.push(java::Ins::Dup);
                let method = class.const_method(&name, "<init>", "()V");
                insns.push(java::Ins::InvokeSpecial { index: method });
            },
            ir::Ins::NewSlice(st) => {
                match st {
                    ir::StorableType::Compound(ctr) => {
                        let class_idx = class.const_class(&crate::util::class_name_for_compound(class, ctr));

                        insns.push(java::Ins::ANewArray { index: class_idx });
                    },
                    ir::StorableType::Value(v) => {
                        insns.push(java::Ins::NewArray { atype: crate::util::value_type_to_descriptor(v, &class) });
                    },
                    ir::StorableType::Slice(_) => todo!(),
                    ir::StorableType::SliceData(_) => panic!(),
                }
            },
            ir::Ins::Convert(from, to) => {
                if let Some(ins) = java::opt::ins::conv(&crate::util::value_type_to_descriptor(from, class), &crate::util::value_type_to_descriptor(to, class)) {
                    insns.push(ins);
                }
            },
            ir::Ins::Call(idx) => {
                let call_func = self.unit().get_function(*idx).unwrap();

                let name = match call_func.location() {
                    Some(loc) => loc.to_string(),
                    None => class.name().to_string()
                };
                let method_ref = class.const_method(&name, &crate::util::name_for_function(call_func), &TranslationContext::signature_as_descriptor(call_func.signature(), &class));

                insns.push(java::Ins::InvokeStatic { index: method_ref });
            },
            ir::Ins::Ret => {
                if let Some(ret_type) = func.signature().returns().get(0) {
                    insns.push(java::opt::ins::ret(&crate::util::value_type_to_descriptor(ret_type, class)));
                } else {
                    insns.push(java::Ins::Return);
                }
            },
            ir::Ins::Inc(_, _) => todo!(),
            ir::Ins::Dec(_, _) => todo!(),
            ir::Ins::Add(vt) =>
                insns.push(java::opt::ins::add(&crate::util::value_type_to_descriptor(vt, class))),
            ir::Ins::Mul(_) => todo!(),
            ir::Ins::Div(_) => todo!(),
            ir::Ins::Sub(_) => todo!(),
            ir::Ins::Eq(_) => todo!(),
            ir::Ins::Ne(_) => todo!(),
            ir::Ins::Lt(vt) =>
                match vt {
                    ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::U16 | ir::ValueType::I16 | ir::ValueType::U32 | ir::ValueType::I32 => {
                        insns.push(java::Ins::IfICmpLt { branch: 3 + 1 + 3 });
                        insns.push(java::Ins::IConst0);
                        insns.push(java::Ins::Goto { branch: 3 + 1 });
                        stack_map.empty_stack_target(insns.tell(), class);
                        insns.push(java::Ins::IConst1);
                        stack_map.single_stack_target(insns.tell(), java::VerificationTypeInfo::Integer, class);
                    },
                    ir::ValueType::U64 | ir::ValueType::I64 => todo!(),
                    _ => panic!()
                },
            ir::Ins::Le(_) => todo!(),
            ir::Ins::Gt(vt) =>
                match vt {
                    ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::U16 | ir::ValueType::I16 | ir::ValueType::U32 | ir::ValueType::I32 => {
                        insns.push(java::Ins::IfICmpGt { branch: 3 + 1 + 3 });
                        insns.push(java::Ins::IConst0);
                        insns.push(java::Ins::Goto { branch: 3 + 1 });
                        stack_map.empty_stack_target(insns.tell(), class);
                        insns.push(java::Ins::IConst1);
                        stack_map.single_stack_target(insns.tell(), java::VerificationTypeInfo::Integer, class);
                    },
                    ir::ValueType::U64 | ir::ValueType::I64 => todo!(),
                    _ => panic!()
                },
            ir::Ins::Ge(_) => todo!(),
            ir::Ins::Loop(code, condition, inc) => {
                stack_map.empty_stack_target(insns.tell(), class);
                
                let mut condition_branch = InstructionTarget::new(insns.tell());
                for ins in condition { self.translate_ins(func, ins, path_stack, &mut condition_branch, stack_map, class); }
                let condition_branch_size = condition_branch.tell() - insns.tell();

                // Ifeq jump to end - 3 bytes

                let mut code_branch = InstructionTarget::new(insns.tell() + condition_branch_size + 3);
                for ins in code { self.translate_ins(func, ins, path_stack, &mut code_branch, stack_map, class); }
                let code_branch_size = code_branch.tell() - condition_branch_size - 3 - insns.tell();

                let mut inc_branch = InstructionTarget::new(insns.tell() + condition_branch_size + 3 + code_branch_size);
                for ins in inc { self.translate_ins(func, ins, path_stack, &mut inc_branch, stack_map, class); }
                let inc_branch_size = inc_branch.tell() - condition_branch_size - 3 - code_branch_size - insns.tell();

                let overlap = insns.tell();
                
                insns.extend(condition_branch, overlap);
                insns.push(java::Ins::IfEq { branch: (code_branch_size + inc_branch_size + 3 + 3) as i16 });
                insns.extend(code_branch, overlap + condition_branch_size + 3);
                insns.extend(inc_branch, overlap + condition_branch_size + 3 + code_branch_size);
                insns.push(java::Ins::Goto { branch: -((condition_branch_size + code_branch_size + inc_branch_size + 3) as i16) });

                stack_map.empty_stack_target(insns.tell(), class);
            },
            ir::Ins::If(true_then, condition) => {
                let mut condition_branch = InstructionTarget::new(insns.tell());
                for ins in condition { self.translate_ins(func, ins, path_stack, &mut condition_branch, stack_map, class); }
                let condition_branch_size = condition_branch.tell() - insns.tell();

                // Ifeq jump to end - 3 bytes

                let mut true_branch = InstructionTarget::new(insns.tell() + condition_branch_size + 3);
                for ins in true_then { self.translate_ins(func, ins, path_stack, &mut true_branch, stack_map, class); }
                let true_branch_size = true_branch.tell() - condition_branch_size - 3 - insns.tell();

                let overlap = insns.tell();
                
                insns.extend(condition_branch, overlap);
                insns.push(java::Ins::IfEq { branch: (true_branch_size + 3) as i16 });
                insns.extend(true_branch, overlap + condition_branch_size + 3);
                stack_map.empty_stack_target(insns.tell(), class);
            },
            ir::Ins::IfElse(true_then, false_then, condition) => {
                let mut condition_branch = InstructionTarget::new(insns.tell());
                for ins in condition { self.translate_ins(func, ins, path_stack, &mut condition_branch, stack_map, class); }
                let condition_branch_size = condition_branch.tell() - insns.tell();

                // Ifeq jump to false_then - 3 bytes

                let mut true_branch = InstructionTarget::new(insns.tell() + condition_branch_size + 3);
                for ins in true_then { self.translate_ins(func, ins, path_stack, &mut true_branch, stack_map, class); }
                let true_branch_size = true_branch.tell() - condition_branch_size - 3 - insns.tell();

                let mut false_branch = InstructionTarget::new(insns.tell() + condition_branch_size + 3 + true_branch_size + 3);
                for ins in false_then { self.translate_ins(func, ins, path_stack, &mut false_branch, stack_map, class); }
                let false_branch_size = false_branch.tell() - condition_branch_size - 3 - true_branch_size - 3 - insns.tell();

                let overlap = insns.tell();
                
                insns.extend(condition_branch, overlap);
                insns.push(java::Ins::IfEq { branch: (true_branch_size + 3 + 3) as i16 });
                insns.extend(true_branch, overlap + condition_branch_size + 3);
                insns.push(java::Ins::Goto { branch: (false_branch_size + 3) as i16 });
                stack_map.empty_stack_target(insns.tell(), class);
                insns.extend(false_branch, overlap + condition_branch_size + 6 + true_branch_size);

                stack_map.empty_stack_target(insns.tell(), class);
            },
            ir::Ins::Break(_) => todo!(),
            ir::Ins::Continue(_) => todo!(),
            ir::Ins::PushLiteral(vt, i) => 
                match vt {
                    ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::U16 | ir::ValueType::I16 | ir::ValueType::U32 | ir::ValueType::I32 => insns.push(java::Ins::SIPush { value: *i as i16 }),
                    ir::ValueType::U64 | ir::ValueType::I64 => todo!(),
                    _ => panic!(),
                },
            ir::Ins::Drop => insns.push(java::Ins::Pop),
        }
    }
}