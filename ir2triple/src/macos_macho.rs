use std::collections::HashMap;

use ofile::macho;

fn mangle_func_name(func: &ir::Function) -> String {
	if func.is_entry() {
		return String::from("_main");
	}

    if let Some(ctr) = func.method_of() {
        format!("_{}.{}", ctr.name(), func.name())
    } else {
		format!("_{}", func.name().to_string())
    }
}

pub fn encode(unit: &ir::TranslationUnit, path: &str, relocatable: bool) -> Result<(), String> {
	assert!(relocatable);

	let mut internal_sym_count = 0;
	let mut imported_sym_count = 0;
	for func in unit.functions() {
		if func.is_extern() {
			imported_sym_count += 1;
		} else {
			internal_sym_count += 1;
		}
	}

	// 1. Prepare the header, this includes load commands such as segments, strtab, symtab, dystrtab
	
	let mut header = macho::Header::new(macho::Cpu::X8664, macho::FileType::Object);

	header.push(macho::Command::Segment(macho::Segment {
		name: String::new(),
		vmaddr: 0,
		vmsize: 0,
		fileoff: 0,
		filesize: 0,
		maxprot: macho::PROT_READ | macho::PROT_WRITE | macho::PROT_EXEC,
		initprot: macho::PROT_READ | macho::PROT_WRITE | macho::PROT_EXEC,
		sects: vec![
			macho::Section {
				sectname: String::from("__text"),
				segname: String::from("__TEXT"),
				addr: 0,
				size: 0,
				offset: 0,
				align: 4,
				reloff: 0,
				nreloc: 0,
				sectype: macho::SectionType::Regular,
				attrs: macho::SECTION_PURE_INS | macho::SECTION_SOME_INS
			},
			macho::Section {
				sectname: String::from("__data"),
				segname: String::from("__DATA"),
				addr: 0,
				size: 0,
				offset: 0,
				align: 0,
				reloff: 0,
				nreloc: 0,
				sectype: macho::SectionType::Regular,
				attrs: 0
			}
		],
		flags: 0
	}));

	let symtab = header.push(macho::Command::SymTab(macho::SymTab::default()));
	header.push(macho::Command::DySymTab({
		let mut dysymtab = macho::DySymTab::default();

		dysymtab.externdef_idx = 0;
		dysymtab.externdef_count = internal_sym_count;

		dysymtab.undef_idx = internal_sym_count;
		dysymtab.undef_count = imported_sym_count;

		dysymtab
	}));

	let mut macho = macho::Builder::<ofile::LittleEndian64>::new(header);

	// 2. Add the wilderness, including code, data, etc
	// 	+ update the section offsets / sizes in the header (seg and sect)
	// 	+ append symbols, append to strtab
	// 	+ update dysymtab

    let ctx = ir2x86::TranslationContext::new(x86::Mode::X8664);

	let mut globals_relocs = Vec::new();
	let mut globals = Vec::new();
	let mut global_offsets = Vec::new();

	let gid_allocator = ir2x86::GlobalIDAllocator::new(unit);
	for global in unit.globals() {
		global_offsets.push(globals.len());
		globals.extend(ctx.translate_global(global, unit, &gid_allocator, &mut globals_relocs, globals.len(), 0));
	}

	macho.section_mut(1).size = globals.len() as u64;
	macho.section_mut(1).nreloc = globals_relocs.len() as u32;

	let mut func_symbols = HashMap::new();
	let mut x86_encoding = x86::EncodeContext::new();

	for (i, func) in unit.functions().iter().enumerate() {
		if func.is_extern() { continue }

		let mut ins = ctx.translate_function(&func, unit);
        x86::opt::pass_zero(&mut ins);

		let (addr, _) = x86_encoding.append_function(&ins);

		let name_idx = macho.string(&mangle_func_name(func));
		let idx = macho.push_symbol(macho::Symbol {
			stridx: name_idx,
			typ: macho::SymbolType::Sect,
			external: true,
			private: false,
			sect: 1,
			desc: 0,
			value: addr as u64
		});

		func_symbols.insert(i, idx);
	}

	for (i, func) in unit.functions().iter().enumerate() {
		if !func.is_extern() { continue }

		let name_idx = macho.string(&mangle_func_name(func));
		let idx = macho.push_symbol(macho::Symbol {
			stridx: name_idx,
			typ: macho::SymbolType::Undefined,
			external: true,
			private: false,
			sect: 0,
			desc: 0,
			value: 0
		});

		func_symbols.insert(i, idx);
	}

	let text_length = x86_encoding.len();
	let (mut text, text_relocs) = x86_encoding.take();

	macho.section_mut(0).size = text_length as u64;
	macho.section_mut(0).nreloc = text_relocs.len() as u32;
	macho.section_mut(1).addr = text_length as u64;

	let mut first_text_reloc = None;
	for reloc in text_relocs {
		if first_text_reloc.is_none() {
			first_text_reloc = Some(0);
		}

		match reloc.kind() {
			x86::RelocationType::LocalFunctionSymbol(_) => panic!(),
			x86::RelocationType::RelativeGlobalSymbol(idx) => {
				match gid_allocator.object_id_of_global_id(*idx) {
					ir2x86::GlobalObjectId::Global(global) => {
						let global_addr = global_offsets[global.idx()];
						text[reloc.offset()..reloc.offset() + 4].copy_from_slice(
							&((global_addr + (text_length - reloc.offset() - 4)) as u32).to_le_bytes()
						);

						macho.push_reloc(macho::Reloc {
							addr: reloc.offset() as u32,
							symbolnum: 2, // __data section
							pcrel: true,
							length: 2,
							exter: false,
							typ: macho::RelocType::Signed
						});
					},
					ir2x86::GlobalObjectId::Function(global) => {
						macho.push_reloc(macho::Reloc {
							addr: reloc.offset() as u32,
							symbolnum: *func_symbols.get(&global.idx()).unwrap(),
							pcrel: true,
							length: 2,
							exter: true,
							typ: macho::RelocType::Branch
						});
					}
				}
			},
			x86::RelocationType::AbsoluteGlobalSymbol(_) => panic!(),
		};
	}

	let mut first_data_reloc = None;
	for reloc in globals_relocs {
		match reloc.kind() {
			x86::RelocationType::LocalFunctionSymbol(_) => panic!(),
			x86::RelocationType::RelativeGlobalSymbol(_) => panic!(),
			x86::RelocationType::AbsoluteGlobalSymbol(idx) => {
				match gid_allocator.object_id_of_global_id(*idx) {
					ir2x86::GlobalObjectId::Global(global) => {
						let global_addr = global_offsets[global.idx()];
						globals[reloc.offset()..reloc.offset() + 8].copy_from_slice(
							&(global_addr as u64 + text_length as u64).to_le_bytes()
						);

						let reloc_idx = macho.push_reloc(macho::Reloc {
							addr: reloc.offset() as u32,
							symbolnum: 2, // __data section
							pcrel: false,
							length: 3,
							exter: false,
							typ: macho::RelocType::Unsigned
						});

						if first_data_reloc.is_none() {
							first_data_reloc = Some(reloc_idx);
						}
					},
					ir2x86::GlobalObjectId::Function(_)  => todo!()
				}
			},
		}
	}

	let text_offset = macho.wilderness_append(&text);
	macho.section_mut(0).offset = text_offset;

	let data_offset = macho.wilderness_append(&globals);
	macho.section_mut(1).offset = data_offset;

	macho.segment_mut_at(0).fileoff = text_offset as u64;
	macho.segment_mut_at(0).filesize = text_length as u64 + globals.len() as u64;
	macho.segment_mut_at(0).vmsize = text_length as u64 + globals.len() as u64;

	// 3. Add relocations in their full form
	// 	+ update the section reloffs / relnum in the header	

	if let Some(reloc_idx) = first_text_reloc {
		macho.section_mut(0).reloff = macho.offset_of_reloc(reloc_idx);
	}

	if let Some(reloc_idx) = first_data_reloc {
		macho.section_mut(1).reloff = macho.offset_of_reloc(reloc_idx);
	}
	
	// 4. complete_symtab()

	macho.complete_symtab(symtab);

	std::fs::write(path, macho.build()).expect("Could not write");

	Ok(())
}