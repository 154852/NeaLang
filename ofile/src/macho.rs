use std::marker::PhantomData;

use crate::EncodableInt;

pub enum Cpu {
	X86,
	X8664
}

impl Cpu {
	pub fn cpu_type(&self) -> u32 {
		match self {
			Cpu::X86 => 7,
			Cpu::X8664 => 7 | 0x01000000,
		}
	}

	pub fn cpu_subtype(&self) -> u32 {
		match self {
			Cpu::X86 => 3,
			Cpu::X8664 => 3,
		}
	}
}

pub enum FileType {
	Object,
	Executable
}

impl FileType {
	pub fn file_type(&self) -> u32 {
		match self {
			FileType::Object => 1,
			FileType::Executable => 2,
		}
	}
}

pub enum SectionType {
	Regular,
	ZeroFill,
	CStringLiterals,
	Byte4Literals,
	Byte8Literals,
	PointerLiterals
}

impl SectionType {
	pub fn section_type(&self) -> u32 {
		match self {
			SectionType::Regular => 0,
			SectionType::ZeroFill => 1,
			SectionType::CStringLiterals => 2,
			SectionType::Byte4Literals => 3,
			SectionType::Byte8Literals => 4,
			SectionType::PointerLiterals => 5,
		}
	}
}

pub const SECTION_PURE_INS: u32 = 0x80000000;
pub const SECTION_SOME_INS: u32 = 0x00000400;

pub struct Section {
	pub sectname: String,
	pub segname: String,
	pub addr: u64,
	pub size: u64,
	pub offset: u32,
	pub align: u32,
	pub reloff: u32,
	pub nreloc: u32,
	
	pub sectype: SectionType,
	pub attrs: u32
}

impl Section {
	pub fn size<I: EncodableInt>() -> usize {
		if I::size() == 4 {
			32 + 9*4
		} else {
			32 + 2*8 + 8*4
		}
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		assert!(self.sectname.len() <= 16);
		data.extend(self.sectname.as_bytes());
		for _ in self.sectname.len()..16 { data.push(0); }

		assert!(self.segname.len() <= 16);
		data.extend(self.segname.as_bytes());
		for _ in self.segname.len()..16 { data.push(0); }

		I::encode(self.addr, data);
		I::encode(self.size, data);

		data.extend(I::u32(self.offset));
		data.extend(I::u32(self.align));
		data.extend(I::u32(self.reloff));
		data.extend(I::u32(self.nreloc));
		data.extend(I::u32(self.sectype.section_type() | self.attrs));

		if I::size() == 4 {
			data.extend([0; 8]);
		} else {
			data.extend([0; 12]);
		}
	}
}

pub const PROT_NONE: u32 = 0;
pub const PROT_READ: u32 = 1;
pub const PROT_WRITE: u32 = 2;
pub const PROT_EXEC: u32 = 4;

pub struct Segment {
	pub name: String,
		
	pub vmaddr: u64,
	pub vmsize: u64,
	
	pub fileoff: u64,
	pub filesize: u64,
	
	pub maxprot: u32,
	pub initprot: u32,

	pub sects: Vec<Section>,
	pub flags: u32
}

impl Segment {
	pub fn size<I: EncodableInt>(&self) -> usize {
		I::size() * 4 + 4 * 6 + 16 + self.sects.len() * Section::size::<I>()
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		data.extend(I::u32(0x19));
		data.extend(I::u32(self.size::<I>() as u32));
		
		assert!(self.name.len() <= 16);
		data.extend(self.name.as_bytes());
		for _ in self.name.len()..16 { data.push(0); }

		I::encode(self.vmaddr, data);
		I::encode(self.vmsize, data);
		I::encode(self.fileoff, data);
		I::encode(self.filesize, data);

		data.extend(I::u32(self.maxprot));
		data.extend(I::u32(self.initprot));
		data.extend(I::u32(self.sects.len() as u32));
		data.extend(I::u32(self.flags));

		for sect in self.sects.iter() {
			sect.encode::<I>(data);
		}
	}
}

pub struct SymTab {
	pub symoff: u32,
	pub nsyms: u32,
	pub stroff: u32,
	pub strsize: u32
}

