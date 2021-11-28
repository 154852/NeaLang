pub(crate) fn storable_type_to_descriptor(st: &ir::StorableType, class: &java::ClassFile) -> java::Descriptor {
    match st {
        ir::StorableType::Compound(ctr) => java::Descriptor::Reference(class_name_for_compound(class, ctr)),
        ir::StorableType::Value(v) => value_type_to_descriptor(v, class),
        ir::StorableType::Slice(st) => java::Descriptor::Array(1, Box::new(storable_type_to_descriptor(st, class))),
        ir::StorableType::SliceData(st) => java::Descriptor::Array(1, Box::new(storable_type_to_descriptor(st, class))),
    }
}

pub(crate) fn value_type_to_descriptor(vt: &ir::ValueType, class: &java::ClassFile) -> java::Descriptor {
    match vt {
        ir::ValueType::U8 | ir::ValueType::I8 => java::Descriptor::Byte,
        ir::ValueType::U16 | ir::ValueType::I16 => java::Descriptor::Short,
        ir::ValueType::U32 | ir::ValueType::I32 => java::Descriptor::Int,
        ir::ValueType::U64 | ir::ValueType::I64 => java::Descriptor::Long,
        ir::ValueType::UPtr | ir::ValueType::IPtr => java::Descriptor::Int,
        ir::ValueType::Bool => java::Descriptor::Boolean,
        ir::ValueType::Ref(ref_target) =>
            match ref_target.as_ref() {
                ir::StorableType::Compound(compound) => java::Descriptor::Reference(class_name_for_compound(class, compound)),
                ir::StorableType::Value(_) => todo!(),
                ir::StorableType::Slice(st) => java::Descriptor::Array(1, Box::new(storable_type_to_descriptor(st, class))),
                ir::StorableType::SliceData(_) => panic!("Cannot get jtype for slice data"),
            },
        ir::ValueType::Index(_) => java::Descriptor::Int,
    }
}

pub(crate) fn verification_type_for_storable(storable: &ir::StorableType, class: &mut java::ClassFile) -> java::VerificationTypeInfo {
    match storable {
        ir::StorableType::Compound(ctr) => {
            java::VerificationTypeInfo::Object(class.const_class(&class_name_for_compound(class, ctr)))
        },
        ir::StorableType::Value(vt) => match vt {
            ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool |
            ir::ValueType::U16 | ir::ValueType::I16 |
            ir::ValueType::U32 | ir::ValueType::I32 | ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Index(_) => java::VerificationTypeInfo::Integer,
            ir::ValueType::U64 | ir::ValueType::I64 => java::VerificationTypeInfo::Long,
            ir::ValueType::Ref(c) => verification_type_for_storable(c, class)
        },
        ir::StorableType::Slice(st) => java::VerificationTypeInfo::Object(class.const_class(&format!("[{}", storable_type_to_descriptor(st, class).to_string()))),
        ir::StorableType::SliceData(_) => todo!(),
    }
}

pub(crate) fn field_name_for_global(global: &ir::Global, index: ir::GlobalIndex) -> String {
    match global.name() {
        Some(x) => x.to_string(),
        None => format!("_global${}", index)
    }
}

pub(crate) fn class_name_for_compound(class: &java::ClassFile, compound: &ir::CompoundType) -> String {
    format!("{}${}", class.name(), compound.name())
}

pub(crate) fn name_for_function(func: &ir::Function) -> String {
    if let Some(method_of) = func.method_of() {
        format!("{}${}", method_of.name(), func.name())
    } else {
        func.name().to_string()
    }
}