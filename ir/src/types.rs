use crate::ValueType;

#[derive(Debug)]
pub struct StructProperty {
    name: String,
    prop_type: ValueType
}

#[derive(Debug)]
pub struct StructContent {
    props: Vec<StructProperty>
}

#[derive(Debug)]
pub enum TypeContent {
    Struct()
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
    Value(ValueType)
}

impl StorableType {
    pub fn is_any_value(&self) -> bool {
        match self {
            StorableType::Compound(_) => false,
            StorableType::Value(_) => true,
        }
    }

    pub fn is_value(&self, value: &ValueType) -> bool {
        match self {
            StorableType::Compound(_) => false,
            StorableType::Value(vt) => vt == value,
        }
    }
}