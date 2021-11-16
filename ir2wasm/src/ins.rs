use crate::{TranslationContext, size_for_compound_up_to_prop, size_for_st, value_type_to_num_type, vts_count_for_compound_up_to_prop, vts_count_for_st};

pub enum Path {
    Local(usize),
    Addr
}

pub struct PathStack {
    paths: Vec<Path>
}

impl PathStack {
    pub fn new() -> PathStack {
        PathStack {
            paths: Vec::new()
        }
    }

    pub fn push(&mut self, path: Path) {
        self.paths.push(path);
    }

    pub fn pop(&mut self) -> Path {
        self.paths.pop().expect("Path stack underflow")
    }
}

fn wasm_local_index_from_ir_local_index(local: usize, func: &ir::Function) -> usize {
    let mut index = 0;
    for local in &func.locals()[0..local] {
        index += vts_count_for_st(local.local_type());
    }
    index
}

impl<'a> TranslationContext<'a> {
    pub(crate) fn translate_ins(&self, func: &ir::Function, path_stack: &mut PathStack, ins: &ir::Ins, insns: &mut Vec<wasm::Ins>) {
        match ins {
            ir::Ins::PushPath(value_path, _vt) => {
                let mut path = match value_path.origin() {
                    ir::ValuePathOrigin::Local(idx, _) => {
                        Path::Local(wasm_local_index_from_ir_local_index(*idx, func))
                    },
                    ir::ValuePathOrigin::Global(idx, _) => {
                        insns.push(wasm::Ins::ConstI32(*self.globals.get(*idx).expect("Global does not exist")));
                        Path::Addr
                    },
                    ir::ValuePathOrigin::Deref(_) => {
                        Path::Addr
                    },
                };
                
                for component in value_path.components() {
                    match component {
                        ir::ValuePathComponent::Slice(_st) => match path {
                            Path::Local(idx) => {
                                insns.push(wasm::Ins::LocalGet(idx));
                                insns.push(wasm::Ins::Add(wasm::NumType::I32));
                                path = Path::Addr;
                            },
                            Path::Addr => {
                                insns.push(wasm::Ins::Load(wasm::NumType::I32, wasm::MemArg::new(0, 0)));
                                insns.push(wasm::Ins::Add(wasm::NumType::I32));
                            }
                        },
                        ir::ValuePathComponent::Property(prop_idx, ctr, _) => match path {
                            Path::Local(idx) => path = Path::Local(idx + vts_count_for_compound_up_to_prop(ctr, *prop_idx)),
                            Path::Addr => {
                                insns.push(wasm::Ins::ConstI32(size_for_compound_up_to_prop(ctr.as_ref(), *prop_idx) as i32));
                                insns.push(wasm::Ins::Add(wasm::NumType::I32));
                            },
                        },
                        ir::ValuePathComponent::Length => match path {
                            Path::Local(idx) => path = Path::Local(idx + 1),
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
                    Path::Local(idx) => insns.push(wasm::Ins::LocalSet(idx)),
                    Path::Addr => match vt {
                        ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool =>
                            insns.push(wasm::Ins::StoreTrunc(wasm::NumType::I32, wasm::NumSize::Bits8, wasm::MemArg::new(0, 0))),
                        ir::ValueType::U16 | ir::ValueType::I16 => 
                            insns.push(wasm::Ins::StoreTrunc(wasm::NumType::I32, wasm::NumSize::Bits16, wasm::MemArg::new(0, 0))),
                        ir::ValueType::U32 | ir::ValueType::I32 | ir::ValueType::U64 | ir::ValueType::I64 | ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) => 
                            insns.push(wasm::Ins::Store(value_type_to_num_type(vt), wasm::MemArg::new(0, 0))),
                    }
                }
            },
            ir::Ins::Push(vt) => {
                match path_stack.pop() {
                    Path::Local(idx) => insns.push(wasm::Ins::LocalGet(idx)),
                    Path::Addr => match vt {
                        ir::ValueType::U8 | ir::ValueType::Bool =>
                            insns.push(wasm::Ins::LoadZX(wasm::NumType::I32, wasm::NumSize::Bits8, wasm::MemArg::new(0, 0))),
                        ir::ValueType::I8 =>
                            insns.push(wasm::Ins::LoadSX(wasm::NumType::I32, wasm::NumSize::Bits8, wasm::MemArg::new(0, 0))),
                        ir::ValueType::U16 =>
                            insns.push(wasm::Ins::LoadZX(wasm::NumType::I32, wasm::NumSize::Bits16, wasm::MemArg::new(0, 0))),
                        ir::ValueType::I16 =>
                            insns.push(wasm::Ins::LoadSX(wasm::NumType::I32, wasm::NumSize::Bits16, wasm::MemArg::new(0, 0))),
                        ir::ValueType::U32 | ir::ValueType::I32 | ir::ValueType::U64 | ir::ValueType::I64 | ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) =>
                            insns.push(wasm::Ins::Load(value_type_to_num_type(vt), wasm::MemArg::new(0, 0))),
                    },
                }
            },
            ir::Ins::Index(st) => {
                insns.push(wasm::Ins::ConstI32(size_for_st(st) as i32));
                insns.push(wasm::Ins::Mul(wasm::NumType::I32));
            },
            ir::Ins::New(st) => {
                insns.push(wasm::Ins::ConstI32(size_for_st(st) as i32));
                insns.push(wasm::Ins::Call(
                    self.function_index(self.unit().find_function_index("nl_new_object").expect("Not linked with std"))
                ));
            },
            ir::Ins::NewSlice(st) => {
                insns.push(wasm::Ins::ConstI32(size_for_st(st) as i32));
                insns.push(wasm::Ins::Call(
                    self.function_index(self.unit().find_function_index("nl_new_slice").expect("Not linked with std"))
                ));
            },
            ir::Ins::PushLiteral(vt, i) => {
                insns.push(match vt {
                    ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::I16 | ir::ValueType::U16 | ir::ValueType::I32 | ir::ValueType::U32 => wasm::Ins::ConstI32(*i as i32),
                    ir::ValueType::U64 | ir::ValueType::I64 => wasm::Ins::ConstI64(*i as i64),
                    ir::ValueType::UPtr | ir::ValueType::IPtr =>  wasm::Ins::ConstI32(*i as i32),
                    ir::ValueType::Bool =>  wasm::Ins::ConstI32(*i as i32),
                    ir::ValueType::Ref(_) | ir::ValueType::Index(_) => panic!(),
                });
            },
            ir::Ins::Add(vt) => {
                insns.push(wasm::Ins::Add(value_type_to_num_type(vt)));
            },
            ir::Ins::Lt(vt) => {
                insns.push(wasm::Ins::Lt(value_type_to_num_type(vt), vt.signed()));
            },
            ir::Ins::Call(idx) => {
                insns.push(wasm::Ins::Call(self.function_index(*idx)));
            },
            ir::Ins::Loop(code, condition, inc) => {
                insns.push(wasm::Ins::Block(wasm::BlockType::Empty, vec![wasm::Ins::Loop(wasm::BlockType::Empty, {
                    let mut inner_ins = Vec::new();
                    for ins in condition { self.translate_ins(func, path_stack, ins, &mut inner_ins); }
                    inner_ins.push(wasm::Ins::Eqz(wasm::NumType::I32));
                    inner_ins.push(wasm::Ins::BrIf(1));

                    for ins in code { self.translate_ins(func, path_stack, ins, &mut inner_ins); }
                    for ins in inc { self.translate_ins(func, path_stack, ins, &mut inner_ins); }

                    inner_ins.push(wasm::Ins::Br(0));

                    inner_ins
                })]))
            },
            ir::Ins::Convert(from, to) => {
                if value_type_to_num_type(from) == value_type_to_num_type(to) {
                    // Do nothing
                } else {
                    todo!()
                }
            },
            ir::Ins::Ret => {
                insns.push(wasm::Ins::Return);
            },
            _ => todo!("{:?}", ins)
        }
    }
}