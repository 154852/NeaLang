use crate::{Ins, StorableType};

#[derive(Debug)]
pub struct TranslationUnit {
    functions: Vec<Function>,
}

impl TranslationUnit {
    pub fn new() -> TranslationUnit {
        TranslationUnit {
            functions: Vec::new(),
        }
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