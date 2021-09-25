use std::collections::HashMap;

pub type LocalSymbolID = usize;

pub enum RelocationType {
    LocalFunctionSymbol(LocalSymbolID)
}

pub struct Relocation {
    kind: RelocationType,
    offset: usize,
    addend: i64
}

impl Relocation {
    pub fn new_local_branch(symbol: LocalSymbolID, offset: usize, addend: i64) -> Relocation {
        Relocation {
            kind: RelocationType::LocalFunctionSymbol(symbol),
            offset, addend
        }
    }

    pub fn fill(data: &mut Vec<u8>, local_symbols: &HashMap<LocalSymbolID, usize>, relocations: &Vec<Relocation>) {
        for relocation in relocations.iter() {
            match &relocation.kind {
                RelocationType::LocalFunctionSymbol(symbol) => {
                    let addr = local_symbols.get(symbol).expect("Local symbol not definied");
                    data[relocation.offset..relocation.offset+4].copy_from_slice(
                        &(((*addr as i64 - relocation.offset as i64) + relocation.addend) as u32).to_le_bytes()
                    );
                }
            }
        }
    }
}