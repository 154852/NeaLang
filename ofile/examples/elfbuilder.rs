use std::io::Write;

use ofile::LittleEndian64;
use ofile::elf::*;
use ofile::elfbuilder;
use ofile::elfbuilder::*;

fn main() {
	let text = vec![
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

	let mut elf = StaticELF::new();

	elf.push_symbol(elfbuilder::Symbol::Function("main".to_string(), 0x401000, text.len() as u64));

	elf.set_text(0x401000, text);
	elf.set_data(0x402000, data);
	elf.set_rodata(0x403000, rodata);

	let header = Header::new_with_entry(ABI::SysV, ObjectFileType::Executable, Machine::X8664, 0x401000);
	let (header, body) = elf.encode::<LittleEndian64>(header);

	let mut file = std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.open("ofile/examples/generated.elf").expect("Could not open");
	
	file.write(&header).expect("Could not write");
	file.write(&body).expect("Could not write");

	drop(file);
}