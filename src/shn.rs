use std::sync::Arc;
use std::num::Wrapping;

/// Length of the crypto header of each file
pub const SHN_CRYPT_HEADER_LEN: usize = 0x20;

pub type Result<T> = ::std::result::Result<T, ShnError>;
pub type Endianess = ::byteorder::LittleEndian;

/// De- or encrypts data. Needs to be called over the complete blob of data
/// of the file to be successfull.
pub fn decrypt(data: &mut [u8]) {
    let mut num = data.len() as u8;
    for i in (0..data.len()).rev() {
	      let old_content = data[i];
	      data[i] = old_content ^ num;
	      /* black magic.. no idea how it works. its just transcriped it from the
	       * original version from Cedric.. this really needs some cleanup
         * It seems, however to be symetrical.
         */
	      let mut num3 = Wrapping(i as u8);
	      num3 = num3 & Wrapping(15);
	      num3 = num3 + Wrapping(0x55);
	      num3 = num3 ^ (Wrapping(i as u8) * Wrapping(11));
	      num3 = num3 ^ Wrapping(num);
	      num3 = num3 ^ Wrapping(170);
	      let Wrapping(x) = num3;
	      num = x;
    }
}

/// Represents a data type within a `SHN` File.
#[derive(Clone, PartialEq, Debug)]
pub enum ShnDataType {
    /// A string with a fixed length
    StringFixedLen,
    /// A string terminated with a `00`-byte
    StringZeroTerminated,
    /// A 8 bit value
    Byte,
    /// A signed 8 bit value
    SignedByte,
    /// A signed 16 bit value
    SignedShort,
    /// An unsigned 16 bit value
    UnsignedShort,
    /// A signed 32 bit value
    SignedInteger,
    /// An unsigned 32 bit value
    UnsignedInteger,
    /// A 32 bit floating point value
    SingleFloatingPoint,
}

/// Represents a single data cell within the `SHN`-File
#[derive(Clone, PartialEq, Debug)]
pub enum ShnCell {
    /// A cell containing a `StringFixedLen` type value
    StringFixedLen(String),
    /// A cell containing a `StringZeroTerminated` type value
    StringZeroTerminated(String),
    /// A cell containing a `Byte` type value
    Byte(u8),
    /// A cell containing a `SignedByte` type value
    SignedByte(i8),
    /// A cell containing a `SignedShort` type value
    SignedShort(i16),
    /// A cell containing a `UnsignedShort` type value
    UnsignedShort(u16),
    /// A cell containing a `SignedInteger` type value
    SignedInteger(i32),
    /// A cell containing a `UnsignedInteger` type value
    UnsignedInteger(u32),
    /// A cell containing a `SingleFloatingPoint` type value
    SingleFloatingPoint(f32),
}

impl ShnDataType {
    /// Returns the `ShnDataType` matching `id`
    pub fn from_id(id: u32) -> ShnDataType {
	      match id {
	          1 | 12 | 16		=> ShnDataType::Byte,
	          2			=> ShnDataType::UnsignedShort,
	          3 | 11 | 18 | 27	=> ShnDataType::UnsignedInteger,
	          5			=> ShnDataType::SingleFloatingPoint,
	          9 | 24		=> ShnDataType::StringFixedLen,
	          13 | 21		=> ShnDataType::SignedShort,
	          20			=> ShnDataType::SignedByte,
	          22			=> ShnDataType::SignedInteger,
	          26			=> ShnDataType::StringZeroTerminated,
	          _			=> unimplemented!(),
	      }
    }
    /// Returns the lowest `id` matching the data type.
    pub fn to_id(&self) -> u32 {
        match *self {
            ShnDataType::Byte                   => 1,
            ShnDataType::UnsignedShort          => 2,
            ShnDataType::UnsignedInteger        => 3,
            ShnDataType::SingleFloatingPoint    => 5,
            ShnDataType::StringFixedLen         => 9,
            ShnDataType::SignedShort            => 13,
            ShnDataType::SignedByte             => 20,
            ShnDataType::SignedInteger          => 22,
            ShnDataType::StringZeroTerminated   => 26,
        }
    }
    /// Returns the default length of the data type represented by `self`
    pub fn default_length(&self) -> usize {
        match *self {
            ShnDataType::SignedByte |
            ShnDataType::Byte                   => 1,

            ShnDataType::SignedShort |
            ShnDataType::UnsignedShort          => 2,

            ShnDataType::SignedInteger |
            ShnDataType::UnsignedInteger |
            ShnDataType::SingleFloatingPoint    => 4,

            _                                   => 0,
        }
    }
}

