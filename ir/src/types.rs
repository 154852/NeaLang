#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    U8, I8,
    U16, I16,
    U32, I32,
    U64, I64,
    UPtr, IPtr,
    Bool,
    Ref(Box<StorableType>),
    Index(Box<StorableType>)
}

impl ValueType {
    pub fn is_signed(&self) -> bool {
        match &self {
            ValueType::U8 | ValueType::U16 | ValueType::U32 | ValueType::U64 | ValueType::UPtr | ValueType::Bool | ValueType::Ref(_) | ValueType::Index(_) => false,
            ValueType::I8 | ValueType::I16 | ValueType::I32 | ValueType::I64 | ValueType::IPtr => true,
        }
    }

    pub fn is_num(&self) -> bool {
        match &self {
            ValueType::Ref(_) | ValueType::Index(_) => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorableType {
    Compound(CompoundTypeRef),
    Value(ValueType),
    Slice(Box<StorableType>),
    SliceData(Box<StorableType>)
}

#[derive(Debug, Clone, Copy)]
pub struct PropertyIndex(usize);

impl PropertyIndex {
    pub fn new(value: usize) -> PropertyIndex {
        PropertyIndex(value)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for PropertyIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

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
        self.props.get(idx.idx())
    }

    pub fn find_prop(&self, name: &str) -> Option<PropertyIndex> {
        for (p, prop) in self.props.iter().enumerate() {
            if prop.name() == name { return Some(PropertyIndex::new(p)) }
        }
        None
    }

    pub fn props(&self) -> &Vec<StructProperty> {
        &self.props
    }
}

#[derive(Debug)]
pub enum CompoundContent {
    Struct(StructContent)
}

#[derive(Debug)]
pub struct CompoundType {
    name: String,
    content: CompoundContent
}

impl CompoundType {
    pub fn new<T: Into<String>>(name: T, content: CompoundContent) -> CompoundTypeRef {
        std::rc::Rc::new(CompoundType {
            name: name.into(),
            content
        })
    }

    pub fn content(&self) -> &CompoundContent {
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