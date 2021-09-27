use std::collections::HashMap;
use crate::{Ins, GlobalSymbolID, Relocation};

pub struct EncodeContext {
	raw: Vec<u8>,
	relocations: Vec<Relocation>,
	global_symbols: HashMap<GlobalSymbolID, usize>
}

impl EncodeContext {
	pub fn new() -> EncodeContext {
		EncodeContext {
			raw: Vec::new(),
			relocations: Vec::new(),
			global_symbols: HashMap::new()
		}
	}

	pub fn append_function(&mut self, global_id: GlobalSymbolID, code: &Vec<Ins>) -> usize {
		let add = 8 - (self.raw.len() % 8);
		if add != 8 {
			// Pad with nops to be 8 byte aligned
			self.raw.extend(&vec![0x90; add]);
		}

		let addr = self.raw.len();
		self.global_symbols.insert(global_id, addr);

		let mut local_symbols = HashMap::new();
		let mut new_relocations = Vec::new();
		for ins in code {
			ins.encode(&mut self.raw, &mut local_symbols, &mut new_relocations);
		}

		for reloc in new_relocations {
			if reloc.is_local() {
				reloc.write_local(&mut self.raw, &local_symbols);
			} else {
				self.relocations.push(reloc);
			}
		}

		addr as usize
	}

	pub fn finish(mut self) -> Vec<u8> {
		for reloc in self.relocations {
			reloc.write_global(&mut self.raw, &self.global_symbols);
		}

		self.raw
	}
}