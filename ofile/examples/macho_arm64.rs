use ofile::macho;

fn main() {
	let text_data = vec![0xaa; 16];
    let data_data = vec![0xbb; 8];
	
	let mut header = macho::Header::new(macho::Cpu::Arm64, macho::FileType::Object);
	
	header.push(macho::Command::Segment(macho::Segment {
		name: String::new(),
		vmaddr: 0,
		vmsize: text_data.len() as u64 + data_data.len() as u64,
		fileoff: 0,
		filesize: text_data.len() as u64 + data_data.len() as u64,
		maxprot: macho::PROT_READ | macho::PROT_WRITE | macho::PROT_EXEC,
		initprot: macho::PROT_READ | macho::PROT_WRITE | macho::PROT_EXEC,
		sects: vec![
			macho::Section {
				sectname: String::from("__text"),
				segname: String::from("__TEXT"),
				addr: 0,
				size: text_data.len() as u64,
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
				addr: text_data.len() as u64,
				size: data_data.len() as u64,
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

	// header.push(macho::Command::DySymTab({
	// 	let mut dysymtab = macho::DySymTab::default();

	// 	// dysymtab.locals_idx = 0;
    //     // dysymtab.locals_count = 0;

	// 	// dysymtab.externdef_idx = dysymtab.locals_idx + dysymtab.locals_count;
	// 	// dysymtab.externdef_count = 2;

	// 	// dysymtab.undef_idx = dysymtab.externdef_idx + dysymtab.externdef_count;
	// 	// dysymtab.undef_count = 1;
		
	// 	dysymtab
	// }));

	let mut macho = macho::Builder::<ofile::LittleEndian64>::new(header);

	let text_offset = macho.wilderness_append(&text_data);
	let data_offset = macho.wilderness_append(&data_data);

	macho.segment_mut_at(0).fileoff = text_offset as u64;
	macho.section_mut(0).offset = text_offset;
	macho.section_mut(1).offset = data_offset;

    // let global_string = macho.string("");
	macho.push_symbol(macho::Symbol {
		stridx: 0,
		typ: macho::SymbolType::Sect,
		external: true,
		private: false,
		sect: 2,
		desc: 0,
		value: text_data.len() as u64
	});

	let main_string = macho.string("_main");
	macho.push_symbol(macho::Symbol {
		stridx: main_string,
		typ: macho::SymbolType::Sect,
		external: true,
		private: false,
		sect: 1,
		desc: 0,
		value: 0
	});

	let puts_string = macho.string("_puts");
	macho.push_symbol(macho::Symbol {
		stridx: puts_string,
		typ: macho::SymbolType::Undefined,
		external: true,
		private: false,
		sect: 0,
		desc: 0,
		value: 0
	});

    macho.complete_symtab(symtab);

	std::fs::write("ofile/examples/generated-macho.o", macho.build()).expect("Could not write to file");
}