impl SymTab {
	pub fn size() -> usize {
		4*4 + 8
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		data.extend(I::u32(0x02));
		data.extend(I::u32(SymTab::size() as u32));

		data.extend(I::u32(self.symoff));
		data.extend(I::u32(self.nsyms));
		data.extend(I::u32(self.stroff));
		data.extend(I::u32(self.strsize));
	}
}

impl Default for SymTab {
	fn default() -> Self {
		SymTab {
			symoff: 0, nsyms: 0,
			stroff: 0, strsize: 0
		}
	}
}

pub struct DySymTab {
	pub locals_idx: u32,
	pub locals_count: u32,

	pub externdef_idx: u32,
	pub externdef_count: u32,

	pub undef_idx: u32,
	pub undef_count: u32,

	pub toc_idx: u32,
	pub toc_count: u32,

	pub modtab_idx: u32,
	pub modtab_count: u32,

	pub extref_idx: u32,
	pub extref_count: u32,

	pub indirect_idx: u32,
	pub indirect_count: u32,

	pub extrel_idx: u32,
	pub extrel_count: u32,

	pub localrel_idx: u32,
	pub localrel_count: u32,
}

impl DySymTab {
	pub fn size() -> usize {
		4*2*9 + 8
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		data.extend(I::u32(0x0b));
		data.extend(I::u32(DySymTab::size() as u32));

		data.extend(I::u32(self.locals_idx)); data.extend(I::u32(self.locals_count));
		data.extend(I::u32(self.externdef_idx)); data.extend(I::u32(self.externdef_count));
		data.extend(I::u32(self.undef_idx)); data.extend(I::u32(self.undef_count));
		data.extend(I::u32(self.toc_idx)); data.extend(I::u32(self.toc_count));
		data.extend(I::u32(self.modtab_idx)); data.extend(I::u32(self.modtab_count));
		data.extend(I::u32(self.extref_idx)); data.extend(I::u32(self.extref_count));
		data.extend(I::u32(self.indirect_idx)); data.extend(I::u32(self.indirect_count));
		data.extend(I::u32(self.extrel_idx)); data.extend(I::u32(self.extrel_count));
		data.extend(I::u32(self.localrel_idx)); data.extend(I::u32(self.localrel_count));
	}
}

impl Default for DySymTab {
    fn default() -> Self {
        DySymTab {
            locals_idx: 0, locals_count: 0, externdef_idx: 0, externdef_count: 0,
            undef_idx: 0, undef_count: 0, toc_idx: 0, toc_count: 0,
            modtab_idx: 0, modtab_count: 0, extref_idx: 0, extref_count: 0,
            indirect_idx: 0, indirect_count: 0, extrel_idx: 0, extrel_count: 0,
            localrel_idx: 0, localrel_count: 0,
        }
    }
}

pub enum Platform {
	MacOs,
	IOs,
	TvOs,
	WatchOs
}

impl Platform {
	pub fn platform(&self) -> u32 {
		match self {
			Platform::MacOs => 1,
			Platform::IOs => 2,
			Platform::TvOs => 3,
			Platform::WatchOs => 4,
		}
	}
}

pub struct BuildVersion {
	pub platform: Platform,
	pub min_os: (u16, u8, u8),
	pub sdk: (u16, u8, u8),
	
	// Assume 0 ntools
}

impl BuildVersion {
	pub fn size() -> usize {
		8 + 4*4
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		data.extend(I::u32(0x32));
		data.extend(I::u32(BuildVersion::size() as u32));

		data.extend(I::u32(self.platform.platform()));

		data.push(self.min_os.2);
		data.push(self.min_os.1);
		data.extend(I::u16(self.min_os.0));
		
		data.push(self.sdk.2);
		data.push(self.sdk.1);
		data.extend(I::u16(self.sdk.0));

		data.extend(I::u32(0));
	}
}

pub enum Command {
	Segment(Segment),
	SymTab(SymTab),
	DySymTab(DySymTab),
	BuildVersion(BuildVersion)
}

impl Command {
	pub fn size<I: EncodableInt>(&self) -> usize {
		match self {
			Command::Segment(segment) => segment.size::<I>(),
			Command::SymTab(_) => SymTab::size(),
			Command::DySymTab(_) => DySymTab::size(),
			Command::BuildVersion(_) => BuildVersion::size()
		}
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		match self {
    		Command::Segment(segment) => segment.encode::<I>(data),
			Command::SymTab(symtab) => symtab.encode::<I>(data),
			Command::DySymTab(dysymtab) => dysymtab.encode::<I>(data),
			Command::BuildVersion(build_version) => build_version.encode::<I>(data)
		}
	}
}

