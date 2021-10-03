use crate::ValueType;

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