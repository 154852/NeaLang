use java::ClassFile;

use crate::{InstructionTarget, PathStack, StackMapBuilder};

pub(crate) fn storable_type_to_jtype(st: &ir::StorableType, class: &ClassFile) -> java::Descriptor {
    match st {
        ir::StorableType::Compound(ctr) => java::Descriptor::Reference(name_for_compound(class, ctr)),
        ir::StorableType::Value(v) => value_type_to_jtype(v, class),
        ir::StorableType::Slice(st) => java::Descriptor::Array(1, Box::new(storable_type_to_jtype(st, class))),
        ir::StorableType::SliceData(st) => java::Descriptor::Array(1, Box::new(storable_type_to_jtype(st, class))),
    }
}

pub(crate) fn value_type_to_jtype(vt: &ir::ValueType, class: &ClassFile) -> java::Descriptor {
    match vt {
        ir::ValueType::U8 => java::Descriptor::Byte,
        ir::ValueType::I8 => java::Descriptor::Byte,
        ir::ValueType::U16 => java::Descriptor::Short,
        ir::ValueType::I16 => java::Descriptor::Short,
        ir::ValueType::U32 => java::Descriptor::Int,
        ir::ValueType::I32 => java::Descriptor::Int,
        ir::ValueType::U64 => java::Descriptor::Long,
        ir::ValueType::I64 => java::Descriptor::Long,
        ir::ValueType::UPtr => java::Descriptor::Int,
        ir::ValueType::IPtr => java::Descriptor::Int,
        ir::ValueType::Bool => java::Descriptor::Boolean,
        ir::ValueType::Ref(vt) =>
            match vt.as_ref() {
                ir::StorableType::Compound(compound) => java::Descriptor::Reference(name_for_compound(class, compound)),
                ir::StorableType::Value(_) => todo!(),
                ir::StorableType::Slice(st) => java::Descriptor::Array(1, Box::new(storable_type_to_jtype(st, class))),
                ir::StorableType::SliceData(_) => panic!("Cannot get jtype for slice data"),
            },
        ir::ValueType::Index(_) => java::Descriptor::Int,
    }
}

pub fn name_for_global(global: &ir::Global, index: ir::GlobalIndex) -> String {
    match global.name() {
        Some(x) => x.to_string(),
        None => format!("_global${}", index)
    }
}

pub fn name_for_compound(class: &ClassFile, compound: &ir::CompoundType) -> String {
    format!("{}${}", class.name(), compound.name())
}

pub fn name_for_function(func: &ir::Function) -> String {
    if let Some(method_of) = func.method_of() {
        format!("{}${}", method_of.name(), func.name())
    } else {
        func.name().to_string()
    }
}

pub struct TranslationContext<'a> {
    unit: &'a ir::TranslationUnit,
}

impl<'a> TranslationContext<'a> {
    pub fn signature_as_descriptor(signature: &ir::Signature, class: &ClassFile) -> String {
        let mut params = Vec::new();
            for param in signature.params() {
                params.push(value_type_to_jtype(param, class));
            }

            java::Descriptor::function_to_string(
                &params,
                &if signature.return_count() == 0 {
                    java::Descriptor::Void
                } else if signature.return_count() == 1 {
                    value_type_to_jtype(signature.returns().get(0).unwrap(), class)
                } else {
                    todo!("Multiple returns")
                }
            )
    }