pub struct Header {
	cpu: Cpu,
	file_type: FileType,
	commands: Vec<Command>,
}

impl Header {
	pub fn new(cpu: Cpu, file_type: FileType) -> Header {
		Header {
			cpu, file_type,
			commands: Vec::new()
		}
	}

	pub fn push(&mut self, command: Command) -> usize {
		self.commands.push(command);
		self.commands.len() - 1
	}

	pub fn segment_mut_at(&mut self, idx: usize) -> &mut Segment {
		match self.commands.get_mut(idx) {
			Some(Command::Segment(segment)) => segment,
			_ => panic!("Not a segment")
		}
	}

	pub fn section_mut(&mut self, mut idx: usize) -> &mut Section {
		for command in self.commands.iter_mut() {
			match command {
				Command::Segment(segment) => {
					if segment.sects.len() > idx {
						return &mut segment.sects[idx];
					} else {
						idx -= segment.sects.len();
					}
				},
				_ => {}
			}
		}

		panic!("No section with index")
	}

	pub fn size<I: EncodableInt>(&self) -> usize {
		let mut size = 4*8;

		for cmd in self.commands.iter() {
			size += cmd.size::<I>();
		}

		size
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		match I::size() {
			4 => data.extend(I::u32(0xfeedface)),
			8 => data.extend(I::u32(0xfeedfacf)),
			_ => panic!()
		}

		data.extend(I::u32(self.cpu.cpu_type()));
		data.extend(I::u32(self.cpu.cpu_subtype()));
		
		data.extend(I::u32(self.file_type.file_type()));

		data.extend(I::u32(self.commands.len() as u32));
		let mut size = 0;
		for cmd in self.commands.iter() {
			size += cmd.size::<I>();
		}
		data.extend(I::u32(size as u32));

		// Flags
		data.extend(I::u32(0x2000)); // ?
		
		// Reserved
		data.extend([0; 4]);

		for cmd in self.commands.iter() {
			cmd.encode::<I>(data);
		}
	}
}

pub enum SymbolType {
	Undefined,
	Absolute,
	Sect,
	PreboundUndefined,
	Indirect
}

impl SymbolType {
	pub fn symbol_type(&self) -> u8 {
		match self {
			SymbolType::Undefined => 0x0,
			SymbolType::Absolute => 0x2,
			SymbolType::Sect => 0xe,
			SymbolType::PreboundUndefined => 0xc,
			SymbolType::Indirect => 0xa,
		}
	}
}

pub struct Symbol {
	pub stridx: u32,
	pub typ: SymbolType,
	pub external: bool,
	pub private: bool,
	pub sect: u8,
	pub desc: u16,
	pub value: u64
}

impl Symbol {
	pub fn size<I: EncodableInt>() -> usize {
		8 + I::size()
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		data.extend(I::u32(self.stridx));
		data.push(if self.external { 1 } else { 0 } | self.typ.symbol_type() | if self.private { 1 << 4 } else { 0 });
		data.push(self.sect);
		data.extend(I::u16(self.desc));
		I::encode(self.value, data);
	}
}

pub type NList = Symbol;

pub enum RelocType {
	Unsigned,
	Signed,
	Branch,
	GotLoad,
	Got,
	Subtractor,
	Signed1,
	Signed2,
	Signed4
}

impl RelocType {
	pub fn reloc_type(&self) -> u8 {
		match self {
			RelocType::Unsigned => 0,
			RelocType::Signed => 1,
			RelocType::Branch => 2,
			RelocType::GotLoad => 3,
			RelocType::Got => 4,
			RelocType::Subtractor => 5,
			RelocType::Signed1 => 6,
			RelocType::Signed2 => 7,
			RelocType::Signed4 => 8,
		}
	}
}

pub struct Reloc {
	pub addr: u32,
	pub symbolnum: u32, // either a 1 indexed section, or a 0 indexed symbol
	pub pcrel: bool,
	pub length: u8,
	pub exter: bool,
	pub typ: RelocType
}

