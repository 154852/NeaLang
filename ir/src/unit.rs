use crate::Ins;

#[derive(Debug)]
pub struct TranslationUnit {
    functions: Vec<Function>,
    globals: Vec<Global>
}

impl TranslationUnit {
    pub fn new() -> TranslationUnit {
        TranslationUnit {
            functions: Vec::new(),
            globals: Vec::new()
        }
    }

    pub fn add_function(&mut self, function: Function) -> FunctionIndex {
        self.functions.push(function);
        self.functions.len() - 1
    }

    pub fn get_function(&self, idx: FunctionIndex) -> &Function {
        &self.functions[idx]
    }

    pub fn add_global(&mut self, global: Global) {
        self.globals.push(global);
    }

    pub fn functions(&self) -> &Vec<Function> {
        &self.functions
    }

    pub fn globals(&self) -> &Vec<Global> {
        &self.globals
    }
}

pub type GlobalIndex = usize;

#[derive(Debug)]
pub struct Global {
    name: Option<String>,
    data: GlobalData
}

#[derive(Debug)]
pub enum GlobalData {
    RawString(RawStringGlobal),
    Zero(usize),
}

#[derive(Debug)]
pub struct RawStringGlobal {
    data: Vec<u8>,
    size: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    U8, I8,
    U16, I16,
    U32, I32,
    U64, I64,
    UPtr, IPtr,
}

impl ValueType {
    pub fn signed(&self) -> bool {
        match &self {
            ValueType::U8 | ValueType::U16 | ValueType::U32 | ValueType::U64 | ValueType::UPtr => false,
            ValueType::I8 | ValueType::I16 | ValueType::I32 | ValueType::I64 | ValueType::IPtr => true,
        }
    }

    pub fn bytes_size(&self, ptr_bytes_size: u64) -> u64 {
        match self {
            ValueType::U8 | ValueType::I8 => 1,
            ValueType::U16 | ValueType::I16 => 2,
            ValueType::U32 | ValueType::I32 => 4,
            ValueType::U64 | ValueType::I64 => 8,
            ValueType::UPtr | ValueType::IPtr => ptr_bytes_size,
        }
    }
}

pub type LocalIndex = usize;

#[derive(Debug)]
pub struct Local {
    value_type: ValueType
}

impl Local {
    pub fn new(value_type: ValueType) -> Local {
        Local { value_type }
    }

    pub fn value_type(&self) -> ValueType {
        self.value_type
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
    code: Vec<Ins>
}

impl Function {
    pub fn new<T: Into<String>>(name: T, signature: Signature) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: Vec::new()
        }
    }

    pub fn push_local(&mut self, local: Local) -> LocalIndex {
        self.locals.push(local);
        self.locals.len() - 1
    }

    pub fn push(&mut self, code: Ins) {
        self.code.push(code);
    }

    pub fn locals(&self) -> &Vec<Local> {
        &self.locals
    }

    pub fn code(&self) -> &Vec<Ins> {
        &self.code
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

pub type BlockMoveDepth = usize;