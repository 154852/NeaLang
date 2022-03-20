use ofile::macho;

fn main() {
	let text_data = vec![
		0x55,
		0x48, 0x89, 0xe5,
		0x48, 0x83, 0xec, 0x10,
		
		0x48, 0x8d, 0x3d, 0x0b, 0x00, 0x00, 0x00,
		0xe8, 0x00, 0x00, 0x00, 0x00,
		
		0x48, 0x83, 0xc4, 0x10,
		0x5d,
		0xc3
	];
	
	let mut cstring_data = "Hello World!".as_bytes().to_vec();
	cstring_data.push(0);

	let mut header = macho::Header::new(macho::Cpu::X8664, macho::FileType::Object);
	
	header.push(macho::Command::Segment(macho::Segment {
		name: String::new(),
		vmaddr: 0,
		vmsize: text_data.len() as u64 + cstring_data.len() as u64,
		fileoff: 0,
		filesize: text_data.len() as u64 + cstring_data.len() as u64,
		maxprot: macho::PROT_READ | macho::PROT_WRITE | macho::PROT_EXEC,
		initprot: macho::PROT_READ | macho::PROT_WRITE | macho::PROT_EXEC,
		sects: vec![
			macho::Section {
				sectname: String::from("__text"),
				segname: String::from("__TEXT"),
				addr: 0,
				size: text_data.len() as u64,
				offset: 0,
				align: 4,
				reloff: 0,
				nreloc: 2,
				sectype: macho::SectionType::Regular,
				attrs: macho::SECTION_PURE_INS | macho::SECTION_SOME_INS
			},
			macho::Section {
				sectname: String::from("__cstring"),
				segname: String::from("__TEXT"),
				addr: text_data.len() as u64,
				size: cstring_data.len() as u64,
				offset: 0,
				align: 0,
				reloff: 0,
				nreloc: 0,
				sectype: macho::SectionType::CStringLiterals,
				attrs: 0
			}
		],
		flags: 0
	}));

	header.push(macho::Command::BuildVersion(macho::BuildVersion {
		platform: macho::Platform::MacOs,
		min_os: (11, 0, 0),
		sdk: (11, 1, 0)
	}));

	let symtab = header.push(macho::Command::SymTab(macho::SymTab::default()));

	header.push(macho::Command::DySymTab({
		let mut dysymtab = macho::DySymTab::default();

		dysymtab.externdef_idx = 0;
		dysymtab.externdef_count = 1;

		dysymtab.undef_idx = 1;
		dysymtab.undef_count = 1;
		
		dysymtab
	}));

	let mut macho = macho::Builder::<ofile::LittleEndian64>::new(header);

	let text_offset = macho.wilderness_append(&text_data);
	let cstrings_offset = macho.wilderness_append(&cstring_data);

	macho.segment_mut_at(0).fileoff = text_offset as u64;
	macho.section_mut(0).offset = text_offset;
	macho.section_mut(1).offset = cstrings_offset;

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

	let string_reloc = macho.push_reloc(macho::Reloc {
		addr: 11,
		symbolnum: 2, // cstring section
		pcrel: true,
		length: 2,
		exter: false,
		typ: macho::RelocType::Signed
	});
	macho.section_mut(0).reloff = macho.offset_of_reloc(string_reloc);

	macho.push_reloc(macho::Reloc {
		addr: 16,
		symbolnum: 1, // _puts symbol
		pcrel: true,
		length: 2,
		exter: true,
		typ: macho::RelocType::Branch
	});

	macho.complete_symtab(symtab);

	std::fs::write("ofile/examples/generated-macho.o", macho.build()).expect("Could not write to file");
}