    pub fn translate_unit_types(unit: &ir::TranslationUnit, name: &str) -> Result<Vec<(String, java::ClassFile)>, String> {
        let mut classes = Vec::new();

        for compound_type in unit.compound_types() {
            let mut classfile = java::ClassFile::new(&format!("{}${}", name, compound_type.name()));

            match compound_type.content() {
                ir::TypeContent::Struct(struc) => {
                    for prop in struc.props() {
                        let field = java::Field::new_on(prop.name(), storable_type_to_jtype(prop.prop_type(), &classfile).to_string(), &mut classfile);
                        field.set_access(java::FieldAccessFlags::from_bits(java::FieldAccessFlags::ACC_PUBLIC));
                    }
                },
            }
            
            let super_init = classfile.const_method("java/lang/Object", "<init>", "()V");

            let init = java::Method::new_on("<init>", "()V", &mut classfile);
            init.set_access(java::MethodAccessFlags::from_bits(java::MethodAccessFlags::ACC_PUBLIC));
            init.add_code(java::Code::new(1, 1, vec![
                java::Ins::ALoad0,
                java::Ins::InvokeSpecial { index: super_init },
                java::Ins::Return,
            ]));

            let outer_class = classfile.const_class(name);
            let inner_class_name = classfile.const_str(compound_type.name());
            classfile.add_inner_class(java::InnerClass::new(
                classfile.this_index(),outer_class, inner_class_name
            ));

            classes.push((format!("{}${}", name, compound_type.name()), classfile));
        }

        Ok(classes)
    }

    pub fn translate_unit(unit: &ir::TranslationUnit, name: &str) -> Result<java::ClassFile, String> {
        let mut classfile = java::ClassFile::new(name);

        let ctx = TranslationContext {
            unit
        };

        let mut main_name = None;

        for func in unit.functions() {
            if func.is_extern() { continue; }

            let mut insns = InstructionTarget::new(0);
            let mut path_stack = PathStack::new();
            let mut stack_map = StackMapBuilder::new(func);
            for ins in func.code() {
                ctx.translate_ins(func, ins, &mut path_stack, &mut insns, &mut stack_map, &mut classfile);
            }

            let method = java::Method::new_on(name_for_function(func), TranslationContext::signature_as_descriptor(func.signature(), &classfile), &mut classfile);

            // TODO: Find correct max size
            let mut code = java::Code::new(10, func.local_count() as u16, insns.take());
            code.add_map(stack_map.take());
            method.add_code(code);
            method.set_access(java::MethodAccessFlags::from_bits(
                java::MethodAccessFlags::ACC_PUBLIC | java::MethodAccessFlags::ACC_STATIC
            ));

            if func.is_entry() {
                main_name = Some(name_for_function(func));
            }
        }

        if let Some(main_name) = main_name {
            let target = classfile.const_method(&classfile.name().to_string(), &main_name, "()V");
            
            let method = java::Method::new_on("main", "([Ljava/lang/String;)V", &mut classfile);
            method.add_code(java::Code::new(0, 1, vec![
                java::Ins::InvokeStatic {
                    index: target
                },
                java::Ins::Return
            ]));
            method.set_access(java::MethodAccessFlags::from_bits(java::MethodAccessFlags::ACC_PUBLIC | java::MethodAccessFlags::ACC_STATIC));
        }

        for compound_type in unit.compound_types() {
            let inner_class = classfile.const_class(&format!("{}${}", name, compound_type.name()));
            let inner_class_name = classfile.const_str(compound_type.name());
            classfile.add_inner_class(java::InnerClass::new(
                inner_class, classfile.this_index(), inner_class_name
            ));
        }

        let mut clinit = Vec::new();

        for (idx, global) in unit.globals().iter().enumerate() {
            let name = name_for_global(global, idx);
            let desc = storable_type_to_jtype(global.global_type(), &classfile).to_string();
            let field = java::Field::new_on(
                &name,
                &desc,
                &mut classfile
            );

            field.set_access(java::FieldAccessFlags::from_bits(java::FieldAccessFlags::ACC_PRIVATE | java::FieldAccessFlags::ACC_STATIC));

            if let Some(default) = global.default() {
                ctx.translate_storable(default, global.global_type(), &mut classfile, &mut clinit);
                clinit.push(java::Ins::PutStatic { index: classfile.const_field(&classfile.name().to_string(), &name, &desc) })
            }
        }

        if clinit.len() != 0 {
            clinit.push(java::Ins::Return);
            
            let method = java::Method::new_on("<clinit>", "()V", &mut classfile);
            method.add_code(java::Code::new(10, 0, clinit));
            method.set_access(java::MethodAccessFlags::from_bits(java::MethodAccessFlags::ACC_STATIC));
        }

        Ok(classfile)
    }

    pub fn unit(&self) -> &'a ir::TranslationUnit {
        &self.unit
    }
}