//! # shn-rs
//! provides an api to work with `*.shn` files found in the game `Fiesta Online`
//! Said files are statically typed data tables with a less then ideal format.

#![feature(plugin)]
#![plugin(clippy)]

#![deny(missing_docs)]

extern crate encoding;
extern crate byteorder;

mod shn;
mod shn_reader;
mod shn_writer;

pub use shn::{
    ShnDataType,
    ShnCell,
    ShnColumn,
    ShnSchema,
    ShnRow,
    ShnFile,
    ShnError,
};

/// Reads a `ShnFile` from the provided input, using the given encoding
/// for any strings
pub fn read_from<S: std::io::Read>(source: &mut S,
                                   encoding: &encoding::EncodingRef)
                                   -> shn::Result<ShnFile> {
    shn_reader::ShnReader::read_from(source, encoding)
}

/// Writes the `ShnFile` to the provided output, using the given encoding
/// for any strings.
pub fn write_to<D: std::io::Write>(dest: &mut D,
                                   file: &shn::ShnFile,
                                   encoding: &encoding::EncodingRef)
                                   -> shn::Result<()> {
    shn_writer::ShnWriter::write_to(file, encoding, dest)
}
