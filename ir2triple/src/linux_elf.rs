use x86;
use ir2x86;
use std::{collections::HashMap, io::Write};
use ofile::{elf, elfbuilder};

const TEXT_BASE: u64 = 0x401000;

struct GlobalIDAllocator<'a> {
	unit: &'a ir::TranslationUnit,
	symbol_ids: HashMap<x86::GlobalSymbolID, (usize, i64)>
}

impl<'a> GlobalIDAllocator<'a> {
	fn global_id_of_function(&self, func: usize) -> x86::GlobalSymbolID {
		func
	}

	fn global_id_of_global(&self, global: usize) -> x86::GlobalSymbolID {
		self.unit.functions().len() + global
	}

	fn push_global_symbol_mapping(&mut self, global: x86::GlobalSymbolID, symbol: usize, addend: i64) {
		self.symbol_ids.insert(global, (symbol, addend));
	}

	fn symbol_for_global_id(&self, global: x86::GlobalSymbolID) -> Option<(usize, i64)> {
		self.symbol_ids.get(&global).map(|x| *x)
	}
}

pub fn encode(unit: &ir::TranslationUnit, path: &str, relocatable: bool) -> Result<(), String> {
	let mut elf = if relocatable {
		elfbuilder::StaticELF::new_relocatable()
	} else {
		elfbuilder::StaticELF::new()
	};

	let mut x86_encoding = x86::EncodeContext::new();
	let ctx = ir2x86::TranslationContext::new(x86::Mode::X8664);

	let text_base = if relocatable { 0 } else { TEXT_BASE };

	let mut gid_allocator = GlobalIDAllocator {
		unit,
		symbol_ids: HashMap::new()
	};

	for (i, func) in unit.functions().iter().enumerate() {
		let gid = gid_allocator.global_id_of_function(i);
		
		if func.is_extern() {
			if !relocatable {
				return Err(format!("Cannot import function '{}' with a statically compiled binary", func.name()));
			}
			gid_allocator.push_global_symbol_mapping(gid, elf.push_symbol(elfbuilder::Symbol::Relocatable(func.name().to_string())), 0);
		} else {
			let mut ins = ctx.translate_function(&func, unit);
			x86::opt::pass_zero(&mut ins);
			
			let (addr, length) = x86_encoding.append_function(&ins);
			gid_allocator.push_global_symbol_mapping(gid, elf.push_symbol(elfbuilder::Symbol::Function(func.name().to_string(), text_base + addr as u64, length as u64)), 0);
		}
	}

	let data_base = if relocatable { 0 } else { ofile::align_up(text_base + x86_encoding.len() as u64, 1 << 12) };

	let data_base_symbol = elf.push_symbol(elfbuilder::Symbol::Section(2)); // data is section 2, TODO: This should not be hard coded

	let mut relocs = Vec::new();
	let mut data = Vec::new();
	for (i, global) in unit.globals().iter().enumerate() {
		let gid = gid_allocator.global_id_of_global(i);

		if let Some(name) = global.name() {
			let pushed = ctx.translate_global(global, unit, &mut relocs, gid, data.len(), 0);
			gid_allocator.push_global_symbol_mapping(gid, elf.push_symbol(elfbuilder::Symbol::Object(
				name.to_string(),
				data_base + data.len() as u64,
				pushed.len() as u64
			)), 0);
			data.extend(pushed);
		} else {
			let pushed = ctx.translate_global(global, unit, &mut relocs, gid, data.len(), 0);
			gid_allocator.push_global_symbol_mapping(gid, data_base_symbol, data.len() as i64);
			data.extend(pushed);
		}
	}

	// Data relocations
	if !relocatable {
		todo!()
	} else {
		for reloc in relocs {
			match reloc.kind() {
				x86::RelocationType::AbsoluteGlobalSymbol(id) => {
					let (symbol, addend) = gid_allocator.symbol_for_global_id(*id).expect("Invalid relocation symbol");

					elf.push_data_relocation(elf::Rela::new(
						reloc.offset() as u64,
						symbol as u64,
						elf::RelocationType::X86646464,
						reloc.addend() + addend
					));
				},
				_ => panic!("Cannot relocate non-global symbol in .data")
			}
		}
	}

	let (text, relocs) = x86_encoding.take();

	// Test relocations
	if !relocatable {
		todo!()
	} else {
		for reloc in relocs {
			match reloc.kind() {
				x86::RelocationType::RelativeGlobalSymbol(id) => {
					let (symbol, addend) = gid_allocator.symbol_for_global_id(*id).expect("Invalid relocation symbol");
	
					elf.push_text_relocation(elf::Rela::new(
						reloc.offset() as u64,
						symbol as u64,
						elf::RelocationType::X8664Plt32,
						reloc.addend() + addend
					));
				},
				_ => panic!("Cannot relocate non-global symbol in .text")
			}
		}
	}

	elf.set_text(text_base, text);
	elf.set_data(data_base, data);
	// elf.set_rodata(0x403000, rodata);

	let header = elf::Header::new_with_entry(
		elf::ABI::SysV,
		if relocatable { elf::ObjectFileType::Relocatable } else { elf::ObjectFileType::Executable },
		elf::Machine::X8664,
		text_base // TODO: Use function entry
	);
	let (header, body) = elf.encode::<ofile::LittleEndian64>(header);

	let mut file = std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.open(path).expect("Could not open");
	
	file.write(&header).expect("Could not write");
	file.write(&body).expect("Could not write");

	drop(file);

	Ok(())
}