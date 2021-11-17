use ir::{CompoundTypeRef, FunctionIndex};
use wasm;

use crate::PathStack;

pub(crate) fn value_type_to_num_type(vt: &ir::ValueType) -> wasm::NumType {
    match vt {
        ir::ValueType::U8 => wasm::NumType::I32,
        ir::ValueType::I8 => wasm::NumType::I32,
        ir::ValueType::U16 => wasm::NumType::I32,
        ir::ValueType::I16 => wasm::NumType::I32,
        ir::ValueType::U32 => wasm::NumType::I32,
        ir::ValueType::I32 => wasm::NumType::I32,
        ir::ValueType::U64 => wasm::NumType::I64,
        ir::ValueType::I64 => wasm::NumType::I64,
        ir::ValueType::UPtr => wasm::NumType::I32,
        ir::ValueType::IPtr => wasm::NumType::I32,
        ir::ValueType::Bool => wasm::NumType::I32,
        ir::ValueType::Ref(_) | ir::ValueType::Index(_) => wasm::NumType::I32,
    }
}

pub(crate) fn value_type_to_val_type(vt: &ir::ValueType) -> wasm::ValType {
    wasm::ValType::Num(value_type_to_num_type(vt))
}

pub(crate) fn size_for_compound(ct: &ir::CompoundType) -> usize {
	match ct.content() {
		ir::TypeContent::Struct(s) => {
			let mut size = 0;

			for s in s.props() {
				size += size_for_st(s.prop_type());
			}

			size
		},
	}
}

pub(crate) fn size_for_compound_up_to_prop(ct: &ir::CompoundType, prop: usize) -> usize {
	match ct.content() {
		ir::TypeContent::Struct(s) => {
			let mut size = 0;

			for s in &s.props()[0..prop] {
				size += size_for_st(s.prop_type());
			}

			size
		},
	}
}

pub(crate) fn size_for_vt(vt: &ir::ValueType) -> usize {
	match vt {
		ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool => 1,
		ir::ValueType::U16 | ir::ValueType::I16 => 2,
		ir::ValueType::U32 | ir::ValueType::I32 => 4,
		ir::ValueType::U64 | ir::ValueType::I64 => 8,
		ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) => 4,
	}
}

pub(crate) fn size_for_st(st: &ir::StorableType) -> usize {
	match st {
		ir::StorableType::Compound(ct) => size_for_compound(ct),
		ir::StorableType::Value(vt) => size_for_vt(vt),
		ir::StorableType::Slice(_) => 8,
		ir::StorableType::SliceData(_) => panic!("Cannot compute raw size of SliceData type"),
	}
}

pub(crate) fn vts_for_compound(ct: &CompoundTypeRef) -> Vec<ir::ValueType> {
    match ct.content() {
        ir::TypeContent::Struct(s) => {
            let mut values = Vec::new();
            for prop in s.props() {
                values.extend(vts_for_st(prop.prop_type()));
            }
            values
        },
    }
}

pub(crate) fn vts_for_st(st: &ir::StorableType) -> Vec<ir::ValueType> {
	match st {
		ir::StorableType::Compound(ct) => vts_for_compound(ct),
		ir::StorableType::Value(vt) => vec![vt.clone()],
		ir::StorableType::Slice(_) => vec![ir::ValueType::UPtr, ir::ValueType::UPtr],
		ir::StorableType::SliceData(_) => panic!("Cannot store SliceData type as values"),
	}
}

pub(crate) fn vts_count_for_compound(ct: &CompoundTypeRef) -> usize {
    match ct.content() {
        ir::TypeContent::Struct(s) => {
            let mut count = 0;
            for prop in s.props() {
                count += vts_count_for_st(prop.prop_type());
            }
            count
        },
    }
}

pub(crate) fn vts_count_for_compound_up_to_prop(ct: &CompoundTypeRef, prop: usize) -> usize {
    match ct.content() {
        ir::TypeContent::Struct(s) => {
            let mut count = 0;
            for prop in &s.props()[0..prop] {
                count += vts_count_for_st(prop.prop_type());
            }
            count
        },
    }
}

