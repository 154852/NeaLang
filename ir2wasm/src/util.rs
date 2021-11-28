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

pub(crate) fn size_for_compound_type_up_to_prop(ct: &ir::CompoundType, prop_idx: ir::PropertyIndex) -> usize {
    match ct.content() {
        ir::CompoundContent::Struct(s) => {
            let mut size = 0;

            for s in &s.props()[0..prop_idx.idx()] {
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
        ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) => 4,
    }
}

pub(crate) fn size_for_storable_type(st: &ir::StorableType) -> usize {
    match st {
        ir::StorableType::Compound(ct) => size_for_compound_type(ct),
        ir::StorableType::Value(vt) => size_for_value_type(vt),
        ir::StorableType::Slice(_) => 8,
        ir::StorableType::SliceData(_) => panic!("Cannot compute raw size of SliceData type"),
    }
}

pub(crate) fn value_types_for_compound_type(ct: &ir::CompoundTypeRef) -> Vec<ir::ValueType> {
    match ct.content() {
        ir::CompoundContent::Struct(struc) => {
            let mut values = Vec::new();
            for prop in struc.props() {
                values.extend(value_types_for_storable_type(prop.prop_type()));
            }
            values
        },
    }
}

pub(crate) fn value_types_for_storable_type(st: &ir::StorableType) -> Vec<ir::ValueType> {
    match st {
        ir::StorableType::Compound(ct) => value_types_for_compound_type(ct),
        ir::StorableType::Value(vt) => vec![vt.clone()],
        ir::StorableType::Slice(_) => vec![ir::ValueType::UPtr, ir::ValueType::UPtr],
        ir::StorableType::SliceData(_) => panic!("Cannot store SliceData type as values"),
    }
}

pub(crate) fn value_type_count_for_compound(ct: &ir::CompoundTypeRef) -> usize {
    match ct.content() {
        ir::CompoundContent::Struct(struc) => {
            let mut count = 0;
            for prop in struc.props() {
                count += value_type_count_for_storable_type(prop.prop_type());
            }
            count
        },
    }
}

pub(crate) fn value_type_count_for_compound_up_to_prop(ct: &ir::CompoundTypeRef, prop: ir::PropertyIndex) -> usize {
    match ct.content() {
        ir::CompoundContent::Struct(struc) => {
            let mut count = 0;
            for prop in &struc.props()[0..prop.idx()] {
                count += value_type_count_for_storable_type(prop.prop_type());
            }
            count
        },
    }
}

pub(crate) fn value_type_count_for_storable_type(st: &ir::StorableType) -> usize {
    match st {
        ir::StorableType::Compound(ct) => value_type_count_for_compound(ct),
        ir::StorableType::Value(_) => 1,
        ir::StorableType::Slice(_) => 2,
        ir::StorableType::SliceData(_) => panic!("Cannot store SliceData type as values"),
    }
}

pub(crate) fn wasm_local_index_from_ir_local_index(local: ir::LocalIndex, func: &ir::Function) -> usize {
    let mut index = 0;
    for local in &func.locals()[0..local.idx()] {
        index += crate::util::value_type_count_for_storable_type(local.local_type());
    }
    index
}