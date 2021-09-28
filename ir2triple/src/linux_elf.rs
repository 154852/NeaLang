use x86;
use ir2x86;
use std::io::Write;
use ofile::{elf, elfbuilder};

const TEXT_BASE: u64 = 0x401000;

pub fn encode(unit: &ir::TranslationUnit, path: &str, relocatable: bool) {
	match unit.validate() {
		Ok(_) => println!("Validated!"),
		Err(e) => panic!("Validation error: {:#?}", e)
	}
	
	let mut elf = if relocatable {
		elfbuilder::StaticELF::new_relocatable()
	} else {
		elfbuilder::StaticELF::new()
	};

	let mut x86_encoding = x86::EncodeContext::new();
	let ctx = ir2x86::TranslationContext::new(x86::Mode::X8664);

	let text_base = if relocatable { 0 } else { TEXT_BASE };

	for (i, func) in unit.functions().iter().enumerate() {
		let mut ins = ctx.translate_function(&func, &unit);
		x86::opt::pass_zero(&mut ins);
		
		let (addr, length) = x86_encoding.append_function(i, &ins);

		elf.push_symbol(elfbuilder::Symbol::Function(func.name().to_string(), text_base + addr as u64, length as u64));
	}

	let text = x86_encoding.finish();
	println!("Assembled!");

	elf.set_text(text_base, text);
	// elf.set_data(0x402000, data);
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
}