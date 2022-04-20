use std::collections::HashMap;

use crate::{Ins, RelocationType, InsRelocMode, InsReloc};

pub struct EncodeContext {
    raw: Vec<u8>,
    relocations: Vec<(InsReloc, usize)>,
}

impl EncodeContext {
    pub fn new() -> EncodeContext {
        EncodeContext {
            raw: Vec::new(),
            relocations: Vec::new(),
        }
    }

    pub fn append_function(&mut self, code: &Vec<Ins>) -> (usize, usize) {
        let addr = self.raw.len();

        let mut local_symbols = HashMap::new();
        let mut new_relocations = Vec::new();
        for ins in code {
            match ins {
                Ins::LocalSymbol(id) => {
                    local_symbols.insert(*id, self.raw.len());
                },
                _ => {
                    let res = ins.encode();
                    self.raw.extend(res.get().to_le_bytes());
                    if let Some(reloc) = res.get_reloc() {
                        new_relocations.push((reloc, self.raw.len() - 4));
                    }
                }
            }
        }

        for reloc in new_relocations {
            if let RelocationType::LocalFunctionSymbol(symbol) = reloc.0.symbol() {
                let addr = *local_symbols.get(symbol).expect("Local symbol not definied");
                match reloc.0.mode() {
                    InsRelocMode::Branch26 => {
                        let mut curr = u32::from_le_bytes(self.raw[reloc.1..reloc.1 + 4].try_into().unwrap());
                        curr |= (((addr as i32 - reloc.1 as i32) as u32) >> 2) & ((1<<27) - 1);
                        self.raw[reloc.1..reloc.1 + 4].copy_from_slice(&curr.to_le_bytes());
                    },
                    InsRelocMode::Branch19Shift5 => {
                        let mut curr = u32::from_le_bytes(self.raw[reloc.1..reloc.1 + 4].try_into().unwrap());
                        let val = (((addr as i32 - reloc.1 as i32) as u32) >> 2) & ((1<<21) - 1);
                        curr |= val << 5;
                        self.raw[reloc.1..reloc.1 + 4].copy_from_slice(&curr.to_le_bytes());
                    },
                    InsRelocMode::Page21 => unreachable!(),
                    InsRelocMode::PageOff12 => unreachable!(),
                }
            } else {
                self.relocations.push(reloc);
            }
        }

        (addr, self.raw.len() - addr)
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn take(self) -> (Vec<u8>, Vec<(InsReloc, usize)>) {
        (self.raw, self.relocations)
    }
}