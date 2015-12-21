#![allow(dead_code)]
extern crate encoding;
extern crate byteorder;

mod buffed_io;
mod iterex;

use std::sync::Arc;
use std::io::{ Read, Cursor};

use byteorder::{ReadBytesExt, };
use encoding::{Encoding, DecoderTrap };

use iterex::{IteratorEx, };

pub const SHN_CRYPT_HEADER_LEN: usize = 36;

pub type Result<T> = std::result::Result<T, ShnError>;
pub type Endianess = byteorder::BigEndian;

/// Represents a data type within a `SHN` File.
#[derive(Clone, PartialEq)]
pub enum ShnDataType {
	StringFixedLen,
	StringZeroTerminated,
	Byte,
	SignedByte,
	SignedShort,
	UnsignedShort,
	SignedInteger,
	UnsignedInteger,
	SingleFloatingPoint,
}

/// Represents a single data cell within the `SHN`-File
#[derive(Clone, PartialEq)]
pub enum ShnCell {
	StringFixedLen(String),
	StringZeroTerminated(String),
	Byte(u8),
	SignedByte(i8),
	SignedShort(i16),
	UnsignedShort(u16),
	SignedInteger(i32),
	UnsignedInteger(u32),
	SingleFloatingPoint(f32),
}

impl ShnDataType {
	fn from_id(id: u32) -> ShnDataType {
		match id {
			1 | 9 | 24 			=> ShnDataType::StringFixedLen,
			26 					=> ShnDataType::StringZeroTerminated,
			12 | 16				=> ShnDataType::Byte,
			20					=> ShnDataType::SignedByte,
			13 | 21				=> ShnDataType::SignedShort,
			2					=> ShnDataType::UnsignedShort,
			22					=> ShnDataType::SignedInteger,
			3 | 11 | 18 | 27	=> ShnDataType::UnsignedInteger,
			5					=> ShnDataType::SingleFloatingPoint,
			_					=> unimplemented!(),
		}
	}

	fn to_id(data_type: ShnDataType) -> u32 {
		// as often multiple id's match to the same type, we'll always return
		// the lowest id.
		match data_type {
			ShnDataType::StringFixedLen	=> 1,
			ShnDataType::StringZeroTerminated => 26,
			ShnDataType::Byte => 12,
			ShnDataType::SignedByte => 20,
			ShnDataType::SignedShort => 13,
			ShnDataType::UnsignedShort => 2,
			ShnDataType::SignedInteger => 22,
			ShnDataType::UnsignedInteger => 3,
			ShnDataType::SingleFloatingPoint => 5,
		}
	}
}

impl ShnCell {
	pub fn data_type(&self) -> ShnDataType {
		match self {
			&ShnCell::StringFixedLen(_)
				=> ShnDataType::StringFixedLen,
			&ShnCell::StringZeroTerminated(_)
				=> ShnDataType::StringZeroTerminated,
			&ShnCell::Byte(_)
				=> ShnDataType::Byte,
			&ShnCell::SignedByte(_)
				=> ShnDataType::SignedByte,
			&ShnCell::SignedShort(_)
				=> ShnDataType::SignedShort,
			&ShnCell::UnsignedShort(_)
				=> ShnDataType::UnsignedShort,
			&ShnCell::SignedInteger(_)
				=> ShnDataType::SignedInteger,
			&ShnCell::UnsignedInteger(_)
				=> ShnDataType::UnsignedInteger,
			&ShnCell::SingleFloatingPoint(_)
				=> ShnDataType::SingleFloatingPoint,
		}
	}
}

#[derive(Clone, PartialEq)]
pub struct ShnColumn {
	name:			String,
	data_type:		ShnDataType,
	data_length:	i32,
}

