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
	if !relocatable {
		return Err("Macho files currently must be relocatable. Try building with -c.".to_string());
	}

	// 1. Prepare the header, this includes load commands such as segments, strtab, symtab, dystrtab
	
	let mut header = macho::Header::new(macho::Cpu::Arm64, macho::FileType::Object);

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
				align: 2,
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
				align: 3,
				reloff: 0,
				nreloc: 0,
				sectype: macho::SectionType::Regular,
				attrs: 0
			}
		],
		flags: 0
	}));

	let symtab = header.push(macho::Command::SymTab(macho::SymTab::default()));

	let mut macho = macho::Builder::<ofile::LittleEndian64>::new(header);

	// 2. Add the wilderness, including code, data, etc
	// 	+ update the section offsets / sizes in the header (seg and sect)
	// 	+ append symbols, append to strtab
	// 	+ update dysymtab

    let ctx = ir2arm64::TranslationContext::new();

	// let mut func_locations = Vec::new();
	let mut func_locations = vec![];
	let mut arm64_encoding = arm64::EncodeContext::new();

	// 1. Map address space, ignoring symbols for now
	for func in unit.functions().iter() {
		if func.is_extern() {
			func_locations.push(0);
			continue
		}

		let ins = ctx.translate_function(&func, unit);

		let (addr, _) = arm64_encoding.append_function(&ins);
		func_locations.push(addr);
	}

	let (text, text_relocs) = arm64_encoding.take();

	let mut global_offsets = Vec::new();
	let mut globals = Vec::new();
	let mut globals_relocs = Vec::new();
	let gid_allocator = ir2arm64::GlobalIDAllocator::new(unit);
	for global in unit.globals().iter() {
		while globals.len() % 8 != 0 { globals.push(0); }

		global_offsets.push(globals.len());
		globals.extend(ctx.translate_global(global, unit, &gid_allocator, &mut globals_relocs, globals.len(), 0));
	}

	let mut tmp_idx = 0;
	let mut func_symbols = HashMap::new();
	let mut global_symbols = HashMap::new();
	// 1. Locally push internal function symbols
	// for (i, func) in unit.functions().iter().enumerate() {
	// 	if func.is_extern() { continue }

	// 	let name_idx = macho.string(&format!("ltmp{}", tmp_idx));
	// 	macho.push_symbol(macho::Symbol {
	// 		stridx: name_idx,
	// 		typ: macho::SymbolType::Sect,
	// 		external: false,
	// 		private: true,
	// 		sect: 1,
	// 		desc: 0,
	// 		value: func_locations[i] as u64
	// 	});

	// 	// func_symbols.insert(i, idx);
	// 	tmp_idx += 1;
	// }

	// 2. Locally push global symbols and their aliases
	for i in 0..unit.globals().len() {
		let name_idx = macho.string(&format!("ltmp{}", tmp_idx));
		let idx = macho.push_symbol(macho::Symbol {
			stridx: name_idx,
			typ: macho::SymbolType::Sect,
			external: false,
			private: true,
			sect: 2,
			desc: 0,
			value: global_offsets[i] as u64 + text.len() as u64
		});
		global_symbols.insert(i, idx);

		tmp_idx += 1;
	}
	
	// 3. Globally push internal function symbols
	for (i, func) in unit.functions().iter().enumerate() {
		if func.is_extern() { continue }

		let name_idx = macho.string(&mangle_func_name(func));
		let idx = macho.push_symbol(macho::Symbol {
			stridx: name_idx,
			typ: macho::SymbolType::Sect,
			external: true,
			private: false,
			sect: 1,
			desc: 0,
			value: func_locations[i] as u64
		});

		func_symbols.insert(i, idx);
	}

	// 4. Globally push external function symbols
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

	macho.section_mut(0).size = text.len() as u64;
	macho.section_mut(0).nreloc = text_relocs.len() as u32;
	
	macho.section_mut(1).addr = text.len() as u64;
	macho.section_mut(1).size = globals.len() as u64;
	macho.section_mut(1).nreloc = globals_relocs.len() as u32;

	let mut first_text_reloc = None;
	for reloc in text_relocs {
		let reloc = match reloc.0.symbol() {
			arm64::RelocationType::LocalFunctionSymbol(_) => panic!(),
			arm64::RelocationType::RelativeGlobalSymbol(idx) => {
				match gid_allocator.object_id_of_global_id(*idx) {
					ir2arm64::GlobalObjectId::Global(global) => {
						match reloc.0.mode() {
							arm64::InsRelocMode::Branch26 => unreachable!(),
							arm64::InsRelocMode::Branch19Shift5 => unreachable!(),
							arm64::InsRelocMode::Page21 => macho.push_reloc(macho::Reloc {
								addr: reloc.1 as u32,
								symbolnum: *global_symbols.get(&global.idx()).unwrap(),
								pcrel: true,
								length: 2,
								exter: true,
								typ: macho::RelocType::Arm64Page21
							}),
							arm64::InsRelocMode::PageOff12 => macho.push_reloc(macho::Reloc {
								addr: reloc.1 as u32,
								symbolnum: *global_symbols.get(&global.idx()).unwrap(),
								pcrel: false,
								length: 2,
								exter: true,
								typ: macho::RelocType::Arm64PageOff12
							}),
						}
					},
					ir2arm64::GlobalObjectId::Function(global) => {
						macho.push_reloc(macho::Reloc {
							addr: reloc.1 as u32,
							symbolnum: *func_symbols.get(&global.idx()).unwrap(),
							pcrel: true,
							length: 2,
							exter: true,
							typ: match reloc.0.mode() {
								arm64::InsRelocMode::Branch26 => macho::RelocType::Arm64Branch26,
								arm64::InsRelocMode::Branch19Shift5 => unreachable!(),
								arm64::InsRelocMode::Page21 => macho::RelocType::Arm64Page21,
								arm64::InsRelocMode::PageOff12 => macho::RelocType::Arm64PageOff12,
							}
						})
					}
				}
			},
			arm64::RelocationType::AbsoluteGlobalSymbol(_) => panic!(),
		};

		if first_text_reloc.is_none() {
			first_text_reloc = Some(reloc);
		}
	}

	let mut first_data_reloc = None;
	for reloc in globals_relocs {
		match reloc.kind() {
			arm64::RelocationType::LocalFunctionSymbol(_) => panic!(),
			arm64::RelocationType::RelativeGlobalSymbol(_) => panic!(),
			arm64::RelocationType::AbsoluteGlobalSymbol(idx) => {
				match gid_allocator.object_id_of_global_id(*idx) {
					ir2arm64::GlobalObjectId::Global(global) => {
						let reloc_idx = macho.push_reloc(macho::Reloc {
							addr: reloc.offset() as u32,
							symbolnum: *global_symbols.get(&global.idx()).unwrap(),
							pcrel: false,
							length: 3,
							exter: true,
							typ: macho::RelocType::Arm64Unsigned
						});
	
						if first_data_reloc.is_none() {
							first_data_reloc = Some(reloc_idx);
						}
					},
					ir2arm64::GlobalObjectId::Function(_)  => todo!()
				}
			},
		}
	}

	let text_offset = macho.wilderness_append(&text);
	macho.section_mut(0).offset = text_offset;

	let data_offset = macho.wilderness_append(&globals);
	macho.section_mut(1).offset = data_offset;

	macho.segment_mut_at(0).fileoff = text_offset as u64;
	macho.segment_mut_at(0).filesize = text.len() as u64 + globals.len() as u64;
	macho.segment_mut_at(0).vmsize = text.len() as u64 + globals.len() as u64;

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