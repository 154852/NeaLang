use crate::{CompoundTypeRef, Global, GlobalIndex, Ins, StorableType, ValueType};

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

    pub fn compound_types(&self) -> &Vec<CompoundTypeRef> {
        &self.compound_types
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
        GlobalIndex::new(self.globals.len() - 1)
    }

    pub fn find_alloc(&self) -> Option<FunctionIndex> {
        for (f, function) in self.functions.iter().enumerate() {
            if function.is_alloc() { return Some(FunctionIndex::new(f)); }
        }
        None
    }

    pub fn find_alloc_slice(&self) -> Option<FunctionIndex> {
        for (f, function) in self.functions.iter().enumerate() {
            if function.is_alloc_slice() { return Some(FunctionIndex::new(f)); }
        }
        None
    }

    pub fn globals(&self) -> &Vec<Global> {
        &self.globals
    }

    pub fn get_global(&self, idx: GlobalIndex) -> Option<&Global> {
        self.globals.get(idx.idx())
    }

    pub fn add_function(&mut self, function: Function) -> FunctionIndex {
        self.functions.push(function);
        FunctionIndex::new(self.functions.len() - 1)
    }

    pub fn get_function(&self, idx: FunctionIndex) -> Option<&Function> {
        self.functions.get(idx.idx())
    }

    pub fn functions(&self) -> &Vec<Function> { &self.functions }
    pub fn function_count(&self) -> usize { self.functions.len() }

    pub fn get_function_mut(&mut self, idx: FunctionIndex) -> Option<&mut Function> {
        self.functions.get_mut(idx.idx())
    }

    pub fn find_function_index(&self, name: &str) -> Option<FunctionIndex> {
        for (c, ct) in self.functions.iter().enumerate() {
            if ct.method_of().is_none() && ct.name() == name {
                return Some(FunctionIndex::new(c));
            }
        }

        None
    }

    pub fn find_method_index(&self, ctr: CompoundTypeRef, name: &str) -> Option<FunctionIndex> {
        for (c, func) in self.functions.iter().enumerate() {
            if let Some(ct) = func.method_of() {
                if ct == ctr && func.name() == name {
                    return Some(FunctionIndex::new(c));
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

#[derive(Debug, Clone, Copy)]
pub struct LocalIndex(usize);

impl LocalIndex {
    pub fn new(value: usize) -> LocalIndex {
        LocalIndex(value)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for LocalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct FunctionIndex(usize);

impl FunctionIndex {
    pub fn new(value: usize) -> FunctionIndex {
        FunctionIndex(value)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for FunctionIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug)]
pub struct Signature {
    /// Params are pushed in order, so that the first param is evaluated first, so are popped in reverse order
    params: Vec<ValueType>,
    /// Returns are pushed in order, so that the first return is evaluated first, so are popped in reverse order
    returns: Vec<ValueType>
}

impl Signature {
    pub fn new(params: Vec<ValueType>, returns: Vec<ValueType>) -> Signature {
        Signature { params, returns }
    }

    pub fn params(&self) -> &Vec<ValueType> { &self.params }
    pub fn param_count(&self) -> usize { self.params.len() }

    pub fn returns(&self) -> &Vec<ValueType> { &self.returns }
    pub fn return_count(&self) -> usize { self.returns.len() }
}

#[derive(Debug)]
pub enum FunctionAttr {
    /// Marks function as the entry point. May lead to it having it's name changed.
    Entry,

    /// Marks function as being the implementation for new
    Alloc,

    /// Marks function as being the implementation for new slice
    AllocSlice,

    /// Specifies the location for an extern function - used for example to specify module in wasm and class in java
    ExternLocation(String)
}

#[derive(Debug)]
pub struct Function {
    name: String,
    locals: Vec<Local>,
    signature: Signature,
    code: Option<Vec<Ins>>,
    method_of: Option<CompoundTypeRef>,
    
    attrs: Vec<FunctionAttr>
}

impl Function {
    pub fn new<T: Into<String>>(name: T, signature: Signature) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: Some(Vec::new()),
            method_of: None,
            attrs: Vec::new(),
        }
    }

    pub fn new_method<T: Into<String>>(name: T, signature: Signature, ctr: CompoundTypeRef) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: Some(Vec::new()),
            method_of: Some(ctr),
            attrs: Vec::new(),
        }
    }

    pub fn new_extern<T: Into<String>>(name: T, signature: Signature) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: None,
            method_of: None,
            attrs: Vec::new(),
        }
    }

    pub fn new_extern_method<T: Into<String>>(name: T, signature: Signature, ctr: CompoundTypeRef) -> Function {
        Function {
            name: name.into(),
            locals: Vec::new(),
            signature,
            code: None,
            method_of: Some(ctr),
            attrs: Vec::new(),
        }
    }

    pub fn is_extern(&self) -> bool {
        self.code.is_none()
    }

    pub fn set_extern(&mut self) { self.code = None; }
    pub fn set_non_extern(&mut self) { self.code = Some(Vec::new()); }

    pub fn attrs(&self) -> &Vec<FunctionAttr> {
        &self.attrs
    }

    pub fn attr_count(&self) -> usize {
        self.attrs.len()
    }

    pub fn is_entry(&self) -> bool {
        for attr in &self.attrs {
            if matches!(attr, FunctionAttr::Entry) {
                return true;
            }
        }
        false
    }

    pub fn is_alloc(&self) -> bool {
        for attr in &self.attrs {
            if matches!(attr, FunctionAttr::Alloc) {
                return true;
            }
        }
        false
    }

    pub fn is_alloc_slice(&self) -> bool {
        for attr in &self.attrs {
            if matches!(attr, FunctionAttr::AllocSlice) {
                return true;
            }
        }
        false
    }
    
    pub fn location(&self) -> Option<&str> {
        for attr in &self.attrs {
            if let FunctionAttr::ExternLocation(name) = attr {
                return Some(name);
            }
        }
        None
    }

    pub fn push_attr(&mut self, attr: FunctionAttr) {
        self.attrs.push(attr);
    }

    pub fn method_of(&self) -> Option<CompoundTypeRef> {
        self.method_of.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn push_local(&mut self, local: Local) -> LocalIndex {
        self.locals.push(local);
        LocalIndex::new(self.locals.len() - 1)
    }

    pub fn get_local(&self, idx: LocalIndex) -> Option<&Local> {
        self.locals.get(idx.idx())
    }

    pub fn push(&mut self, code: Ins) {
        self.code.as_mut().expect("Cannot push to extern function").push(code);
    }

    pub fn locals(&self) -> &Vec<Local> { &self.locals }
    pub fn local_count(&self) -> usize { self.locals.len() }

    pub fn last_local_index(&self) -> LocalIndex { LocalIndex::new(self.locals.len() - 1) }

    /// Will panic if function is extern, so should only be used when certain it is not
    pub fn code(&self) -> &Vec<Ins> {
        self.code.as_ref().expect("Attempt to get code from extern function")
    }

    pub fn code_opt(&self) -> Option<&Vec<Ins>> {
        self.code.as_ref()
    }

    /// Will panic if function is extern, so should only be used when certain it is not
    pub fn code_mut(&mut self) -> &mut Vec<Ins> {
        self.code.as_mut().expect("Attempt to get code from extern function")
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

pub type BlockMoveDepth = usize;