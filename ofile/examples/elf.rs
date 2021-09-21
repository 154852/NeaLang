use std::io::Write;

use ofile::elf::*;

const BASE: u64 = 0x401000;

fn main() {
	let mut elf = ELF::new(Header::new(ABI::SysV, ObjectFileType::Executable, Machine::X8664));

	let text = vec![
		0x48, 0xc7, 0xc7, 0x2a, 0x00, 0x00, 0x00, // mov rdi, 42
		0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, // mov rax, 0x3c
		0x0f, 0x05 // syscall
	];

	let data = vec![
		1, 2, 3, 4
	];

	let text_ph_idx = elf.push_program_header(ProgramHeader::new_load(0, BASE, BASE, text.len() as u64));
	elf.program_header_mut(text_ph_idx).set_flags(true, false, true);
	elf.header_mut().set_entry_point(BASE);

	let data_ph_idx = elf.push_program_header(ProgramHeader::new_load(0, BASE + 0x1000, BASE + 0x1000, data.len() as u64));

	let mut raw_data = Vec::<u8>::new();

	let size = elf.size::<ofile::LittleEndian64>() as u64;

	ofile::align_up_vec_offset(&mut raw_data, size as usize, 1 << 12);
	elf.program_header_mut(text_ph_idx).set_offset(raw_data.len() as u64 + size);
	raw_data.extend(&text);

	ofile::align_up_vec_offset(&mut raw_data, size as usize, 1 << 12);
	elf.program_header_mut(data_ph_idx).set_offset(raw_data.len() as u64 + size);
	raw_data.extend(&data);

	let mut file = std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.open("ofile/examples/generated.elf").expect("Could not open");
	
	file.write(&elf.encode::<ofile::LittleEndian64>()).expect("Could not write");
	file.write(&raw_data).expect("Could not write");

	drop(file);
}