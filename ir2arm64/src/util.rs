pub(crate) fn offset_of_compound_property(ct: &ir::CompoundTypeRef, idx: ir::PropertyIndex) -> usize {
    match ct.content() {
        ir::CompoundContent::Struct(struc) => {
            let mut offset = 0;

            for prop in &struc.props()[0..idx.idx()] {
                offset += size_for_storable_type(prop.prop_type());
            }

            offset
        },
    }
}

pub(crate) fn size_for_compound_type(ct: &ir::CompoundType) -> usize {
    match ct.content() {
        ir::CompoundContent::Struct(s) => {
            let mut size = 0;

            for s in s.props() {
                size += size_for_storable_type(s.prop_type());
            }

            size
        },
    }
}

pub(crate) fn size_for_value_type(vt: &ir::ValueType) -> usize {
    match vt {
        ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool => 1,
        ir::ValueType::U16 | ir::ValueType::I16 => 2,
        ir::ValueType::U32 | ir::ValueType::I32 => 4,
        ir::ValueType::U64 | ir::ValueType::I64 => 8,
        ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) => 8,
    }
}

pub(crate) fn size_for_storable_type(storable: &ir::StorableType) -> usize {
    match storable {
        ir::StorableType::Compound(ct) => size_for_compound_type(ct),
        ir::StorableType::Value(vt) => size_for_value_type(vt),
        ir::StorableType::Slice(_) => 16,
        ir::StorableType::SliceData(_) => panic!("Cannot compute raw size of SliceData type"),
    }
}