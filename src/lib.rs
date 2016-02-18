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

pub use shn_reader::ShnReader;
pub use shn_writer::ShnWriter;
