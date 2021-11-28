use crate::ins::PathStack;


pub struct TranslationContext<'a> {
    unit: &'a ir::TranslationUnit,
    extern_count: usize,
    globals: Vec<i32>
}

impl<'a> TranslationContext<'a> {
    fn compound_to_memory(&self, ct: &ir::CompoundValue, raw: &mut Vec<u8>) {
        match ct {
            ir::CompoundValue::Struct(s) => {
                for prop in s.props() {
                    self.storable_to_memory(prop.value(), raw);
                }
            },
        }
    }

    fn storable_to_memory(&self, storable: &ir::StorableValue, raw: &mut Vec<u8>) {
        match storable {
            ir::StorableValue::Compound(ct) => self.compound_to_memory(ct, raw),
            ir::StorableValue::Value(v) => match v {
                ir::Value::U8(v) => raw.extend(v.to_le_bytes()),
                ir::Value::I8(v) => raw.extend(v.to_le_bytes()),
                ir::Value::U16(v) => raw.extend(v.to_le_bytes()),
                ir::Value::I16(v) => raw.extend(v.to_le_bytes()),
                ir::Value::U32(v) => raw.extend(v.to_le_bytes()),
                ir::Value::I32(v) => raw.extend(v.to_le_bytes()),
                ir::Value::U64(v) => raw.extend(v.to_le_bytes()),
                ir::Value::I64(v) => raw.extend(v.to_le_bytes()),
                ir::Value::UPtr(v) => raw.extend((*v as u32).to_le_bytes()),
                ir::Value::IPtr(v) => raw.extend((*v as i32).to_le_bytes()),
                ir::Value::Bool(v) => raw.extend((*v as u8).to_le_bytes()),
                ir::Value::Ref(idx) => raw.extend(self.globals.get(idx.idx()).expect("Out of order global dependency").to_le_bytes()),
            },
            ir::StorableValue::Slice(gidx, idx, len) => {
                raw.extend(
                    (
                        self.globals.get(gidx.idx()).expect("Out of order global dependency")
                        + (
                            *idx as i32 * crate::util::size_for_storable_type(match self.unit.get_global(*gidx).unwrap().global_type() {
                                ir::StorableType::Slice(st) => st,
                                ir::StorableType::SliceData(st) => st,
                                _ => panic!()
                            }) as i32
                        )
                    ).to_le_bytes()
                );
                raw.extend((*len as u32).to_le_bytes());
            },
            ir::StorableValue::SliceData(data) => {
                for element in data {
                    self.storable_to_memory(element, raw);
                }
            },
        }
    }

    pub(crate) fn get_global_addr(&self, global: ir::GlobalIndex) -> Option<i32> {
        match self.globals.get(global.idx()) {
            Some(x) => Some(*x),
            None => None
        }
    }

    pub fn translate_unit(unit: &ir::TranslationUnit) -> Result<wasm::Module, String> {
        let mut module = wasm::Module::new();

        let mut extern_count = 0;
        for func in unit.functions() {
            if func.is_extern() { extern_count += 1; }
        }

        let mut ctx = TranslationContext {
            unit, extern_count,
            globals: Vec::new()
        };

        // TODO: This is very order dependent, which may not always work
        let mut raw = Vec::new();
        for global in unit.globals() {
            ctx.globals.push(raw.len() as i32);
            if let Some(default) = global.default() {
                ctx.storable_to_memory(default, &mut raw);
            } else {
                raw.extend(vec![0; crate::util::size_for_storable_type(global.global_type())]);
            }
        }

        // TODO: 8 pages is arbitrary here
        module.add_memory(wasm::MemType::new(wasm::Limits::new(8)));
        module.add_export(wasm::Export::new("mem", wasm::ExportDescriptor::Mem(0)));

        let mem_size = module.add_global(wasm::Global::new(wasm::GlobalType::new(wasm::ValType::Num(wasm::NumType::I32)), wasm::Expr::with(vec![
            wasm::Ins::ConstI32(raw.len() as i32)
        ])));
        module.add_export(wasm::Export::new("mem_size", wasm::ExportDescriptor::Global(mem_size)));

        module.add_data(wasm::Data::Active(0, wasm::Expr::with(vec![
            wasm::Ins::ConstI32(0)
        ]), raw));

        for func in unit.functions() {
            if !func.is_extern() { continue; }

            let wfunc = module.add_type(wasm::FunctionType::new(
                func.signature().params().iter().map(|x| crate::util::value_type_to_val_type(x)).collect(),    
                func.signature().returns().iter().map(|x| crate::util::value_type_to_val_type(x)).collect()
            ));

            module.add_import(wasm::Import::new(
                if let Some(location) = func.location() {
                    location.to_string()
                } else {
                    "std".to_string()
                },
                if let Some(method_of) = func.method_of() {
                    format!("{}.{}", method_of.name(), func.name())
                } else {
                    func.name().to_owned()
                },
                wasm::ImportDescriptor::Type(wfunc)
            ));
        }

        for func in unit.functions() {
            if func.is_extern() { continue; }

            let wfunc = module.add_type(wasm::FunctionType::new(
                func.signature().params().iter().map(|x| crate::util::value_type_to_val_type(x)).collect(),    
                func.signature().returns().iter().map(|x| crate::util::value_type_to_val_type(x)).collect()
            ));
            module.add_function(wfunc);

            let mut code = Vec::new();

            let mut locals = Vec::new();
            for param in &func.locals()[func.signature().param_count()..] {
                for vt in crate::util::value_types_for_storable_type(param.local_type()) {
                    locals.push(crate::util::value_type_to_val_type(&vt));
                }
            }

            let mut path_stack = PathStack::new();

            for ins in func.code() {
                ctx.translate_ins(func, &mut path_stack, ins, &mut code);
            }
    
            module.add_code(wasm::Code::new(locals, wasm::Expr::with(code)));

            module.add_export(wasm::Export::new(if let Some(method_of) = func.method_of() {
                format!("{}.{}", method_of.name(), func.name())
            } else if func.is_entry() {
                "main".to_owned()
            } else {
                func.name().to_owned()
            }, wasm::ExportDescriptor::Func(wfunc)));
        }

        Ok(module)
    }

    pub fn function_index(&self, ir_index: ir::FunctionIndex) -> Option<usize> {
        if self.unit.get_function(ir_index)?.is_extern() {
            let mut idx = 0;
            for func in &self.unit.functions()[0..ir_index.idx()] {
                if func.is_extern() { idx += 1; }
            }

            Some(idx)
        } else {
            let mut idx = self.extern_count;
            for func in &self.unit.functions()[0..ir_index.idx()] {
                if !func.is_extern() { idx += 1; }
            }

            Some(idx)
        }
    }

    pub fn unit(&self) -> &'a ir::TranslationUnit {
        &self.unit
    }
}