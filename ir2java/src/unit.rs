use crate::ins::{InstructionTarget, PathStack, StackMapBuilder};

pub struct TranslationContext<'a> {
    unit: &'a ir::TranslationUnit,
}

impl<'a> TranslationContext<'a> {
    pub(crate) fn signature_as_descriptor(signature: &ir::Signature, class: &java::ClassFile) -> String {
        let mut params = Vec::new();
            for param in signature.params() {
                params.push(crate::util::value_type_to_descriptor(param, class));
            }

            java::Descriptor::function_to_string(
                &params,
                &if signature.return_count() == 0 {
                    java::Descriptor::Void
                } else if signature.return_count() == 1 {
                    crate::util::value_type_to_descriptor(signature.returns().get(0).unwrap(), class)
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
                ir::CompoundContent::Struct(struc) => {
                    for prop in struc.props() {
                        let field = java::Field::new_on(prop.name(), crate::util::storable_type_to_descriptor(prop.prop_type(), &classfile).to_string(), &mut classfile);
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
                classfile.this_index(), outer_class, inner_class_name
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
            let mut stack_map = StackMapBuilder::new(func);

            for (l, local) in func.locals()[func.signature().param_count()..].iter().enumerate() {
                match local.local_type() {
                    ir::StorableType::Compound(_) => {
                        insns.push(java::Ins::AConstNull);
                        insns.push(java::Ins::AStore { local: (l + func.signature().param_count()) as u8 });
                    },
                    ir::StorableType::Value(val) =>
                        match val {
                            ir::ValueType::Ref(_) => {
                                insns.push(java::Ins::AConstNull);
                                insns.push(java::Ins::AStore { local: (l + func.signature().param_count()) as u8 });
                            },
                            _ => {
                                insns.push(java::Ins::IConst0);
                                insns.push(java::Ins::IStore { local: (l + func.signature().param_count()) as u8 });
                            }
                        },
                    ir::StorableType::Slice(_) => {
                        insns.push(java::Ins::AConstNull);
                        insns.push(java::Ins::AStore { local: (l + func.signature().param_count()) as u8 });
                    },
                    ir::StorableType::SliceData(_) => panic!(),
                }

                stack_map.accessed_local(ir::LocalIndex::new(l + func.signature().param_count()));
            }

            let mut path_stack = PathStack::new();
            for ins in func.code() {
                ctx.translate_ins(func, ins, &mut path_stack, &mut insns, &mut stack_map, &mut classfile);
            }

            let method = java::Method::new_on(crate::util::name_for_function(func), TranslationContext::signature_as_descriptor(func.signature(), &classfile), &mut classfile);

            // TODO: Find correct max size
            let mut code = java::Code::new(10, func.local_count() as u16, insns.take());
            code.add_map(stack_map.take());
            method.add_code(code);
            method.set_access(java::MethodAccessFlags::from_bits(
                java::MethodAccessFlags::ACC_PUBLIC | java::MethodAccessFlags::ACC_STATIC
            ));

            if func.is_entry() {
                main_name = Some(crate::util::name_for_function(func));
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
            let name = crate::util::field_name_for_global(global, ir::GlobalIndex::new(idx));
            let desc = crate::util::storable_type_to_descriptor(global.global_type(), &classfile).to_string();
            let field = java::Field::new_on(&name, &desc, &mut classfile);

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