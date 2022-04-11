mod module;
pub(crate) mod encode;
mod ins;

pub use module::*;
pub use ins::*;

#[cfg(test)]
mod tests;

// See: https://webassembly.github.io/spec/core/