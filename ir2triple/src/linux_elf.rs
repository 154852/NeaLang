use x86;
use ir2x86;
use std::io::Write;
use ofile::{elf, elfbuilder};

const TEXT_BASE: u64 = 0x401000;

pub fn encode(unit: &ir::TranslationUnit, path: &str, relocatable: bool) -> Result<(), String> {
	let mut elf = if relocatable {
		elfbuilder::StaticELF::new_relocatable()
	} else {
		elfbuilder::StaticELF::new()
	};

	let mut x86_encoding = x86::EncodeContext::new(relocatable);
	let ctx = ir2x86::TranslationContext::new(x86::Mode::X8664);

	let text_base = if relocatable { 0 } else { TEXT_BASE };

	for (i, func) in unit.functions().iter().enumerate() {
		if func.is_extern() {
			if !relocatable {
				return Err(format!("Cannot import function '{}' with a statically compiled binary", func.name()));
			}
			elf.push_symbol(elfbuilder::Symbol::Relocatable(func.name().to_string()));
			continue;
		}

		let mut ins = ctx.translate_function(&func, unit);
		x86::opt::pass_zero(&mut ins);
		
		let (addr, length) = x86_encoding.append_function(i, &ins);
		elf.push_symbol(elfbuilder::Symbol::Function(func.name().to_string(), text_base + addr as u64, length as u64));
	}

	let data_base = if relocatable { 0 } else { ofile::align_up(text_base + x86_encoding.len() as u64, 1 << 12) };

	let mut relocs = Vec::new();
	let mut data = Vec::new();
	for (i, global) in unit.globals().iter().enumerate() {
		let pushed = ctx.translate_global(global, unit, &mut relocs, unit.functions().len() + i, data.len());
		elf.push_symbol(elfbuilder::Symbol::Object(global.name().map_or_else(|| String::new(), |x| x.to_string()), data_base + data.len() as u64, pushed.len() as u64));
		x86_encoding.append_global(unit.functions().len() + i, data_base as usize + data.len());
		data.extend(pushed);
	}

	if !relocatable {
		for reloc in relocs {
			if !reloc.write_global(&mut data, x86_encoding.global_symbols()) {
				panic!("Undefined symbol, could not build non-relocatable file");
			}
		}
	} else {
		// Data relocations
		for reloc in relocs {
			elf.push_data_relocation(elf::Rela::new(
				reloc.offset() as u64,
				match reloc.kind() {
					x86::RelocationType::AbsoluteGlobalSymbol(id) => *id as u64 + 1,
					_ => panic!("Cannot relocate non-global symbol in .data")
				},
				elf::RelocationType::X86646464,
				reloc.addend()
			));
		}
	}

	let (text, relocs) = x86_encoding.finish();

	// Test relocations
	for reloc in relocs {
		elf.push_text_relocation(elf::Rela::new(
			reloc.offset() as u64,
			match reloc.kind() {
				x86::RelocationType::RelativeGlobalSymbol(id) => *id as u64 + 1,
				_ => unreachable!()
			},
			elf::RelocationType::X8664Plt32,
			reloc.addend()
		));
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