impl Reloc {
	pub fn size() -> usize {
		8
	}

	pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
		data.extend(I::u32(self.addr));
		let extra_byte = if self.pcrel { 1 } else { 0 } | (self.length << 1) | if self.exter { 1 << 3 } else { 0 } | (self.typ.reloc_type() << 4);
		data.extend(I::u32(self.symbolnum | ((extra_byte as u32) << 24)));
	}
}

pub struct Builder<I: EncodableInt> {
	_phantom: PhantomData<I>,
	
	header: Header,
	wilderness: Vec<u8>,
	symbols: Vec<Symbol>,
	relocs: Vec<Reloc>,
	strtab: Vec<u8>
}

/// 1. Prepare the header, this includes load commands such as segments, strtab, symtab, dystrtab
/// 2. Add the wilderness, including code, data, etc
/// 	+ update the section offsets in the header
/// 	+ append symbols, append to strtab
/// 3. Add relocations in their full form
/// 	+ update the section reloffs / relnum in the header
/// 4. complete_symtab()
impl<I: EncodableInt> Builder<I> {
	pub fn new(header: Header) -> Builder<I> {
		Builder {
			_phantom: Default::default(),
			header,
			wilderness: Vec::new(),
			symbols: Vec::new(),
			relocs: Vec::new(),
			strtab: vec![0]
		}
	}

	pub fn command_mut(&mut self, idx: usize) -> &mut Command {
		self.header.commands.get_mut(idx).expect("Invalid command index")
	}

	pub fn segment_mut_at(&mut self, idx: usize) -> &mut Segment {
		self.header.segment_mut_at(idx)
	}

	pub fn section_mut(&mut self, idx: usize) -> &mut Section {
		self.header.section_mut(idx)
	}

	pub fn push_symbol(&mut self, symbol: Symbol) -> u32 {
		self.symbols.push(symbol);
		self.symbols.len() as u32 - 1
	}

	pub fn push_reloc(&mut self, reloc: Reloc) -> usize {
		self.relocs.push(reloc);
		self.relocs.len() - 1
	}

	pub fn symtab_offset(&self) -> u32 {
		(self.header.size::<I>() + crate::align_up(self.wilderness.len(), 1 << 3)) as u32
	}

	pub fn offset_of_reloc(&self, idx: usize) -> u32 {
		(self.header.size::<I>() + crate::align_up(self.wilderness.len(), 1 << 3) + self.symbols.len() * Symbol::size::<I>() + idx * Reloc::size()) as u32
	}

	pub fn strtab_offset(&self) -> u32 {
		(self.header.size::<I>() + crate::align_up(self.wilderness.len(), 1 << 3) + self.symbols.len() * Symbol::size::<I>() + self.relocs.len() * Reloc::size()) as u32
	}

	pub fn wilderness_append(&mut self, data: &[u8]) -> u32 {
		let idx = self.wilderness.len();
		self.wilderness.extend(data);
		(self.header.size::<I>() + idx) as u32
	}

	pub fn string(&mut self, string: &str) -> u32 {
		self.strtab.extend(string.as_bytes());
		self.strtab.push(0);

		(self.strtab.len() - string.len() - 1) as u32
	}

	pub fn complete_symtab(&mut self, idx: usize) {
		let new_symoff = self.symtab_offset();
		let new_stroff = self.strtab_offset();
		
		match self.header.commands.get_mut(idx) {
			Some(Command::SymTab(symtab)) => {
				symtab.symoff = new_symoff;
				symtab.nsyms = self.symbols.len() as u32;
				symtab.stroff = new_stroff;
				symtab.strsize = self.strtab.len() as u32; // FIXME: Does this need to be aligned?
			},
			_ => panic!("Not a symtab")
		}
	}

	pub fn build(&self) -> Vec<u8> {
		let mut data = Vec::new();
		self.header.encode::<I>(&mut data);
		
		data.extend(&self.wilderness);
		crate::align_up_vec(&mut data, 1 << 3);
		
		for sym in self.symbols.iter() { sym.encode::<I>(&mut data); }
		for reloc in self.relocs.iter() { reloc.encode::<I>(&mut data); }

		data.extend(&self.strtab);

		crate::align_up_vec(&mut data, 1 << 3);

		data
	}
}