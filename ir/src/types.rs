use crate::{GlobalIndex, ValueType};

#[derive(Debug)]
pub struct StructProperty {
    name: String,
    prop_type: StorableType
}

impl StructProperty {
    pub fn new<T: Into<String>>(name: T, prop_type: StorableType) -> StructProperty {
        StructProperty {
            name: name.into(),
            prop_type
        }
    }

    pub fn prop_type(&self) -> &StorableType {
        &self.prop_type
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct StructContent {
    props: Vec<StructProperty>
}

impl StructContent {
    pub fn new() -> StructContent {
        StructContent {
            props: Vec::new()
        }
    }

    pub fn push_prop(&mut self, prop: StructProperty) {
        self.props.push(prop);
    }

    pub fn prop(&self, idx: PropertyIndex) -> Option<&StructProperty> {
        self.props.get(idx)
    }

    pub fn props(&self) -> &Vec<StructProperty> {
        &self.props
    }
}

pub type PropertyIndex = usize;

#[derive(Debug)]
pub enum TypeContent {
    Struct(StructContent)
}

#[derive(Debug)]
pub struct CompoundType {
    name: String,
    content: TypeContent
}

impl CompoundType {
    pub fn new<T: Into<String>>(name: T, content: TypeContent) -> CompoundTypeRef {
        std::rc::Rc::new(CompoundType {
            name: name.into(),
            content
        })
    }

    pub fn content(&self) -> &TypeContent {
        &self.content
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for CompoundType {
    fn eq(&self, other: &Self) -> bool {
        self as *const CompoundType == other as *const CompoundType
    }
}

pub type CompoundTypeRef = std::rc::Rc<CompoundType>;

#[derive(Debug, Clone, PartialEq)]
pub enum StorableType {
    Compound(CompoundTypeRef),
    Value(ValueType),
    Slice(Box<StorableType>)
}

impl StorableType {
    pub fn is_any_value(&self) -> bool {
        match self {
            StorableType::Value(_) => true,
            _ => false,
        }
    }

    pub fn is_value(&self, value: &ValueType) -> bool {
        match self {
            StorableType::Value(vt) => vt == value,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct StructProp {
    value: Storable
}

impl StructProp {
    pub fn new(value: Storable) -> StructProp {
        StructProp{
            value
        }
    }

    pub fn value(&self) -> &Storable {
        &self.value
    }
}

#[derive(Debug)]
pub struct Struct {
    props: Vec<StructProp>
}

impl Struct {
    pub fn new(props: Vec<StructProp>) -> Struct {
        Struct {
            props
        }
    }

    pub fn props(&self) -> &Vec<StructProp> {
        &self.props
    }
}

#[derive(Debug)]
pub enum Compound {
    Struct(Struct)
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
pub struct OwnedSlice {
    elements: Vec<Storable>
}

impl OwnedSlice {
    pub fn new(elements: Vec<Storable>) -> OwnedSlice {
        OwnedSlice {
            elements
        }
    }

    pub fn elements(&self) -> &Vec<Storable> {
        &self.elements
    }
}

#[derive(Debug)]
pub enum Slice {
    OwnedSlice(OwnedSlice)
}

#[derive(Debug)]
pub enum Storable {
    Compound(Compound),
    Value(Value),
    Slice(Slice)
}