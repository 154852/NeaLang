use crate::{EncodableInt, align_up_vec_offset, elf::*};
use crate::elf;

pub struct StrTab {
    data: Vec<u8>
}

impl StrTab {
    pub fn new() -> StrTab {
        StrTab {
            data: Vec::new()
        }
    }

    pub fn push(&mut self, string: &str) -> u64 {
        let idx = self.data.len();
        self.data.extend(string.as_bytes());
        self.data.push(0);

        idx as u64
    }

    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn get(self) -> Vec<u8> {
        self.data
    }
}

pub enum Symbol {
    /// name, vaddr, size
    Function(String, u64, u64),

    /// name, vaddr, size
    Object(String, u64, u64),

    /// name
    Relocatable(String)
}

impl Symbol {
    pub fn get_elf_symbol(&self, strtab: &mut StrTab) -> elf::Symbol {
        match self {
            Symbol::Function(name, vaddr, size) => elf::Symbol::new_function(strtab.push(name), *vaddr, *size, 1), // TODO: Don't hardcode this
            Symbol::Object(name, vaddr, size) => elf::Symbol::new_object(strtab.push(name), *vaddr, *size, 2), // TODO: Don't hardcode this
            Symbol::Relocatable(name) => elf::Symbol::new_relocatable(strtab.push(name))
        }
    }
}

pub struct StaticELF {
    /// Text section (and possibly segment). Will always be segment 0 and section 1.
    /// Will be read + exec
    text: Option<(u64, Vec<u8>)>,

    /// Writable data section (and possibly segment). Will always come after .text (if present)
    /// Will be read + write
    data: Option<(u64, Vec<u8>)>,

    /// Read only data section (and possibly segment). Will always come after .data (if present)
    /// Will be read
    rodata: Option<(u64, Vec<u8>)>,

    symbols: Vec<Symbol>,
    
    /// Relocations in the text section, if not null then file is relocatable. If empty then the .rela.text section is omitted
    text_relocations: Option<Vec<Rela>>,

    /// Relocations in the data section, if not null then file is relocatable. If empty then the .rela.data section is omitted
    data_relocations: Option<Vec<Rela>>,

    /// Relocations in the rodata section, if not null then file is relocatable. If empty then the .rela.rodata section is omitted
    rodata_relocations: Option<Vec<Rela>>
}

impl StaticELF {
    pub fn new() -> StaticELF {
        StaticELF {
            text: None,
            data: None,
            rodata: None,
            symbols: Vec::new(),
            text_relocations: None,
            data_relocations: None,
            rodata_relocations: None
        }
    }

    pub fn new_relocatable() -> StaticELF {
        StaticELF {
            text: None,
            data: None,
            rodata: None,
            symbols: Vec::new(),
            text_relocations: Some(Vec::new()),
            data_relocations: Some(Vec::new()),
            rodata_relocations: Some(Vec::new())
        }
    }

    pub fn push_symbol(&mut self, symbol: Symbol) -> usize {
        self.symbols.push(symbol);
        self.symbols.len() // not -1, as a null symbol will be pushed first
    }

    pub fn set_text(&mut self, vaddr: u64, text: Vec<u8>) {
        self.text = Some((vaddr, text));
    }

    pub fn set_data(&mut self, vaddr: u64, data: Vec<u8>) {
        self.data = Some((vaddr, data));
    }

    pub fn set_rodata(&mut self, vaddr: u64, rodata: Vec<u8>) {
        self.rodata = Some((vaddr, rodata));
    }

    pub fn relocatable(&self) -> bool {
        self.text_relocations.is_some() || self.data_relocations.is_some() || self.rodata_relocations.is_some()
    }

    pub fn push_text_relocation(&mut self, relocation: Rela) {
        self.text_relocations.as_mut().expect("Cannot push text relocation when the file is not relocatable").push(relocation);
    }

    pub fn push_data_relocation(&mut self, relocation: Rela) {
        self.data_relocations.as_mut().expect("Cannot push data relocation when the file is not relocatable").push(relocation);
    }

