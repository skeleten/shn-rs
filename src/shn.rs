use std::sync::Arc;
use std::io::Read;

use byteorder::ReadBytesExt;
use encoding::{Encoding, DecoderTrap };

pub const SHN_CRYPT_HEADER_LEN: usize = 0x20;

pub type Result<T> = ::std::result::Result<T, ShnError>;
pub type Endianess = ::byteorder::LittleEndian;

/// Represents a data type within a `SHN` File.
#[derive(Clone, PartialEq, Debug)]
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
#[derive(Clone, PartialEq, Debug)]
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
	pub fn from_id(id: u32) -> ShnDataType {
		match id {
			1 | 12 | 16			=> ShnDataType::Byte,
			2					=> ShnDataType::UnsignedShort,
			3 | 11 | 18 | 27	=> ShnDataType::UnsignedInteger,
			5					=> ShnDataType::SingleFloatingPoint,
			9 | 24				=> ShnDataType::StringFixedLen,
			13 | 21				=> ShnDataType::SignedShort,
			20					=> ShnDataType::SignedByte,
			22					=> ShnDataType::SignedInteger,
			26					=> ShnDataType::StringZeroTerminated,
			_					=> unimplemented!(),
		}
	}
}

impl ShnCell {
	pub fn data_type(&self) -> ShnDataType {
		match self {
			&ShnCell::StringFixedLen(_)			=> ShnDataType::StringFixedLen,
			&ShnCell::StringZeroTerminated(_)	=> ShnDataType::StringZeroTerminated,
			&ShnCell::Byte(_)					=> ShnDataType::Byte,
			&ShnCell::SignedByte(_)				=> ShnDataType::SignedByte,
			&ShnCell::SignedShort(_)			=> ShnDataType::SignedShort,
			&ShnCell::UnsignedShort(_)			=> ShnDataType::UnsignedShort,
			&ShnCell::SignedInteger(_)			=> ShnDataType::SignedInteger,
			&ShnCell::UnsignedInteger(_)		=> ShnDataType::UnsignedInteger,
			&ShnCell::SingleFloatingPoint(_)	=> ShnDataType::SingleFloatingPoint,
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct ShnColumn {
	pub name:			String,
	pub data_type:		ShnDataType,
	pub data_length:	i32,
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
			data_length:		0,
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

	pub fn read<T>(&self, cursor: &mut T, enc: &Encoding) -> Result<ShnCell>
	 		where T: Read {
		match self.data_type {
			ShnDataType::StringFixedLen => {
				let mut buf = vec![0; self.data_length as usize];
				try!(cursor.read(&mut buf[..]).map_err(|_| ShnError::InvalidFile));
				let str = try!(enc.decode(&buf[..], DecoderTrap::Ignore).map_err(|e| {
					println!("error while decoding: {:?}", e);
					ShnError::InvalidEncoding
				}));
				Ok(ShnCell::StringFixedLen(str.trim_matches('\u{0}').to_string()))
			},
			ShnDataType::StringZeroTerminated => {
				let mut buf = Vec::new();
				loop {
					// testing
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

#[derive(Clone, PartialEq, Debug)]
pub struct ShnSchema {
	pub columns:		Vec<ShnColumn>,
}

impl ShnSchema {
	pub fn new() -> Self {
		ShnSchema {
			columns:	Vec::new(),
		}
	}
}

pub struct ShnRow {
	pub schema:		Arc<ShnSchema>,
	pub data:	Vec<ShnCell>
}

pub struct ShnFile {
	pub crypt_header:	[u8; SHN_CRYPT_HEADER_LEN],
	pub header:			u32, // or was it u16?
	pub schema:			Arc<ShnSchema>,
	pub data:			Vec<ShnRow>
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

#[derive(Debug)]
pub enum ShnError {
	InvalidSchema,
	InvalidFile,
	InvalidEncoding,
}