impl ShnColumn {
	pub fn new_string_fixed_len(name: &str, len: i32) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::StringFixedLen,
			data_length:		len,
		}
	}

	pub fn new_string_terminated(name: &str) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::StringZeroTerminated,
			data_length:		0,	// actually no idea what this is supposed to
									// be.
		}
	}

	pub fn new_byte(name: &str) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::Byte,
			data_length:		1,
		}
	}

	pub fn new_signed_byte(name: &str) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::SignedByte,
			data_length:		1,
		}
	}

	pub fn new_unsigned_short(name: &str) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::UnsignedShort,
			data_length:		2,
		}
	}

	pub fn new_signed_short(name: &str) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::SignedShort,
			data_length:		2,
		}
	}

	pub fn new_unsigned_integer(name: &str) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::UnsignedInteger,
			data_length:		4,
		}
	}

	pub fn new_signed_integer(name: &str) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::SignedInteger,
			data_length:		4,
		}
	}

	pub fn new_single_floating_point(name: &str) -> Self {
		ShnColumn {
			name:				name.to_string(),
			data_type:			ShnDataType::SingleFloatingPoint,
			data_length:		4,
		}
	}

	pub fn read<T>(&self, cursor: &mut Cursor<T>, enc: &Encoding) -> Result<ShnCell>
	 		where T: AsRef<[u8]>{
		match self.data_type {
			ShnDataType::StringFixedLen => {
				let mut buf = Vec::with_capacity(self.data_length as usize);
				try!(cursor.read(&mut buf[..]).map_err(|_| ShnError::InvalidFile));
				let str = try!(enc.decode(&buf[..], DecoderTrap::Ignore)
								.map_err(|_| ShnError::InvalidEncoding));
				Ok(ShnCell::StringFixedLen(str))
			},
			ShnDataType::StringZeroTerminated => {
				let mut buf = Vec::new();
				loop {
					let d = try!(cursor.read_u8().map_err(|_| ShnError::InvalidFile));
					if d == 0 { break; }
					buf.push(d);
				}
				let str = try!(enc.decode(&buf[..], DecoderTrap::Ignore)
								.map_err(|_| ShnError::InvalidEncoding));
				Ok(ShnCell::StringZeroTerminated(str))
			},
			ShnDataType::Byte => {
				let d = try!(cursor.read_u8().map_err(|_| ShnError::InvalidFile));
				Ok(ShnCell::Byte(d))
			},
			ShnDataType::SignedByte => {
				let d = try!(cursor.read_i8().map_err(|_| ShnError::InvalidFile));
				Ok(ShnCell::SignedByte(d))
			},
			ShnDataType::SignedShort => {
				let d = try!(cursor.read_i16::<Endianess>().map_err(|_| ShnError::InvalidFile));
				Ok(ShnCell::SignedShort(d))
			},
			ShnDataType::UnsignedShort => {
				let d = try!(cursor.read_u16::<Endianess>().map_err(|_| ShnError::InvalidFile));
				Ok(ShnCell::UnsignedShort(d))
			},
			ShnDataType::SignedInteger => {
				let d = try!(cursor.read_i32::<Endianess>().map_err(|_| ShnError::InvalidFile));
				Ok(ShnCell::SignedInteger(d))
			},
			ShnDataType::UnsignedInteger => {
				let d = try!(cursor.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
				Ok(ShnCell::UnsignedInteger(d))
			},
			ShnDataType::SingleFloatingPoint => {
				let d = try!(cursor.read_f32::<Endianess>().map_err(|_| ShnError::InvalidFile));
				Ok(ShnCell::SingleFloatingPoint(d))
			}
		}
	}
}

#[derive(Clone, PartialEq)]
pub struct ShnSchema {
	columns:		Vec<ShnColumn>,
}

impl ShnSchema {
	pub fn new() -> Self {
		ShnSchema {
			columns:	Vec::new(),
		}
	}
}

pub struct ShnRow {
	// We don't want to allow altering the schema while already having any data.
	schema:		Arc<ShnSchema>,
	data:		Vec<ShnCell>
}

pub struct ShnFile {
	crypt_header:	[u8; SHN_CRYPT_HEADER_LEN],
	header:			u32, // or was it u16?
	schema:			Arc<ShnSchema>,
	data:			Vec<ShnRow>
}

impl ShnFile {
	pub fn append_row(&mut self, row: ShnRow) -> Result<()> {
		if row.schema != self.schema {
			Err(ShnError::InvalidSchema)
		} else {
			self.data.push(row);
			Ok(())
		}
	}
}

pub enum ShnError {
	InvalidSchema,
	InvalidFile,
	InvalidEncoding,
}

pub struct ShnReader;

