use crate::PathStack;

pub(crate) fn compound_name_to_java_string(name: &str) -> String {
    name.to_string()
}

pub(crate) fn storable_type_to_jtype(st: &ir::StorableType) -> java::Descriptor {
    match st {
        ir::StorableType::Compound(_) => todo!(),
        ir::StorableType::Value(v) => value_type_to_jtype(v),
        ir::StorableType::Slice(st) => java::Descriptor::Array(1, Box::new(storable_type_to_jtype(st))),
        ir::StorableType::SliceData(_) => panic!("Cannot get jtype for slice data"),
    }
}

pub(crate) fn value_type_to_jtype(vt: &ir::ValueType) -> java::Descriptor {
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
                ir::StorableType::Compound(c) => java::Descriptor::Reference(compound_name_to_java_string(c.name())),
                ir::StorableType::Value(_) => todo!(),
                ir::StorableType::Slice(st) => java::Descriptor::Array(1, Box::new(storable_type_to_jtype(st))),
                ir::StorableType::SliceData(_) => panic!("Cannot get jtype for slice data"),
            },
        ir::ValueType::Index(_) => java::Descriptor::Int,
    }
}

pub struct TranslationContext<'a> {
    unit: &'a ir::TranslationUnit,
}

impl<'a> TranslationContext<'a> {
    pub fn signature_as_descriptor(signature: &ir::Signature) -> String {
        let mut params = Vec::new();
            for param in signature.params() {
                params.push(value_type_to_jtype(param));
            }

            java::Descriptor::function_to_string(
                &params,
                &if signature.return_count() == 0 {
                    java::Descriptor::Void
                } else if signature.return_count() == 1 {
                    value_type_to_jtype(signature.returns().get(0).unwrap())
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
                        let field = java::Field::new_on(prop.name(), storable_type_to_jtype(prop.prop_type()).to_string(), &mut classfile);
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

            let mut insns = Vec::new();
            let mut path_stack = PathStack::new();
            for ins in func.code() {
                ctx.translate_ins(func, ins, &mut path_stack, &mut insns, &mut classfile);
            }

            let method = java::Method::new_on(func.name(), TranslationContext::signature_as_descriptor(func.signature()), &mut classfile);

            // TODO: Find correct max size
            method.add_code(java::Code::new(10, func.local_count() as u16, insns));
            method.set_access(java::MethodAccessFlags::from_bits(
                java::MethodAccessFlags::ACC_PUBLIC | java::MethodAccessFlags::ACC_STATIC
            ));

            if func.is_entry() {
                main_name = Some(func.name().clone());
            }
        }

        if let Some(main_name) = main_name {
            let name_index = classfile.consant_pool_index_of_str(&main_name).unwrap();
            let desc_index = classfile.consant_pool_index_of_str("()V").unwrap();
            let name_and_type = classfile.add_constant(java::Constant::NameAndType(java::NameAndType::new(name_index, desc_index)));
            let method_ref = classfile.add_constant(java::Constant::MethodRef(java::MethodRef::new(classfile.this_index(), name_and_type)));

            let method = java::Method::new_on("main", "([Ljava/lang/String;)V", &mut classfile);

            method.add_code(java::Code::new(0, 1, vec![
                java::Ins::InvokeStatic {
                    index: method_ref
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

        Ok(classfile)
    }

    pub fn unit(&self) -> &'a ir::TranslationUnit {
        &self.unit
    }
}