pub(crate) fn vts_count_for_st(st: &ir::StorableType) -> usize {
	match st {
		ir::StorableType::Compound(ct) => vts_count_for_compound(ct),
		ir::StorableType::Value(_) => 1,
		ir::StorableType::Slice(_) => 2,
		ir::StorableType::SliceData(_) => panic!("Cannot store SliceData type as values"),
	}
}


pub struct TranslationContext<'a> {
    unit: &'a ir::TranslationUnit,
    extern_count: usize,
    pub(crate) globals: Vec<i32>
}

impl<'a> TranslationContext<'a> {
    fn compound_to_memory(&self, ct: &ir::Compound, raw: &mut Vec<u8>) {
        match ct {
            ir::Compound::Struct(s) => {
                for prop in s.props() {
                    self.storable_to_memory(prop.value(), raw);
                }
            },
        }
    }

    fn storable_to_memory(&self, storable: &ir::Storable, raw: &mut Vec<u8>) {
        match storable {
            ir::Storable::Compound(ct) => self.compound_to_memory(ct, raw),
            ir::Storable::Value(v) => match v {
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
                ir::Value::Ref(idx) => raw.extend(self.globals.get(*idx).expect("Out of order global dependency").to_le_bytes()),
            },
            ir::Storable::Slice(gidx, idx, len) => {
                raw.extend(
                    (self.globals.get(*gidx).expect("Out of order global dependency") + *idx as i32).to_le_bytes()
                );
                raw.extend((*len as u32).to_le_bytes());
            },
            ir::Storable::SliceData(data) => {
                for el in data {
                    self.storable_to_memory(el, raw);
                }
            },
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

        let mut raw = Vec::new();
        for global in unit.globals() {
            ctx.globals.push(raw.len() as i32);
            if let Some(default) = global.default() {
                ctx.storable_to_memory(default, &mut raw);
            } else {
                raw.extend(vec![0; size_for_st(global.global_type())]);
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
                func.signature().params().iter().map(|x| value_type_to_val_type(x)).collect(),    
                func.signature().returns().iter().map(|x| value_type_to_val_type(x)).collect()
            ));

            module.add_import(wasm::Import::new("std", if let Some(method_of) = func.method_of() {
                format!("{}.{}", method_of.name(), func.name())
            } else {
                func.name().to_owned()
            }, wasm::ImportDescriptor::Type(wfunc)));
        }

        for func in unit.functions() {
            if func.is_extern() { continue; }

            let wfunc = module.add_type(wasm::FunctionType::new(
                func.signature().params().iter().map(|x| value_type_to_val_type(x)).collect(),    
                func.signature().returns().iter().map(|x| value_type_to_val_type(x)).collect()
            ));
            module.add_function(wfunc);

            let mut code = Vec::new();

            let mut locals = Vec::new();
            for param in &func.locals()[func.signature().params().len()..] {
                for vt in vts_for_st(param.local_type()) {
                    locals.push(value_type_to_val_type(&vt));
                }
            }

            let mut path_stack = PathStack::new();

            for ins in func.code() {
                ctx.translate_ins(func, &mut path_stack, ins, &mut code);
            }
    
            module.add_code(wasm::Code::new(locals, wasm::Expr::with(code)));

            module.add_export(wasm::Export::new(if let Some(method_of) = func.method_of() {
                format!("{}.{}", method_of.name(), func.name())
            } else {
                func.name().to_owned()
            }, wasm::ExportDescriptor::Func(wfunc)));
        }

        Ok(module)
    }

    pub fn function_index(&self, ir_index: FunctionIndex) -> usize {
        if self.unit.get_function(ir_index).is_extern() {
            let mut idx = 0;
            for func in &self.unit.functions()[0..ir_index] {
                if func.is_extern() { idx += 1; }
            }

            idx
        } else {
            let mut idx = self.extern_count;
            for func in &self.unit.functions()[0..ir_index] {
                if !func.is_extern() { idx += 1; }
            }

            idx
        }
    }

    pub fn unit(&self) -> &'a ir::TranslationUnit {
        &self.unit
    }
}