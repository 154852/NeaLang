use std::collections::HashMap;
use crate::{Ins, Relocation, RelocationType};

pub struct EncodeContext {
	raw: Vec<u8>,
	relocations: Vec<Relocation>,
}

impl EncodeContext {
	pub fn new() -> EncodeContext {
		EncodeContext {
			raw: Vec::new(),
			relocations: Vec::new(),
		}
	}

	pub fn append_function(&mut self, code: &Vec<Ins>) -> (usize, usize) {
		let add = 8 - (self.raw.len() % 8);
		if add != 8 {
			// Pad with nops to be 8 byte aligned
			self.raw.extend(&vec![0x90; add]);
		}

		let addr = self.raw.len();

		let mut local_symbols = HashMap::new();
		let mut new_relocations = Vec::new();
		for ins in code {
			ins.encode(&mut self.raw, &mut local_symbols, &mut new_relocations);
		}

		for reloc in new_relocations {
			match &reloc.kind() {
				RelocationType::LocalFunctionSymbol(symbol) => {
					let addr = local_symbols.get(symbol).expect("Local symbol not definied");
					self.raw[reloc.offset()..reloc.offset()+4].copy_from_slice(
						&(((*addr as i64 - reloc.offset() as i64) + reloc.addend()) as u32).to_le_bytes()
					);
				},
				_ => self.relocations.push(reloc)
			}
		}

		(addr, self.raw.len() - addr)
	}

	pub fn len(&self) -> usize {
		self.raw.len()
	}

	pub fn take(self) -> (Vec<u8>, Vec<Relocation>) {
		(self.raw, self.relocations)
	}
}