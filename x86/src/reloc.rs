pub type LocalSymbolID = usize;
pub type GlobalSymbolID = usize;

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