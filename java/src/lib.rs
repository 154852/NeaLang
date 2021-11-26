#![allow(dead_code)]

mod attribute;
mod classfile;
mod constantpool;
mod descriptor;
mod instructions;
mod io;

pub use classfile::*;
pub use descriptor::*;
pub use constantpool::*;
pub use instructions::*;
pub use attribute::*;

#[derive(Debug)]
pub struct Error {
    msg: String
}

impl Error {
    
}

impl<T: Into<String>> From<T> for Error {
    fn from(msg: T) -> Error {
        Error {
            msg: msg.into()
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;