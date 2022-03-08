use crate::StorableType;

#[derive(Debug, Clone, Copy)]
pub struct GlobalIndex(usize);

impl GlobalIndex {
    pub fn new(value: usize) -> GlobalIndex {
        GlobalIndex(value)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for GlobalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug)]
pub struct Global {
    name: Option<String>,
    global_type: StorableType,
    default: Option<StorableValue>
}

impl Global {
    pub fn new<T: Into<String>>(name: Option<T>, global_type: StorableType) -> Global {
        Global {
            name: match name {
                Some(x) => Some(x.into()),
                None => None
            },
            global_type,
            default: None
        }
    }

    pub fn new_default<T: Into<String>>(name: Option<T>, global_type: StorableType, default: StorableValue) -> Global {
        Global {
            name: match name {
                Some(x) => Some(x.into()),
                None => None
            },
            global_type,
            default: Some(default)
        }
    }

    pub fn name(&self) -> Option<&str> {
        match &self.name {
            Some(x) => Some(x.as_str()),
            None => None
        }
    }

    pub fn default(&self) -> Option<&StorableValue> {
        self.default.as_ref()
    }

    pub fn global_type(&self) -> &StorableType {
        &self.global_type
    }
}

#[derive(Debug)]
pub struct StructPropertyValue {
    value: StorableValue
}

impl StructPropertyValue {
    pub fn new(value: StorableValue) -> StructPropertyValue {
        StructPropertyValue{
            value
        }
    }

    pub fn value(&self) -> &StorableValue {
        &self.value
    }
}

#[derive(Debug)]
pub struct StructValue {
    props: Vec<StructPropertyValue>
}

impl StructValue {
    pub fn new(props: Vec<StructPropertyValue>) -> StructValue {
        StructValue {
            props
        }
    }

    pub fn props(&self) -> &Vec<StructPropertyValue> {
        &self.props
    }
}

#[derive(Debug)]
pub enum CompoundValue {
    Struct(StructValue)
}

#[derive(Debug)]
pub enum Value {
    U8(u8), I8(i8),
    U16(u16), I16(i16),
    U32(u32), I32(i32),
    U64(u64), I64(i64),
    UPtr(usize), IPtr(isize),
    Bool(bool),
    Ref(GlobalIndex),
}

#[derive(Debug)]
pub enum StorableValue {
    Compound(CompoundValue),
    Value(Value),
    /// OwnedSlice, index, length
    Slice(GlobalIndex, usize, usize),
    SliceData(Vec<StorableValue>)
}