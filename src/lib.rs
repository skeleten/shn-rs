#![allow(dead_code)]
extern crate encoding;

mod buffed_io;

pub const SHN_CRYPT_HEADER_LEN: usize = 36;

/// Represents a data type within a `SHN` File.
#[derive(Clone, PartialEq)]
pub enum ShnDataType {
	StringFixedLen,
	StringZeroTerminated,
	Byte,
	SignedByte,
	SignedShort,
	UnsignedShort,
	SingedInteger,
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
	SingedInteger(i32),
	UnsignedInteger(u32),
	SingleFloatingPoint(f32),
}

impl ShnDataType {
	fn id_to_data_type(id: u32) -> ShnDataType {
		match id {
			1 | 9 | 24 			=> ShnDataType::StringFixedLen,
			26 					=> ShnDataType::StringZeroTerminated,
			12 | 16				=> ShnDataType::Byte,
			20					=> ShnDataType::SignedByte,
			13 | 21				=> ShnDataType::SignedShort,
			2					=> ShnDataType::UnsignedShort,
			22					=> ShnDataType::SingedInteger,
			3 | 11 | 18 | 27	=> ShnDataType::UnsignedInteger,
			5					=> ShnDataType::SingleFloatingPoint,
			_					=> unimplemented!(),
		}
	}

	fn data_type_to_id(data_type: ShnDataType) -> u32 {
		// as often multiple id's match to the same type, we'll always return
		// the lowest id.
		match data_type {
			ShnDataType::StringFixedLen	=> 1,
			ShnDataType::StringZeroTerminated => 26,
			ShnDataType::Byte => 12,
			ShnDataType::SignedByte => 20,
			ShnDataType::SignedShort => 13,
			ShnDataType::UnsignedShort => 2,
			ShnDataType::SingedInteger => 22,
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
			&ShnCell::SingedInteger(_) 
				=> ShnDataType::SingedInteger,
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
	data_length:	u32,
}

impl ShnColumn {
	pub fn new_string_fixed_len(name: &str, len: u32) -> Self {
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
			data_type:			ShnDataType::SingedInteger,
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

pub struct ShnRow<'a> {
	// We don't want to allow altering the schema while already having any data.
	schema:		&'a ShnSchema,
	data:		Vec<ShnCell>
}

pub struct ShnFile<'a> {
	crypt_header:	[u8; SHN_CRYPT_HEADER_LEN],
	header:			u32, // or was it u16?
	schema:			ShnSchema,
	data:			Vec<ShnRow<'a>>
}

impl<'a> ShnFile<'a> {

	pub fn get_schema(&'a self) -> &'a ShnSchema {
		&self.schema
	}

	pub fn append_row(&mut self, row: ShnRow<'a>) -> Result<(), ShnError> {
		if row.schema != &self.schema {
			Err(ShnError::InvalidSchema)
		} else {
			self.data.push(row);
			Ok(())
		}
	}
}

pub enum ShnError {
	InvalidSchema,
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