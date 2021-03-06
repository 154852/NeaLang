#![allow(dead_code)]

mod reg;
mod encode;
mod ins;
mod reloc;
pub mod opt;
mod ctx;

#[cfg(test)]
mod tests;

pub use reg::*;
pub use encode::*;
pub use ins::*;
pub use reloc::*;
pub use ctx::*;

#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum Mode {
    X86,
    X8664
}

impl Mode {
    pub fn ptr_size(&self) -> usize {
        match self {
            Mode::X86 => 4,
            Mode::X8664 => 8
        }
    }

    pub fn stack_ptr(&self) -> Reg {
        match self {
            Mode::X86 => Reg::Esp,
            Mode::X8664 => Reg::Rsp
        }
    }

    pub fn base_ptr(&self) -> Reg {
        match self {
            Mode::X86 => Reg::Ebp,
            Mode::X8664 => Reg::Rbp
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[derive(Debug)]
pub enum Size {
    Quad,
    Double,
    Word,
    Byte
}