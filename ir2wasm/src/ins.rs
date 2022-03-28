use crate::TranslationContext;

pub(crate) enum Path {
    Local(usize),
    Addr
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

impl<'a> TranslationContext<'a> {
    pub(crate) fn translate_ins(&self, func: &ir::Function, path_stack: &mut PathStack, ins: &ir::Ins, insns: &mut Vec<wasm::Ins>) {
        match ins {
            ir::Ins::PushPath(value_path, _) => {
                let mut path = match value_path.origin() {
                    ir::ValuePathOrigin::Local(local_index, _) => {
                        Path::Local(crate::util::wasm_local_index_from_ir_local_index(*local_index, func))
                    },
                    ir::ValuePathOrigin::Global(global_index, _) => {
                        insns.push(wasm::Ins::ConstI32(self.get_global_addr(*global_index).unwrap()));
                        Path::Addr
                    },
                    ir::ValuePathOrigin::Deref(_) => {
                        Path::Addr
                    },
                };
                
                for component in value_path.components() {
                    match component {
                        ir::ValuePathComponent::Slice(_st) =>
                            match path {
                                Path::Local(local_index) => {
                                    insns.push(wasm::Ins::LocalGet(local_index));
                                    insns.push(wasm::Ins::Add(wasm::NumType::I32));
                                    path = Path::Addr;
                                },
                                Path::Addr => {
                                    insns.push(wasm::Ins::Load(wasm::NumType::I32, wasm::MemArg::new(0, 0)));
                                    insns.push(wasm::Ins::Add(wasm::NumType::I32));
                                }
                            },
                        ir::ValuePathComponent::Property(prop_idx, compound_type, _) =>
                            match path {
                                Path::Local(local_index) => path = Path::Local(local_index + crate::util::value_type_count_for_compound_up_to_prop(compound_type, *prop_idx)),
                                Path::Addr => {
                                    insns.push(wasm::Ins::ConstI32(crate::util::size_for_compound_type_up_to_prop(compound_type.as_ref(), *prop_idx) as i32));
                                    insns.push(wasm::Ins::Add(wasm::NumType::I32));
                                },
                            },
                        ir::ValuePathComponent::Length =>
                            match path {
                                Path::Local(local_index) => path = Path::Local(local_index + 1),
                                Path::Addr => {
                                    insns.push(wasm::Ins::ConstI32(4));
                                    insns.push(wasm::Ins::Add(wasm::NumType::I32));
                                },
                            },
                    }
                }
            
                path_stack.push(path);
            },
            ir::Ins::Pop(vt) => {
                match path_stack.pop() {
                    Path::Local(local_index) => insns.push(wasm::Ins::LocalSet(local_index)),
                    Path::Addr =>
                        match vt {
                            ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool =>
                                insns.push(wasm::Ins::StoreTrunc(wasm::NumType::I32, wasm::NumSize::Bits8, wasm::MemArg::new(0, 0))),
                            ir::ValueType::U16 | ir::ValueType::I16 => 
                                insns.push(wasm::Ins::StoreTrunc(wasm::NumType::I32, wasm::NumSize::Bits16, wasm::MemArg::new(0, 0))),
                            ir::ValueType::U32 | ir::ValueType::I32 | ir::ValueType::U64 | ir::ValueType::I64 | ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) => 
                                insns.push(wasm::Ins::Store(crate::util::value_type_to_num_type(vt), wasm::MemArg::new(0, 0))),
                        }
                }
            },
            ir::Ins::Push(vt) => {
                match path_stack.pop() {
                    Path::Local(local_index) => insns.push(wasm::Ins::LocalGet(local_index)),
                    Path::Addr =>
                        match vt {
                            ir::ValueType::U8 | ir::ValueType::Bool =>
                                insns.push(wasm::Ins::LoadZX(wasm::NumType::I32, wasm::NumSize::Bits8, wasm::MemArg::new(0, 0))),
                            ir::ValueType::I8 =>
                                insns.push(wasm::Ins::LoadSX(wasm::NumType::I32, wasm::NumSize::Bits8, wasm::MemArg::new(0, 0))),
                            ir::ValueType::U16 =>
                                insns.push(wasm::Ins::LoadZX(wasm::NumType::I32, wasm::NumSize::Bits16, wasm::MemArg::new(0, 0))),
                            ir::ValueType::I16 =>
                                insns.push(wasm::Ins::LoadSX(wasm::NumType::I32, wasm::NumSize::Bits16, wasm::MemArg::new(0, 0))),
                            ir::ValueType::U32 | ir::ValueType::I32 | ir::ValueType::U64 | ir::ValueType::I64 | ir::ValueType::UPtr |
                            ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) =>
                                insns.push(wasm::Ins::Load(crate::util::value_type_to_num_type(vt), wasm::MemArg::new(0, 0))),
                        },
                }
            },
            ir::Ins::Index(slice_type) => {
                insns.push(wasm::Ins::ConstI32(crate::util::size_for_storable_type(slice_type) as i32));
                insns.push(wasm::Ins::Mul(wasm::NumType::I32));
            },
            ir::Ins::New(st) => {
                insns.push(wasm::Ins::ConstI32(crate::util::size_for_storable_type(st) as i32));
                insns.push(wasm::Ins::Call(
                    self.function_index(self.unit().find_alloc().expect("Not linked with std")).unwrap()
                ));
            },
            ir::Ins::NewSlice(slice_type) => {
                insns.push(wasm::Ins::ConstI32(crate::util::size_for_storable_type(slice_type) as i32));
                insns.push(wasm::Ins::Call(
                    self.function_index(self.unit().find_alloc_slice().expect("Not linked with std")).unwrap()
                ));
            },
            ir::Ins::Free(st) => {
                insns.push(wasm::Ins::ConstI32(crate::util::size_for_storable_type(st) as i32));
                insns.push(wasm::Ins::Call(
                    self.function_index(self.unit().find_free().expect("Not linked with std")).unwrap()
                ));
            },
            ir::Ins::FreeSlice(slice_type) => {
                insns.push(wasm::Ins::ConstI32(crate::util::size_for_storable_type(slice_type) as i32));
                insns.push(wasm::Ins::Call(
                    self.function_index(self.unit().find_free_slice().expect("Not linked with std")).unwrap()
                ));
            },
            ir::Ins::PushLiteral(vt, i) => {
                insns.push(match vt {
                    ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::I16 | ir::ValueType::U16 | ir::ValueType::I32 | ir::ValueType::U32 =>
                        wasm::Ins::ConstI32(*i as i32),
                    ir::ValueType::U64 | ir::ValueType::I64 => wasm::Ins::ConstI64(*i as i64),
                    ir::ValueType::UPtr | ir::ValueType::IPtr =>  wasm::Ins::ConstI32(*i as i32),
                    ir::ValueType::Bool =>  wasm::Ins::ConstI32(*i as i32),
                    ir::ValueType::Ref(_) | ir::ValueType::Index(_) => panic!(),
                });
            },
            ir::Ins::Add(vt) => insns.push(wasm::Ins::Add(crate::util::value_type_to_num_type(vt))),
            ir::Ins::Mul(vt) => insns.push(wasm::Ins::Mul(crate::util::value_type_to_num_type(vt))),
            ir::Ins::Div(vt) => insns.push(wasm::Ins::Div(crate::util::value_type_to_num_type(vt), vt.is_signed())),
            ir::Ins::Sub(vt) => insns.push(wasm::Ins::Sub(crate::util::value_type_to_num_type(vt))),
            ir::Ins::Neg(vt) => {
                insns.push(match vt {
                    ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::I16 | ir::ValueType::U16 | ir::ValueType::I32 | ir::ValueType::U32 =>
                        wasm::Ins::ConstI32(-1),
                    ir::ValueType::U64 | ir::ValueType::I64 => wasm::Ins::ConstI64(-1),
                    ir::ValueType::UPtr | ir::ValueType::IPtr =>  wasm::Ins::ConstI32(-1),
                    ir::ValueType::Bool =>  wasm::Ins::ConstI32(-1),
                    ir::ValueType::Ref(_) | ir::ValueType::Index(_) => panic!(),
                });
                insns.push(wasm::Ins::Mul(crate::util::value_type_to_num_type(vt)));
            }
            ir::Ins::Lt(vt) => insns.push(wasm::Ins::Lt(crate::util::value_type_to_num_type(vt), vt.is_signed())),
            ir::Ins::Le(vt) => insns.push(wasm::Ins::Le(crate::util::value_type_to_num_type(vt), vt.is_signed())),
            ir::Ins::Gt(vt) => insns.push(wasm::Ins::Gt(crate::util::value_type_to_num_type(vt), vt.is_signed())),
            ir::Ins::Ge(vt) => insns.push(wasm::Ins::Ge(crate::util::value_type_to_num_type(vt), vt.is_signed())),
            ir::Ins::Eq(vt) => insns.push(wasm::Ins::Eq(crate::util::value_type_to_num_type(vt))),
            ir::Ins::Ne(vt) => insns.push(wasm::Ins::Ne(crate::util::value_type_to_num_type(vt))),
            ir::Ins::BoolAnd => insns.push(wasm::Ins::And(wasm::NumType::I32)),
            ir::Ins::BoolOr => insns.push(wasm::Ins::Or(wasm::NumType::I32)),
            ir::Ins::Call(idx) => {
                insns.push(wasm::Ins::Call(self.function_index(*idx).unwrap()));
            },
            ir::Ins::Loop(code, condition, inc) => {
                insns.push(wasm::Ins::Block(wasm::BlockType::Empty, vec![wasm::Ins::Loop(wasm::BlockType::Empty, {
                    let mut inner_insns = Vec::new();
                    for ins in condition { self.translate_ins(func, path_stack, ins, &mut inner_insns); }
                    inner_insns.push(wasm::Ins::Eqz(wasm::NumType::I32));
                    inner_insns.push(wasm::Ins::BrIf(1));

                    // TODO: Embed block to allow for continuing
                    for ins in code { self.translate_ins(func, path_stack, ins, &mut inner_insns); }
                    for ins in inc { self.translate_ins(func, path_stack, ins, &mut inner_insns); }

                    inner_insns.push(wasm::Ins::Br(0));

                    inner_insns
                })]));
            },
            ir::Ins::If(true_then, cond) => {
                insns.push(wasm::Ins::Block(wasm::BlockType::Empty, {
                    let mut inner_insns = Vec::new();

                    for ins in cond { self.translate_ins(func, path_stack, ins, &mut inner_insns); }
                    inner_insns.push(wasm::Ins::Eqz(wasm::NumType::I32));
                    inner_insns.push(wasm::Ins::BrIf(0));

                    for ins in true_then { self.translate_ins(func, path_stack, ins, &mut inner_insns); }

                    inner_insns
                }));
            },
            ir::Ins::IfElse(true_then, false_then, cond) => {
                insns.push(wasm::Ins::Block(wasm::BlockType::Empty, {
                    let mut first_inner_insns = Vec::new();

                    first_inner_insns.push(wasm::Ins::Block(wasm::BlockType::Empty, {
                        let mut inner_insns = Vec::new();

                        for ins in cond { self.translate_ins(func, path_stack, ins, &mut inner_insns); }
                        inner_insns.push(wasm::Ins::Eqz(wasm::NumType::I32));
                        inner_insns.push(wasm::Ins::BrIf(0));
    
                        for ins in true_then { self.translate_ins(func, path_stack, ins, &mut inner_insns); }
    
                        inner_insns.push(wasm::Ins::Br(1));
    
                        inner_insns
                    }));

                    for ins in false_then { self.translate_ins(func, path_stack, ins, &mut first_inner_insns); }

                    first_inner_insns
                }));
            },
            ir::Ins::Convert(from, to) => {
                match (from, to) {
                    (ir::ValueType::U64 | ir::ValueType::I64, ir::ValueType::U64 | ir::ValueType::I64) => {},
                    (ir::ValueType::U64 | ir::ValueType::I64, _) => insns.push(wasm::Ins::WrapI64),
                    (_, ir::ValueType::U64) => insns.push(wasm::Ins::Extend(false)),
                    (_, ir::ValueType::I64) => insns.push(wasm::Ins::Extend(true)),
                    _ => {}
                }
                
            },
            ir::Ins::Ret => {
                insns.push(wasm::Ins::Return);
            },
            _ => todo!("{:?}", ins)
        }
    }
}