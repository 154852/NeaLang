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

    pub fn get_global(&self, idx: GlobalIndex) -> Option<&Global> {
        self.globals.get(idx)
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

    pub fn find_function_index(&self, name: &str) -> Option<FunctionIndex> {
        for (c, ct) in self.functions.iter().enumerate() {
            if ct.method_of().is_none() && ct.name() == name {
                return Some(c);
            }
        }

        None
    }

    pub fn find_method_index(&self, ctr: CompoundTypeRef, name: &str) -> Option<FunctionIndex> {
        for (c, func) in self.functions.iter().enumerate() {
            if let Some(ct) = func.method_of() {
                if ct == ctr && func.name() == name {
                    return Some(c);
                }
            }
        }

        None
    }

    pub fn find_function(&self, name: &str) -> Option<&Function> {
        for ct in self.functions.iter() {
            if ct.method_of().is_none() && ct.name() == name {
                return Some(ct);
            }
        }

        None
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
    Index(Box<StorableType>)
}

impl ValueType {
    pub fn signed(&self) -> bool {
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
    code: Option<Vec<Ins>>,
    method_of: Option<CompoundTypeRef>
}

impl Function {
    pub fn new<T: Into<String>>(name: T, signature: Signature) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: Some(Vec::new()),
            method_of: None
        }
    }

    pub fn new_method<T: Into<String>>(name: T, signature: Signature, ctr: CompoundTypeRef) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: Some(Vec::new()),
            method_of: Some(ctr)
        }
    }

    pub fn new_extern<T: Into<String>>(name: T, signature: Signature) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: None,
            method_of: None
        }
    }

    pub fn new_extern_method<T: Into<String>>(name: T, signature: Signature, ctr: CompoundTypeRef) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: None,
            method_of: Some(ctr)
        }
    }

    pub fn is_extern(&self) -> bool {
        self.code.is_none()
    }

    pub fn set_extern(&mut self) {
        self.code = None;
    }

    pub fn method_of(&self) -> Option<CompoundTypeRef> {
        self.method_of.clone()
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

    pub fn code_opt(&self) -> Option<&Vec<Ins>> {
        self.code.as_ref()
    }

    pub fn code_mut(&mut self) -> &mut Vec<Ins> {
        self.code.as_mut().expect("Attempt to get code from extern function")
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

pub type BlockMoveDepth = usize;