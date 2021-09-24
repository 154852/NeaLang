use std::collections::HashMap;

pub type LocalSymbolID = usize;

pub struct UnfilledLocalSymbol {
    symbol: LocalSymbolID,
    offset: usize,
    addend: i64
}

impl UnfilledLocalSymbol {
    pub fn new(symbol: LocalSymbolID, offset: usize, addend: i64) -> UnfilledLocalSymbol {
        UnfilledLocalSymbol {
            symbol, offset, addend
        }
    }

    pub fn fill(data: &mut Vec<u8>, locations: &HashMap<LocalSymbolID, usize>, unfilled: &Vec<UnfilledLocalSymbol>) {
        for unfilled_location in unfilled.iter() {
            let addr = locations.get(&unfilled_location.symbol).expect("Local symbol not definied");
            data[unfilled_location.offset..unfilled_location.offset+4].copy_from_slice(
                &(((*addr as i64 - unfilled_location.offset as i64) + unfilled_location.addend) as u32).to_le_bytes()
            );
        }
    }
}