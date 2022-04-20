use std::io::Write;
use ofile::{elf, elfbuilder};

const TEXT_BASE: u64 = 0x401000;

fn mangle_func_name(func: &ir::Function) -> String {
    if let Some(ctr) = func.method_of() {
        format!("{}.{}", ctr.name(), func.name())
    } else {
        func.name().to_string()
    }
}

pub fn encode(unit: &ir::TranslationUnit, path: &str, relocatable: bool) -> Result<(), String> {
    let mut elf = if relocatable {
        elfbuilder::StaticELF::new_relocatable()
    } else {
        elfbuilder::StaticELF::new()
    };

    let mut x86_encoding = x86::EncodeContext::new();
    let ctx = ir2x86::TranslationContext::new(x86::Mode::X8664);

    let text_base = if relocatable { 0 } else { TEXT_BASE };
    let mut entry = None;

    let mut gid_allocator = ir2x86::GlobalIDAllocator::new(unit);

    let data_base_symbol = elf.push_symbol(elfbuilder::Symbol::Section(2)); // data is section 2, TODO: This should not be hard coded

    for (i, func) in unit.functions().iter().enumerate() {
        let gid = gid_allocator.global_id_of_function(ir::FunctionIndex::new(i));
        
        if func.is_extern() {
            if !relocatable {
                return Err(format!("Cannot import function '{}' with a statically compiled binary", func.name()));
            }
            gid_allocator.push_global_symbol_mapping(gid, elf.push_symbol(elfbuilder::Symbol::Relocatable(mangle_func_name(func))), 0);
        } else {
            let mut ins = ctx.translate_function(&func, unit);
            x86::opt::pass_zero(&mut ins);
            
            let (addr, length) = x86_encoding.append_function(&ins);
            if func.is_entry() {
                entry = Some(text_base + addr as u64);
            }
            gid_allocator.push_global_symbol_mapping(gid, elf.push_symbol(elfbuilder::Symbol::Function(if func.is_entry() {
                "main".to_owned()
            } else {
                mangle_func_name(func)
            }, text_base + addr as u64, length as u64)), 0);
        }
    }

    let data_base = if relocatable { 0 } else { ofile::align_up(text_base + x86_encoding.len() as u64, 1 << 12) };

    let mut relocs = Vec::new();
    let mut data = Vec::new();
    for (i, global) in unit.globals().iter().enumerate() {
        let gid = gid_allocator.global_id_of_global(ir::GlobalIndex::new(i));

        if let Some(name) = global.name() {
            let pushed = ctx.translate_global(global, unit, &gid_allocator, &mut relocs, data.len(), 0);
            gid_allocator.push_global_symbol_mapping(gid, elf.push_symbol(elfbuilder::Symbol::Object(
                name.to_string(),
                data_base + data.len() as u64,
                pushed.len() as u64
            )), 0);
            data.extend(pushed);
        } else {
            let pushed = ctx.translate_global(global, unit, &gid_allocator, &mut relocs, data.len(), 0);
            gid_allocator.push_global_symbol_mapping(gid, data_base_symbol, data.len() as i64);
            data.extend(pushed);
        }
    }

    // Data relocations
    if !relocatable {
        for reloc in relocs {
            match reloc.kind() {
                x86::RelocationType::AbsoluteGlobalSymbol(id) => {
                    let (symbol, addend) = gid_allocator.symbol_for_global_id(*id).expect("Invalid relocation symbol");
                    let addr = (match elf.get_symbol(symbol).expect("Invalid relocation") {
                        elfbuilder::Symbol::Function(_, vaddr, _) => *vaddr,
                        elfbuilder::Symbol::Object(_, vaddr, _) => *vaddr,
                        elfbuilder::Symbol::Relocatable(name) => return Err(format!("Could not statically link due to external reference '{}'", name)),
                        elfbuilder::Symbol::Section(idx) => match idx {
                            1 => text_base,
                            2 => data_base,
                            _ => panic!("Invalid section")
                        },
                    } as i64 + reloc.addend() + addend) as u64;
    
                    data[reloc.offset()..reloc.offset() + 8].copy_from_slice(
                        &addr.to_le_bytes()
                    );
                },
                _ => panic!("Cannot relocate non-global symbol in .text")
            }
        }
    } else {
        for reloc in relocs {
            match reloc.kind() {
                x86::RelocationType::AbsoluteGlobalSymbol(id) => {
                    let (symbol, addend) = gid_allocator.symbol_for_global_id(*id).expect("Invalid relocation symbol");

                    elf.push_data_relocation(elf::Rela::new(
                        reloc.offset() as u64,
                        symbol as u64,
                        elf::RelocationType::X86646464,
                        reloc.addend() + addend
                    ));
                },
                _ => panic!("Cannot relocate non-global symbol in .data")
            }
        }
    }

    let (mut text, relocs) = x86_encoding.take();

    // Text relocations
    if !relocatable {
        for reloc in relocs {
            match reloc.kind() {
                x86::RelocationType::RelativeGlobalSymbol(id) => {
                    let (symbol, addend) = gid_allocator.symbol_for_global_id(*id).expect("Invalid relocation symbol");
                    let addr = (match elf.get_symbol(symbol).expect("Invalid relocation") {
                        elfbuilder::Symbol::Function(_, vaddr, _) => *vaddr,
                        elfbuilder::Symbol::Object(_, vaddr, _) => *vaddr,
                        elfbuilder::Symbol::Relocatable(name) => return Err(format!("Could not statically link due to external reference '{}'", name)),
                        elfbuilder::Symbol::Section(idx) => match idx {
                            1 => text_base,
                            2 => data_base,
                            _ => panic!("Invalid section")
                        },
                    } as i64 + reloc.addend() + addend) as u32;
    
                    text[reloc.offset()..reloc.offset() + 4].copy_from_slice(
                        &(addr - reloc.offset() as u32 - text_base as u32).to_le_bytes()
                    );
                },
                _ => panic!("Cannot relocate non-global symbol in .text")
            }
        }
    } else {
        for reloc in relocs {
            match reloc.kind() {
                x86::RelocationType::RelativeGlobalSymbol(id) => {
                    let (symbol, addend) = gid_allocator.symbol_for_global_id(*id).expect("Invalid relocation symbol");
    
                    elf.push_text_relocation(elf::Rela::new(
                        reloc.offset() as u64,
                        symbol as u64,
                        elf::RelocationType::X8664Plt32,
                        reloc.addend() + addend
                    ));
                },
                _ => panic!("Cannot relocate non-global symbol in .text")
            }
        }
    }

    elf.set_text(text_base, text);
    elf.set_data(data_base, data);

    let header = elf::Header::new_with_entry(
        elf::ABI::SysV,
        if relocatable { elf::ObjectFileType::Relocatable } else { elf::ObjectFileType::Executable },
        elf::Machine::X8664,
        if let Some(entry) = entry {
            entry
        } else if relocatable {
            0
        } else {
            return Err("No entry point specified".to_string());
        }
    );
    let (header, body) = elf.encode::<ofile::LittleEndian64>(header);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path).expect("Could not open");
    
    file.write(&header).expect("Could not write");
    file.write(&body).expect("Could not write");

    drop(file);

    Ok(())
}