impl ShnReader {
	pub fn read_from<T: Read>(mut source: T, enc: &Encoding) -> Result<ShnFile> {
		let _crypt_header = try!(ShnReader::read_crypt_header(&mut source));
		let data_length = try!(source.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let mut data = vec![0; data_length as usize];
		try!(source.read(&mut data[..]).map_err(|_| ShnError::InvalidFile));
		ShnReader::decrypt(&mut data[..]);
		let mut reader = Cursor::new(data);

		let _header = try!(reader.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let record_count = try!(reader.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let default_len = try!(reader.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let colmn_count = try!(reader.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let schema = try!(ShnReader::read_schema(&mut reader,
												 colmn_count,
												 default_len as i32,
												 enc));
		let mut file = ShnFile {
			crypt_header: _crypt_header,
			header: _header,
			schema: Arc::new(schema),
			data: Vec::new()
		};
		try!(ShnReader::read_rows(&mut file, &mut reader, record_count as usize, enc));
		Ok(file)
	}

	fn read_rows<T>(file: &mut ShnFile,
						reader: &mut Cursor<T>,
						count: usize,
						enc: &Encoding) -> Result<()>
			where T: AsRef<[u8]> {

		for _ in 0..count {
			let row = try!(ShnReader::read_row(file, reader, enc));
			file.data.push(row);
		}
		Ok(())
	}

	fn read_row<T>(file: &mut ShnFile,
					reader: &mut Cursor<T>,
					enc: &Encoding) -> Result<ShnRow>
			where T: AsRef<[u8]> {
		let mut data = Vec::new();
		for c in file.schema.columns.iter() {
			let d = try!(c.read(reader, enc));
			data.push(d)
		}
		Err(ShnError::InvalidFile)
	}

	fn read_crypt_header<T: Read>(source: &mut T) -> Result<[u8; SHN_CRYPT_HEADER_LEN]> {
		let mut buffer = [0; SHN_CRYPT_HEADER_LEN];
		try!(source.read(&mut buffer).map_err(|_| ShnError::InvalidFile));
		Ok(buffer)
	}

	fn decrypt(data: &mut [u8]) {
		let mut num = data.len() as u8;
		for i in (0..data.len()).reverse() {
			let old_content = data[i];
			data[i] = old_content ^ num;
			// black magic.. no idea how it works. its just transcriped it from the
			// original version from Cedric.. this really needs some cleanup
			let mut num3 = i as u8;
			num3 = num3 & 15;
			num3 = num3 + 0x55;
			num3 = num3 ^ ((i as u8) * 11);
			num3 = num3 ^ num;
			num3 = num3 ^ 170;
			num = num3;
		}
	}

	fn read_schema<T: Read>(source: &mut T,
							column_count: u32,
							expected_len: i32,
							enc: &Encoding) -> Result<ShnSchema> {
		let mut columns = Vec::with_capacity(column_count as usize);
		let mut len = 0;
		for _ in 0..column_count {
			let mut buf = vec![0; 48];
			try!(source.read(&mut buf[..]).map_err(|_| ShnError::InvalidFile));
			let name = try!(enc.decode(&buf[..], DecoderTrap::Strict)
							.map_err(|_| ShnError::InvalidEncoding));
			let ctype = try!(source.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
			let clen = try!(source.read_i32::<Endianess>().map_err(|_| ShnError::InvalidFile));
			columns.push(ShnColumn {
				name: name,
				data_type: ShnDataType::from_id(ctype),
				data_length: clen,
			});
			len = len + clen;
		}

		if len != expected_len {
			Err(ShnError::InvalidSchema)
		} else {
			Ok(ShnSchema {
				columns: columns,
			})
		}
	}
}

#[cfg(test)]
mod shn_cell_tests {
	use super::{ShnCell, ShnDataType, };

	#[test]
	fn fixed_len_string_data_type() {
		let cell = ShnCell::StringFixedLen("something..".to_string());
		assert!(cell.data_type() == ShnDataType::StringFixedLen);
	}

	#[test]
	fn terminated_string_data_type() {
		let cell = ShnCell::StringZeroTerminated("something..".to_string());
		assert!(cell.data_type() == ShnDataType::StringZeroTerminated);
	}

	#[test]
	fn byte_data_type() {
		let cell = ShnCell::Byte(0);
		assert!(cell.data_type() == ShnDataType::Byte);
	}

	#[test]
	fn signed_byte_data_type() {
		let cell = ShnCell::SignedByte(0);
		assert!(cell.data_type() == ShnDataType::SignedByte);
	}

	#[test]
	fn signed_short_data_type() {
		let cell = ShnCell::SignedShort(0);
		assert!(cell.data_type() == ShnDataType::SignedShort);
	}

	#[test]
	fn unsigned_short_data_type() {
		let cell = ShnCell::UnsignedShort(0);
		assert!(cell.data_type() == ShnDataType::UnsignedShort);
	}

	#[test]
	fn signed_integer_data_type() {
		let cell = ShnCell::SingedInteger(0);
		assert!(cell.data_type() == ShnDataType::SingedInteger);
	}

	#[test]
	fn unsigned_integer_data_type() {
		let cell = ShnCell::UnsignedInteger(0);
		assert!(cell.data_type() == ShnDataType::UnsignedInteger);
	}

	#[test]
	fn single_float_data_type() {
		let cell = ShnCell::SingleFloatingPoint(0.0);
		assert!(cell.data_type() == ShnDataType::SingleFloatingPoint);
	}
}
