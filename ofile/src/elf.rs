use crate::{EncodableInt, Endian};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum ProgramHeaderType {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    Shlib,
    Phdr,
    Stack
}

impl ProgramHeaderType {
    pub fn encode<I: EncodableInt>(&self) -> [u8; 4] {
        I::u32(match self {
            ProgramHeaderType::Null => 0,
            ProgramHeaderType::Load => 1,
            ProgramHeaderType::Dynamic => 2,
            ProgramHeaderType::Interp => 3,
            ProgramHeaderType::Note => 4,
            ProgramHeaderType::Shlib => 5,
            ProgramHeaderType::Phdr => 6,
            ProgramHeaderType::Stack => 0x6474e551,
        })
    }
}

pub struct ProgramHeader {
    header_type: ProgramHeaderType,
    readable: bool, writable: bool, executable: bool,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    file_size: u64,
    mem_size: u64,
}

impl ProgramHeader {
    pub fn new(header_type: ProgramHeaderType, readable: bool, writable: bool, executable: bool, offset: u64, vaddr: u64, paddr: u64, file_size: u64, mem_size: u64) -> ProgramHeader {
        ProgramHeader {
            header_type, readable, writable, executable, offset, vaddr, paddr, file_size, mem_size,
        }
    }

    pub fn new_load(offset: u64, vaddr: u64, paddr: u64, file_size: u64) -> ProgramHeader {
        ProgramHeader {
            header_type: ProgramHeaderType::Load,
            readable: true, writable: false, executable: false,
            offset,
            vaddr,
            paddr,
            file_size,
            mem_size: file_size,
        }
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    pub fn set_flags(&mut self, readable: bool, writable: bool, executable: bool) {
        self.readable = readable;
        self.writable = writable;
        self.executable = executable;
    }

    pub fn size<I: EncodableInt>() -> usize {
        if I::size() == 4 { 32 } else { 56 }
    }

    pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
        data.reserve(ProgramHeader::size::<I>());

        data.extend(&self.header_type.encode::<I>());

        let mut flags: u32 = 0;
        if self.executable { flags |= 1 << 0; }
        if self.writable { flags |= 1 << 1; }
        if self.readable { flags |= 1 << 2; }

        if I::size() == 8 {
            data.extend(&I::u32(flags));
        }

        I::encode(self.offset, data);
        I::encode(self.vaddr, data);
        I::encode(self.paddr, data);

        I::encode(self.file_size, data);
        I::encode(self.mem_size, data);

        if I::size() == 4 {
            data.extend(&I::u32(flags));
        }

        I::encode(match self.header_type {
            ProgramHeaderType::Load => 1 << 12,
            ProgramHeaderType::Stack => 1 << 3,
            _ => 1
        }, data);
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum SectionHeaderType {
    Null,
    ProgBits,
    SymTab,
    StrTab,
    Rela,
    Hash,
    Dynamic,
    Note,
    NoBits,
    Rel,
    ShLib,
    DynSym
}

impl SectionHeaderType {
    pub fn encode<I: EncodableInt>(&self) -> [u8; 4] {
        I::u32(match self {
            SectionHeaderType::Null => 0,
            SectionHeaderType::ProgBits => 1,
            SectionHeaderType::SymTab => 2,
            SectionHeaderType::StrTab => 3,
            SectionHeaderType::Rela => 4,
            SectionHeaderType::Hash => 5,
            SectionHeaderType::Dynamic => 6,
            SectionHeaderType::Note => 7,
            SectionHeaderType::NoBits => 8,
            SectionHeaderType::Rel => 9,
            SectionHeaderType::ShLib => 10,
            SectionHeaderType::DynSym => 11,
        })
    }
}

pub struct SectionHeader {
    header_type: SectionHeaderType,
    name_index: u64,
    writable: bool, allocated: bool, executable: bool,
    vaddr: u64,
    file_offset: u64,
    size: u64,
    link: u32, info: u32,
    addralign: u64, entsize: u64
}

impl SectionHeader {
    pub fn new(header_type: SectionHeaderType, name_index: u64, writable: bool, allocated: bool, executable: bool, vaddr: u64, file_offset: u64, size: u64, link: u32, info: u32, addralign: u64, entsize: u64) -> SectionHeader {
        SectionHeader {
            header_type, name_index, writable, allocated, executable, vaddr, file_offset, size, link, info, addralign, entsize,
        }
    }

    pub fn new_null(name_index: u64) -> SectionHeader {
        SectionHeader {
            header_type: SectionHeaderType::Null,
            name_index,
            writable: false, allocated: false, executable: false,
            vaddr: 0, file_offset: 0, size: 0,
            link: 0, info: 0,
            addralign: 0,
            entsize: 0,
        }
    }

    pub fn new_progbits(name_index: u64, vaddr: u64, file_offset: u64, size: u64) -> SectionHeader {
        SectionHeader {
            header_type: SectionHeaderType::ProgBits,
            name_index,
            writable: false, allocated: true, executable: false,
            vaddr, file_offset, size,
            link: 0, info: 0,
            addralign: 1,
            entsize: 0,
        }
    }

    pub fn new_strtab(name_index: u64, file_offset: u64, size: u64) -> SectionHeader {
        SectionHeader {
            header_type: SectionHeaderType::StrTab,
            name_index,
            writable: false, allocated: true, executable: false,
            vaddr: 0, file_offset, size,
            link: 0, info: 0,
            addralign: 1,
            entsize: 0,
        }
    }

    pub fn new_symtab<I: EncodableInt>(name_index: u64, file_offset: u64, entries: u64, local_count: u32, strtab_idx: u32) -> SectionHeader {
        SectionHeader {
            name_index,
            header_type: SectionHeaderType::SymTab,
            writable: false, allocated: true, executable: false,
            vaddr: 0, file_offset,
            size: entries * Symbol::size::<I>() as u64,
            link: strtab_idx, info: local_count,
            addralign: 1,
            entsize: Symbol::size::<I>() as u64
        }
    }

    pub fn new_relas<I: EncodableInt>(name_index: u64, file_offset: u64, entries: u64, symtab: u32, section: u32) -> SectionHeader {
        SectionHeader {
            name_index,
            header_type: SectionHeaderType::Rela,
            writable: false, allocated: true, executable: false,
            vaddr: 0, file_offset,
            size: entries * Rela::size::<I>() as u64,
            link: symtab, info: section,
            addralign: 1,
            entsize: Rela::size::<I>() as u64
        }
    }

    pub fn set_flags(&mut self, writable: bool, allocated: bool, executable: bool) {
        self.writable = writable;
        self.allocated = allocated;
        self.executable = executable;
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.file_offset = offset;
    }

    pub fn size<I: EncodableInt>() -> usize {
        if I::size() == 4 { 40 } else { 64 }
    }

    pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
        data.reserve(SectionHeader::size::<I>());

        data.extend(&I::u32(self.name_index as u32));
        data.extend(&self.header_type.encode::<I>());

        let mut flags = 0u64;
        if self.writable { flags |= 1 << 0; }
        if self.allocated { flags |= 1 << 1; }
        if self.executable { flags |= 1 << 2; }
        I::encode(flags, data);

        I::encode(self.vaddr, data);
        I::encode(self.file_offset, data);
        I::encode(self.size, data);

        data.extend(&I::u32(self.link));
        data.extend(&I::u32(self.info));

        I::encode(self.addralign, data);
        I::encode(self.entsize, data);
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum ABI {
    SysV,
    Hpux,
    NetBSD,
    Linux,
    Solaris,
    Irix,
    FreeBSD,
    Tru64,
    Arm,
    Standalone
}

impl ABI {
    pub fn encode(&self) -> u8 {
        match self {
            ABI::SysV => 0,
            ABI::Hpux => 1,
            ABI::NetBSD => 2,
            ABI::Linux => 3,
            ABI::Solaris => 6,
            ABI::Irix => 8,
            ABI::FreeBSD => 9,
            ABI::Tru64 => 10,
            ABI::Arm => 97,
            ABI::Standalone => 255
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum ObjectFileType {
    Relocatable,
    Executable,
    Shared,
    Core
}

impl ObjectFileType {
    pub fn encode<I: EncodableInt>(&self) -> [u8; 2] {
        I::u16(match self {
            ObjectFileType::Relocatable => 1,
            ObjectFileType::Executable => 2,
            ObjectFileType::Shared => 3,
            ObjectFileType::Core => 4,
        })
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum Machine {
    M32,
    Sparc,
    I386,
    Motorola68K,
    Motorola88K,
    I860,
    Mips,
    Parisc,
    Sparc32Plus,
    Ppc,
    Ppc64,
    S390,
    Arm,
    Sh,
    SparcV9,
    Ia64,
    X8664,
    Vax
}

impl Machine {
    pub fn encode<I: EncodableInt>(&self) -> [u8; 2] {
        I::u16(match self {
            Machine::M32 => 1,
            Machine::Sparc => 2,
            Machine::I386 => 3,
            Machine::Motorola68K => 4,
            Machine::Motorola88K => 5,
            Machine::I860 => 7,
            Machine::Mips => 8,
            Machine::Parisc => 15,
            Machine::Sparc32Plus => 18,
            Machine::Ppc => 20,
            Machine::Ppc64 => 21,
            Machine::S390 => 22,
            Machine::Arm => 40,
            Machine::Sh => 42,
            Machine::SparcV9 => 43,
            Machine::Ia64 => 50,
            Machine::X8664 => 62,
            Machine::Vax => 75,
        })
    }
}

pub struct Header {
    abi: ABI,
    file_type: ObjectFileType,
    machine: Machine,
    entry_point: u64
}

impl Header {
    pub fn new(abi: ABI, file_type: ObjectFileType, machine: Machine) -> Header {
        Header {
            abi, file_type, machine,
            entry_point: 0
        }
    }

    pub fn new_with_entry(abi: ABI, file_type: ObjectFileType, machine: Machine, entry_point: u64) -> Header {
        Header {
            abi, file_type, machine,
            entry_point
        }
    }

    pub fn set_entry_point(&mut self, entry_point: u64) {
        self.entry_point = entry_point;
    }

    pub fn size<I: EncodableInt>() -> usize {
        if I::size() == 4 { 52 } else { 64 }
    }

    pub fn encode_beginning<I: EncodableInt>(&self, data: &mut Vec<u8>) {
        data.reserve(Header::size::<I>());

        data.extend(&[0x7f, 'E' as u8, 'L' as u8, 'F' as u8]);
        data.push(if I::size() == 4 { 1 } else { 2 }); // ELFCLASS*
        data.push(if I::endian() == Endian::Little { 1 } else { 2 }); // ELFDATA*
        data.push(1); // EV_CURRENT
        data.push(self.abi.encode());
        data.push(0); // ABI Version
        data.extend(&[ 0, 0, 0, 0, 0, 0, 0 ]);

        data.extend(&self.file_type.encode::<I>());
        data.extend(&self.machine.encode::<I>());
        data.extend(&I::u32(1));

        I::encode(self.entry_point, data);
    }
}

pub struct ELF {
    header: Header,
    program_headers: Vec<ProgramHeader>,
    section_headers: Vec<SectionHeader>,
    section_strtab: u64
}

impl ELF {
    pub fn new(header: Header) -> ELF {
        ELF {
            header,
            program_headers: Vec::new(),
            section_headers: Vec::new(),
            section_strtab: 0
        }
    }

    pub fn push_program_header(&mut self, header: ProgramHeader) -> usize {
        self.program_headers.push(header);
        self.program_headers.len() - 1
    }

    pub fn program_header_mut(&mut self, idx: usize) -> &mut ProgramHeader {
        &mut self.program_headers[idx]
    }

    pub fn push_section_header(&mut self, header: SectionHeader) -> usize {
        self.section_headers.push(header);
        self.section_headers.len() - 1
    }

    pub fn section_header_mut(&mut self, idx: usize) -> &mut SectionHeader {
        &mut self.section_headers[idx]
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn set_section_strtab(&mut self, idx: u64) {
        self.section_strtab = idx;
    }

    pub fn size<I: EncodableInt>(&self) -> usize {
        Header::size::<I>() + (self.program_headers.len() * ProgramHeader::size::<I>()) + (self.section_headers.len() * SectionHeader::size::<I>())
    }

    pub fn encode<I: EncodableInt>(&self) -> Vec<u8> {
        let mut data = Vec::new();

        self.header.encode_beginning::<I>(&mut data);

        // Program Header Table starts at sizeof(header)
        if self.program_headers.len() == 0 {
            I::encode(0, &mut data);
        } else {
            I::encode(Header::size::<I>() as u64, &mut data);
        }
        // Section Header Table starts at sizeof(header) + sizeof(programheader) * programheadercount
        I::encode(if self.section_headers.len() == 0 { 0 } else { Header::size::<I>() as u64 + (ProgramHeader::size::<I>() as u64 * self.program_headers.len() as u64) }, &mut data);

        data.extend(&I::u32(0)); // Flags

        data.extend(&I::u16(Header::size::<I>() as u16));

        data.extend(&I::u16(ProgramHeader::size::<I>() as u16));
        data.extend(&I::u16(self.program_headers.len() as u16));

        data.extend(&I::u16(SectionHeader::size::<I>() as u16));
        data.extend(&I::u16(self.section_headers.len() as u16));

        data.extend(&I::u16(self.section_strtab as u16));

        assert_eq!(data.len(), Header::size::<I>() as usize);

        for header in self.program_headers.iter() {
            header.encode::<I>(&mut data);
        }

        for header in self.section_headers.iter() {
            header.encode::<I>(&mut data);
        }

        assert_eq!(data.len(), self.size::<I>());

        data
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum SymbolType {
    NoType,
    Object,
    Func,
    Section,
    File,
}

impl SymbolType {
    pub fn encode(&self) -> u8 {
        match self {
            SymbolType::NoType => 0,
            SymbolType::Object => 1,
            SymbolType::Func => 2,
            SymbolType::Section => 3,
            SymbolType::File => 4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum SymbolBind {
    Local,
    Global,
    Weak
}

impl SymbolBind {
    pub fn encode(&self) -> u8 {
        match self {
            SymbolBind::Local => 0,
            SymbolBind::Global => 1,
            SymbolBind::Weak => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum SymbolVisibility {
    Default,
    Internal,
    Hidden,
    Protected
}

impl SymbolVisibility {
    pub fn encode(&self) -> u8 {
        match self {
            SymbolVisibility::Default => 0,
            SymbolVisibility::Internal => 1,
            SymbolVisibility::Hidden => 2,
            SymbolVisibility::Protected => 3,
        }
    }
}

pub struct Symbol {
    name_index: u64,
    value: u64,
    size: u64,
    symbol_type: SymbolType,
    symbol_bind: SymbolBind,
    visibility: SymbolVisibility,
    section: u16
}

impl Symbol {
    pub fn new(name_index: u64, value: u64, size: u64, symbol_type: SymbolType, symbol_bind: SymbolBind, visibility: SymbolVisibility, section: u16) -> Symbol {
        Symbol {
            name_index, value, size, symbol_type, symbol_bind, visibility, section
        }
    }

    pub fn new_null(name_index: u64) -> Symbol {
        Symbol {
            name_index,
            value: 0,
            size: 0,
            symbol_type: SymbolType::NoType,
            symbol_bind: SymbolBind::Local,
            visibility: SymbolVisibility::Default,
            section: 0
        }
    }

    pub fn new_relocatable(name_index: u64) -> Symbol {
        Symbol {
            name_index,
            value: 0,
            size: 0,
            symbol_type: SymbolType::NoType,
            symbol_bind: SymbolBind::Global,
            visibility: SymbolVisibility::Default,
            section: 0
        }
    }

    pub fn new_section(name_index: u64, section_index: u16) -> Symbol {
        Symbol {
            name_index,
            value: 0,
            size: 0,
            symbol_type: SymbolType::Section,
            symbol_bind: SymbolBind::Global,
            visibility: SymbolVisibility::Default,
            section: section_index
        }
    }

    pub fn new_function(name_index: u64, vaddr: u64, size: u64, section: u16) -> Symbol {
        Symbol {
            name_index,
            value: vaddr,
            size,
            section,
            symbol_type: SymbolType::Func,
            symbol_bind: SymbolBind::Global,
            visibility: SymbolVisibility::Default
        }
    }

    pub fn new_object(name_index: u64, vaddr: u64, size: u64, section: u16) -> Symbol {
        Symbol {
            name_index,
            value: vaddr,
            size,
            section,
            symbol_type: SymbolType::Object,
            symbol_bind: SymbolBind::Global,
            visibility: SymbolVisibility::Default
        }
    }

    pub fn is_local(&self) -> bool {
        self.symbol_bind == SymbolBind::Local
    }

    pub fn set_name_index(&mut self, index: u64) {
        self.name_index = index;
    }

    pub fn set_section_index(&mut self, index: u16) {
        self.section = index;
    }

    pub fn binding_and_type(symbol_type: &SymbolType, bind: &SymbolBind) -> u8 {
        (bind.encode() << 4) + (symbol_type.encode() & 0xf)
    }

    pub fn size<I: EncodableInt>() -> usize {
        if I::size() == 4 { 16 } else { 24 }
    }

    pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
        data.reserve(Symbol::size::<I>());

        data.extend(&I::u32(self.name_index as u32));

        if I::size() == 4 {
            I::encode(self.value, data);
            I::encode(self.size, data);
            data.push(Symbol::binding_and_type(&self.symbol_type, &self.symbol_bind));
            data.push(self.visibility.encode());
            data.extend(&I::u16(self.section));
        } else {
            data.push(Symbol::binding_and_type(&self.symbol_type, &self.symbol_bind));
            data.push(self.visibility.encode());
            data.extend(&I::u16(self.section));
            I::encode(self.value, data);
            I::encode(self.size, data);
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum RelocationType {
    X8664Pc32,
    X8664Plt32,
    X86646464
}

impl RelocationType {
    pub fn encode(&self) -> u8 {
        match self {
            RelocationType::X8664Pc32 => 2,
            RelocationType::X8664Plt32 => 4,
            RelocationType::X86646464 => 1,
        }
    }
}

pub struct Rela {
    offset: u64,
    symbol: u64,
    rel_type: RelocationType,
    addend: i64
}

impl Rela {
    pub fn new(offset: u64, symbol: u64, rel_type: RelocationType, addend: i64) -> Rela {
        Rela {
            offset, symbol, rel_type, addend
        }
    }

    pub fn size<I: EncodableInt>() -> usize {
        if I::size() == 4 { 12 } else { 24 }
    }

    pub fn info<I: EncodableInt>(symbol: u64, rel_type: RelocationType) -> u64 {
        if I::size() == 4 {
            (symbol << 8) | rel_type.encode() as u64
        } else {
            (symbol << 32) | rel_type.encode() as u64
        }
    }

    pub fn encode<I: EncodableInt>(&self, data: &mut Vec<u8>) {
        I::encode(self.offset, data);
        I::encode(Rela::info::<I>(self.symbol, self.rel_type), data);
        I::encode(self.addend as u64, data);
    }
}