    pub fn push_rodata_relocation(&mut self, relocation: Rela) {
        self.rodata_relocations.as_mut().expect("Cannot push rodata relocation when the file is not relocatable").push(relocation);
    }

    // TODO: This is a bit ugly, maybe make this a layer on top of something else where it easier to add sections and their relocations etc?
    pub fn encode<I: EncodableInt>(&self, header: Header) -> (Vec<u8>, Vec<u8>) {
        let mut elf = ELF::new(header);
        let mut shstrtab = StrTab::new();
        let mut strtab = StrTab::new();

        let mut symbols = vec![elf::Symbol::new_null(strtab.push(""))];
        symbols.extend(self.symbols.iter().map(|x| x.get_elf_symbol(&mut strtab)));

        elf.push_section_header(SectionHeader::new_null(shstrtab.push("")));

        let text_idx = match &self.text {
            Some((vaddr, text)) => {
                let text_ph_idx = if !self.relocatable() {
                    let idx = elf.push_program_header(ProgramHeader::new_load(0, *vaddr, *vaddr, text.len() as u64));
                    elf.program_header_mut(idx).set_flags(true, false, true);
                    Some(idx)
                } else {
                    None
                };

                let text_sh_idx = elf.push_section_header(SectionHeader::new_progbits(shstrtab.push(".text"), *vaddr, 0, text.len() as u64));
                elf.section_header_mut(text_sh_idx).set_flags(false, true, true);

                Some((text_ph_idx, text_sh_idx))
            },
            None => None
        };

        let data_idx = match &self.data {
            Some((vaddr, data)) => {
                let data_ph_idx = if !self.relocatable() {
                    Some(elf.push_program_header(ProgramHeader::new_load(0, *vaddr, *vaddr, data.len() as u64)))
                } else {
                    None
                };
                let data_sh_idx = elf.push_section_header(SectionHeader::new_progbits(shstrtab.push(".data.rel.local"), *vaddr, 0, data.len() as u64));
                elf.section_header_mut(data_sh_idx).set_flags(true, true, false);

                Some((data_ph_idx, data_sh_idx))
            },
            None => None
        };

        let rodata_idx = match &self.rodata {
            Some((vaddr, rodata)) => {
                let rodata_ph_idx = if !self.relocatable() {
                    Some(elf.push_program_header(ProgramHeader::new_load(0, *vaddr, *vaddr, rodata.len() as u64)))
                } else {
                    None
                };
                let rodata_sh_idx = elf.push_section_header(SectionHeader::new_progbits(shstrtab.push(".rodata"), *vaddr, 0, rodata.len() as u64));
                elf.section_header_mut(rodata_sh_idx).set_flags(false, true, false);

                Some((rodata_ph_idx, rodata_sh_idx))
            },
            None => None
        };

        let strtab_idx = shstrtab.push(".strtab");
        let strtab_idx = elf.push_section_header(SectionHeader::new_strtab(strtab_idx, 0, strtab.len()));

        let mut locals = 0;
        for symbol in symbols.iter() {
            if symbol.is_local() { locals += 1 };
        }

        let symtab_idx = shstrtab.push(".symtab");
        let symtab_idx = elf.push_section_header(SectionHeader::new_symtab::<I>(symtab_idx, 0, symbols.len() as u64, locals, strtab_idx as u32));

        let text_rela = if let Some(relocs) = &self.text_relocations {
            if relocs.len() == 0 {
                None
            } else {
                let rela_text_idx = shstrtab.push(".rela.text");
                Some(elf.push_section_header(SectionHeader::new_relas::<I>(rela_text_idx, 0, relocs.len() as u64, symtab_idx as u32, text_idx.expect("Cannot do text relocation without text section").1 as u32)))
            }
        } else {
            None
        };

        let data_rela = if let Some(relocs) = &self.data_relocations {
            if relocs.len() == 0 {
                None
            } else {
                // let rela_data_idx = shstrtab.push(".rela.data");
                let rela_data_idx = shstrtab.push(".rela.data.rel.local");
                Some(elf.push_section_header(SectionHeader::new_relas::<I>(rela_data_idx, 0, relocs.len() as u64, symtab_idx as u32, data_idx.expect("Cannot do data relocation without data section").1 as u32)))
            }
        } else {
            None
        };

        let rodata_rela = if let Some(relocs) = &self.rodata_relocations {
            if relocs.len() == 0 {
                None
            } else {
                let rela_rodata_idx = shstrtab.push(".rela.rodata");
                Some(elf.push_section_header(SectionHeader::new_relas::<I>(rela_rodata_idx, 0, relocs.len() as u64, symtab_idx as u32, rodata_idx.expect("Cannot do rodata relocation without rodata section").1 as u32)))
            }
        } else {
            None
        };

        let shstrab_idx = shstrtab.push(".shstrtab");
        let shstrab_idx = elf.push_section_header(SectionHeader::new_strtab(shstrab_idx, 0, shstrtab.len()));
        elf.set_section_strtab(shstrab_idx as u64);

        let mut raw_data = Vec::<u8>::new();

        let size = elf.size::<I>() as u64;

        if let Some((ph_idx, sh_idx)) = text_idx {
            align_up_vec_offset(&mut raw_data, size as usize, 1 << 12);
            if !self.relocatable() { elf.program_header_mut(ph_idx.unwrap()).set_offset(raw_data.len() as u64 + size); }
            elf.section_header_mut(sh_idx).set_offset(raw_data.len() as u64 + size);
            raw_data.extend(&self.text.as_ref().unwrap().1);
        }

        if let Some((ph_idx, sh_idx)) = data_idx {
            align_up_vec_offset(&mut raw_data, size as usize, 1 << 12);
            if !self.relocatable() { elf.program_header_mut(ph_idx.unwrap()).set_offset(raw_data.len() as u64 + size); }
            elf.section_header_mut(sh_idx).set_offset(raw_data.len() as u64 + size);
            raw_data.extend(&self.data.as_ref().unwrap().1);
        }

        if let Some((ph_idx, sh_idx)) = rodata_idx {
            align_up_vec_offset(&mut raw_data, size as usize, 1 << 12);
            if !self.relocatable() { elf.program_header_mut(ph_idx.unwrap()).set_offset(raw_data.len() as u64 + size); }
            elf.section_header_mut(sh_idx).set_offset(raw_data.len() as u64 + size);
            raw_data.extend(&self.rodata.as_ref().unwrap().1);
        }

        elf.section_header_mut(shstrab_idx).set_offset(raw_data.len() as u64 + size);
        raw_data.extend(&shstrtab.get());

        elf.section_header_mut(strtab_idx).set_offset(raw_data.len() as u64 + size);
        raw_data.extend(&strtab.get());

        elf.section_header_mut(symtab_idx).set_offset(raw_data.len() as u64 + size);
        for symbol in symbols.iter() {
            symbol.encode::<I>(&mut raw_data);
        }

        if let Some(rela_text_idx) = text_rela {
            elf.section_header_mut(rela_text_idx).set_offset(raw_data.len() as u64 + size);
            for rela in self.text_relocations.as_ref().unwrap().iter() {
                rela.encode::<I>(&mut raw_data);
            }
        }

        if let Some(rela_data_idx) = data_rela {
            elf.section_header_mut(rela_data_idx).set_offset(raw_data.len() as u64 + size);
            for rela in self.data_relocations.as_ref().unwrap().iter() {
                rela.encode::<I>(&mut raw_data);
            }
        }

        if let Some(rela_rodata_idx) = rodata_rela {
            elf.section_header_mut(rela_rodata_idx).set_offset(raw_data.len() as u64 + size);
            for rela in self.rodata_relocations.as_ref().unwrap().iter() {
                rela.encode::<I>(&mut raw_data);
            }
        }

        (elf.encode::<I>(), raw_data)
    }
}