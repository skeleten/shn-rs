#![feature(plugin)]
#![plugin(clippy)]

extern crate encoding;
extern crate byteorder;

mod shn;
mod shn_reader;
mod shn_writer;

pub use shn::{
    SHN_CRYPT_HEADER_LEN,
    ShnDataType,
    ShnCell,
    ShnColumn,
    ShnSchema,
    ShnRow,
    ShnFile,
    ShnError,
    decrypt as shn_decrypt,
};

pub use shn_reader::ShnReader;
pub use shn_writer::ShnWriter;
