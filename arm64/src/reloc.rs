// Note: This is copied from x86/src/reloc.rs - maybe we should just have a central reloc crate?

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalSymbolID(usize);

impl LocalSymbolID {
    pub fn new(value: usize) -> LocalSymbolID {
        LocalSymbolID(value)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for LocalSymbolID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalSymbolID(usize);

impl GlobalSymbolID {
    pub fn new(value: usize) -> GlobalSymbolID {
        GlobalSymbolID(value)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for GlobalSymbolID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}


pub enum RelocationType {
    LocalFunctionSymbol(LocalSymbolID),
    RelativeGlobalSymbol(GlobalSymbolID),
    AbsoluteGlobalSymbol(GlobalSymbolID)
}

pub struct Relocation {
    kind: RelocationType,
    offset: usize,
    addend: i64
}

impl Relocation {
    pub fn new_local_branch(symbol: LocalSymbolID, offset: usize, addend: i64) -> Relocation {
        Relocation {
            kind: RelocationType::LocalFunctionSymbol(symbol),
            offset, addend
        }
    }

    pub fn new_global_relative(symbol: GlobalSymbolID, offset: usize, addend: i64) -> Relocation {
        Relocation {
            kind: RelocationType::RelativeGlobalSymbol(symbol),
            offset, addend
        }
    }

    pub fn new_global_absolute(global: GlobalSymbolID, offset: usize, addend: i64) -> Relocation {
        Relocation {
            kind: RelocationType::AbsoluteGlobalSymbol(global),
            offset, addend
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(&self.kind, RelocationType::LocalFunctionSymbol(_))
    }

    pub fn kind(&self) -> &RelocationType {
        &self.kind
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn addend(&self) -> i64 {
        self.addend
    }
}