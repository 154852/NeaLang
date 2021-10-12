use crate::{CompoundTypeRef, Ins, Storable, StorableType};

#[derive(Debug)]
pub struct TranslationUnit {
    functions: Vec<Function>,
    compound_types: Vec<CompoundTypeRef>,
    globals: Vec<Global>
}

impl TranslationUnit {
    pub fn new() -> TranslationUnit {
        TranslationUnit {
            functions: Vec::new(),
            compound_types: Vec::new(),
            globals: Vec::new()
        }
    }

    pub fn add_type(&mut self, compound_type: CompoundTypeRef) {
        self.compound_types.push(compound_type);
    }

    pub fn find_type(&self, name: &str) -> Option<CompoundTypeRef> {
        for ct in self.compound_types.iter() {
            if ct.name() == name {
                return Some(ct.clone());
            }
        }

        None
    }

    pub fn add_global(&mut self, global: Global) -> GlobalIndex {
        self.globals.push(global);
        self.globals.len() - 1
    }

    pub fn globals(&self) -> &Vec<Global> {
        &self.globals
    }

    pub fn get_global(&self, idx: GlobalIndex) -> &Global {
        &self.globals[idx]
    }

    pub fn add_function(&mut self, function: Function) -> FunctionIndex {
        self.functions.push(function);
        self.functions.len() - 1
    }

    pub fn get_function(&self, idx: FunctionIndex) -> &Function {
        &self.functions[idx]
    }

	pub fn functions(&self) -> &Vec<Function> {
		&self.functions
	}

    pub fn get_function_mut(&mut self, idx: FunctionIndex) -> &mut Function {
        &mut self.functions[idx]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    U8, I8,
    U16, I16,
    U32, I32,
    U64, I64,
    UPtr, IPtr,
    Bool,
    Ref(Box<StorableType>),
}

impl ValueType {
    pub fn signed(&self) -> bool {
        match &self {
            ValueType::U8 | ValueType::U16 | ValueType::U32 | ValueType::U64 | ValueType::UPtr | ValueType::Bool | ValueType::Ref(_) => false,
            ValueType::I8 | ValueType::I16 | ValueType::I32 | ValueType::I64 | ValueType::IPtr => true,
        }
    }

    pub fn is_num(&self) -> bool {
        match &self {
            ValueType::Ref(_) => false,
            _ => true,
        }
    }
}

pub type LocalIndex = usize;

#[derive(Debug)]
pub struct Local {
    local_type: StorableType
}

impl Local {
    pub fn new(local_type: StorableType) -> Local {
        Local { local_type }
    }

    pub fn local_type(&self) -> &StorableType {
        &self.local_type
    }
}

pub type GlobalIndex = usize;

#[derive(Debug)]
pub struct Global {
    name: Option<String>,
    global_type: StorableType,
    writable: bool,
    default: Option<Storable>
}

impl Global {
    pub fn new<T: Into<String>>(name: Option<T>, global_type: StorableType, writable: bool) -> Global {
        Global {
            name: name.map(|x| x.into()), global_type, writable,
            default: None
        }
    }

    pub fn new_default<T: Into<String>>(name: Option<T>, global_type: StorableType, writable: bool, default: Storable) -> Global {
        Global {
            name: name.map(|x| x.into()), global_type, writable,
            default: Some(default)
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|x| x.as_str())
    }

    pub fn default(&self) -> Option<&Storable> {
        self.default.as_ref()
    }

    pub fn global_type(&self) -> &StorableType {
        &self.global_type
    }
}

pub type FunctionIndex = usize;

#[derive(Debug)]
pub struct Signature {
    // Params are pushed in order, so that the first param is evaluated first, so are popped in reverse order
    params: Vec<ValueType>,
    // Returns are pushed in order, so that the first return is evaluated first, so are popped in reverse order
    returns: Vec<ValueType>
}

impl Signature {
    pub fn new(params: Vec<ValueType>, returns: Vec<ValueType>) -> Signature {
        Signature { params, returns }
    }

    pub fn params(&self) -> &Vec<ValueType> {
        &self.params
    }

    pub fn returns(&self) -> &Vec<ValueType> {
        &self.returns
    }
}

#[derive(Debug)]
pub struct Function {
    name: String,
    locals: Vec<Local>,
    signature: Signature,
    code: Option<Vec<Ins>>
}

impl Function {
    pub fn new<T: Into<String>>(name: T, signature: Signature) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: Some(Vec::new())
        }
    }

    pub fn new_extern<T: Into<String>>(name: T, signature: Signature) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: None
        }
    }

    pub fn is_extern(&self) -> bool {
        self.code.is_none()
    }

	pub fn name(&self) -> &str {
		&self.name
	}

    pub fn push_local(&mut self, local: Local) -> LocalIndex {
        self.locals.push(local);
        self.locals.len() - 1
    }

    pub fn get_local(&self, idx: LocalIndex) -> Option<&Local> {
        self.locals.get(idx)
    }

    pub fn push(&mut self, code: Ins) {
        self.code.as_mut().expect("Cannot push to extern function").push(code);
    }

    pub fn locals(&self) -> &Vec<Local> {
        &self.locals
    }

    pub fn code(&self) -> &Vec<Ins> {
        self.code.as_ref().expect("Attempt to get code from extern function")
    }

    pub fn code_mut(&mut self) -> &mut Vec<Ins> {
        self.code.as_mut().expect("Attempt to get code from extern function")
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

pub type BlockMoveDepth = usize;