mod registerify;
mod ins;
mod unit;

pub use unit::*;

struct LocalSymbolStack {
    next: usize
}

impl LocalSymbolStack {
    fn new() -> LocalSymbolStack {
        LocalSymbolStack {
            next: 1
        }
    }

    fn root(&self) -> x86::LocalSymbolID {
        0
    }
}