impl ShnCell {
    /// Returns the matching `ShnDataType`
    pub fn data_type(&self) -> ShnDataType {
	      match *self {
	          ShnCell::StringFixedLen(_)
                => ShnDataType::StringFixedLen,
	          ShnCell::StringZeroTerminated(_)
                => ShnDataType::StringZeroTerminated,
	          ShnCell::Byte(_)
                => ShnDataType::Byte,
	          ShnCell::SignedByte(_)
                => ShnDataType::SignedByte,
	          ShnCell::SignedShort(_)
                => ShnDataType::SignedShort,
	          ShnCell::UnsignedShort(_)
                => ShnDataType::UnsignedShort,
	          ShnCell::SignedInteger(_)
                => ShnDataType::SignedInteger,
	          ShnCell::UnsignedInteger(_)
                => ShnDataType::UnsignedInteger,
	          ShnCell::SingleFloatingPoint(_)
                => ShnDataType::SingleFloatingPoint,
	      }
    }
}

#[derive(Clone, PartialEq, Debug)]
/// Represents a column in the SHN table.
pub struct ShnColumn {
    /// The name identifing the column
    pub name:		String,
    /// The type of data being held by this column
    pub data_type:	ShnDataType,
    /// The length of the data being held in this column.
    /// Only relevant for string types.
    pub data_length:	i32,
}

impl ShnColumn {
    /// Constructs a new column with the `StringFixedLen` type
    pub fn new_string_fixed_len(name: &str, len: i32) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::StringFixedLen,
	          data_length:	len,
	      }
    }

    /// Constructs a new column with the `StringZeroTerminated` type
    pub fn new_string_terminated(name: &str) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::StringZeroTerminated,
	          data_length:	0,
	      }
    }

    /// Constructs a new column with the `Byte` type
    pub fn new_byte(name: &str) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::Byte,
	          data_length:	1,
	      }
    }

    /// Constructs a new column with the `SignedByte` type
    pub fn new_signed_byte(name: &str) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::SignedByte,
	          data_length:	1,
	      }
    }

    /// Constructs a new column with the `UnsignedShort` type
    pub fn new_unsigned_short(name: &str) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::UnsignedShort,
	          data_length:	2,
	      }
    }

    /// Constructs a new column with the `SignedShort` type
    pub fn new_signed_short(name: &str) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::SignedShort,
	          data_length:	2,
	      }
    }

    /// Constructs a new column with the `UnsignedInteger` type
    pub fn new_unsigned_integer(name: &str) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::UnsignedInteger,
	          data_length:	4,
	      }
    }

    /// Constructs a new column with the `SignedInteger` type
    pub fn new_signed_integer(name: &str) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::SignedInteger,
	          data_length:	4,
	      }
    }

    /// Constructs a new column with the `SingleFloatingPoint` type
    pub fn new_single_floating_point(name: &str) -> Self {
	      ShnColumn {
	          name:		name.to_owned(),
	          data_type:		ShnDataType::SingleFloatingPoint,
	          data_length:	4,
	      }
    }
}

/// Represents the `schema` of an shn file, which is defined by a
/// collection of `ShnRow`s 
#[derive(Clone, PartialEq, Debug)]
pub struct ShnSchema {
    /// The columns defining the schema
    pub columns:		Vec<ShnColumn>,
}

impl ShnSchema {
    /// Constructs a new, empty `ShnSchema`
    pub fn new() -> Self {
	      ShnSchema {
	          columns:	Vec::new(),
	      }
    }

    /// Calculates the default length in bytes of each row.
    pub fn calculate_record_length(&self) -> i32 {
        self.columns.iter()
            .map(|c| c.data_length)
            .fold(0, |a, b| a + b)
    }
}

/// Represents a single row of data within a file
pub struct ShnRow {
    /// Reference to the schema defining the file
    pub schema:       Arc<ShnSchema>,
    /// A collection of cells containing the actual data
    pub data:	        Vec<ShnCell>
}

/// Represents a `SHN` file
pub struct ShnFile {
    /// The cryptographic header
    pub crypt_header:	[u8; SHN_CRYPT_HEADER_LEN],
    /// The header (unknown purpose)
    pub header:		u32, // or was it u16?
    /// The schema defining the file
    pub schema:	  Arc<ShnSchema>,
    /// The data held in the file
    pub data:     Vec<ShnRow>
}

impl ShnFile {
    /// Appends a row to the file (and checks it to be conform to the schema)
    pub fn append_row(&mut self, row: ShnRow) -> Result<()> {
	      if row.schema != self.schema {
	          Err(ShnError::InvalidSchema)
	      } else {
	          self.data.push(row);
	          Ok(())
	      }
    }
}

/// Wrapper for errors within the `shn-rs` crate.
#[allow(enum_variant_names, missing_docs)]
#[derive(Debug)]
pub enum ShnError {
    InvalidSchema,
    InvalidFile,
    InvalidEncoding,
}
