use std::collections::HashMap;

pub type LocalSymbolID = usize;
pub type GlobalSymbolID = usize;

pub enum RelocationType {
    LocalFunctionSymbol(LocalSymbolID),
	GlobalFunctionSymbol(GlobalSymbolID)
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

	pub fn new_global_call(symbol: GlobalSymbolID, offset: usize, addend: i64) -> Relocation {
        Relocation {
            kind: RelocationType::GlobalFunctionSymbol(symbol),
            offset, addend
        }
    }

	pub fn is_local(&self) -> bool {
		matches!(&self.kind, RelocationType::LocalFunctionSymbol(_))
	}

    pub fn write_local(&self, data: &mut Vec<u8>, local_symbols: &HashMap<LocalSymbolID, usize>) {
		match &self.kind {
			RelocationType::LocalFunctionSymbol(symbol) => {
				let addr = local_symbols.get(symbol).expect("Local symbol not definied");
				data[self.offset..self.offset+4].copy_from_slice(
					&(((*addr as i64 - self.offset as i64) + self.addend) as u32).to_le_bytes()
				);
			},
			_ => panic!("Relocation is not local")
		}
    }

	pub fn write_global(&self, data: &mut Vec<u8>, global_symbols: &HashMap<LocalSymbolID, usize>) -> bool {
		match &self.kind {
			RelocationType::GlobalFunctionSymbol(symbol) => {
				let addr = match global_symbols.get(symbol) {
					Some(x) => x,
					None => return false
				};
				data[self.offset..self.offset+4].copy_from_slice(
					&(((*addr as i64 - self.offset as i64) + self.addend) as u32).to_le_bytes()
				);

				true
			},
			_ => panic!("Relocation is not global")
		}
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