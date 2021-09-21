use std::io::Write;

use ofile::{LittleEndian64, elf::{ABI, Header, Machine, ObjectFileType, Rela}, elfbuilder::{self, StaticELF}};

fn main() {
	let text = vec![
		0xe8, 0x00, 0x00, 0x00, 0x00, // call <addr>
		0x48, 0xc7, 0xc7, 0x2a, 0x00, 0x00, 0x00, // mov rdi, 42
		0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, // mov rax, 0x3c
		0x0f, 0x05 // syscall
	];

	let data = vec![
		1, 2, 3, 4
	];

	let rodata = vec![
		5, 6, 7, 8
	];

	let mut elf = StaticELF::new_relocatable();

	elf.push_symbol(elfbuilder::Symbol::Function("main".to_string(), 0, text.len() as u64));
	let do_something_symbol_idx = elf.push_symbol(elfbuilder::Symbol::Relocatable("do_something".to_string()));

	elf.push_text_relocation(Rela::new(1, do_something_symbol_idx as u64, ofile::elf::RelocationType::X8664Plt32, -4));

	elf.set_text(0, text);
	elf.set_data(0, data);
	elf.set_rodata(0, rodata);

	let header = Header::new(ABI::SysV, ObjectFileType::Relocatable, Machine::X8664);
	let (header, body) = elf.encode::<LittleEndian64>(header);

	let mut file = std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.open("ofile/examples/generated.elf").expect("Could not open");
	
	file.write(&header).expect("Could not write");
	file.write(&body).expect("Could not write");

	drop(file);

	// To link, run `gcc ofile/examples/elfreloc.c ofile/examples/generated.elf -o output`
}