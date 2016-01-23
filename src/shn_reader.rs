use ::std::io::{Read, Cursor };
use ::std::sync::Arc;
use ::std::num::Wrapping;
use ::shn::{
    SHN_CRYPT_HEADER_LEN,
    Endianess,
    Result,
    ShnSchema,
    ShnFile,
    ShnRow,
    ShnColumn,
    ShnDataType,
    ShnError,
};
use ::encoding::{
    Encoding,
    DecoderTrap,
};
use ::byteorder::ReadBytesExt;

pub struct ShnReader;

impl ShnReader {
	pub fn read_from<T: Read>(mut source: T, enc: &Encoding) -> Result<ShnFile> {
		let crypt_header = try!(ShnReader::read_crypt_header(&mut source));
		let data_length = try!(source.read_i32::<Endianess>().map_err(|_| ShnError::InvalidFile)) - 0x24;
		let mut data = vec![0; data_length as usize];
		try!(source.read(&mut data[..]).map_err(|_| ShnError::InvalidFile));
		ShnReader::decrypt(&mut data[..]);
		let mut reader = Cursor::new(data);

		let header = try!(reader.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let record_count = try!(reader.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let default_len = try!(reader.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let colmn_count = try!(reader.read_u32::<Endianess>().map_err(|_| ShnError::InvalidFile));
		let schema = try!(ShnReader::read_schema(&mut reader,
												 colmn_count,
												 default_len as i32,
												 enc));
		let mut file = ShnFile {
			crypt_header: crypt_header,
			header: header,
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
		// don't ask me why..
		for c in file.schema.columns.iter() {
			let d = try!(c.read(reader, enc));
			data.push(d)
		}
		Ok(ShnRow {
			schema: file.schema.clone(),
			data: data
		})
	}

	fn read_crypt_header<T: Read>(source: &mut T) -> Result<[u8; SHN_CRYPT_HEADER_LEN]> {
		let mut buffer = [0; SHN_CRYPT_HEADER_LEN];
		try!(source.read(&mut buffer).map_err(|_| ShnError::InvalidFile));
		Ok(buffer)
	}

	fn decrypt(data: &mut [u8]) {
		let mut num = data.len() as u8;
		for i in (0..data.len()).rev() {
			let old_content = data[i];
			data[i] = old_content ^ num;
			// black magic.. no idea how it works. its just transcriped it from the
			// original version from Cedric.. this really needs some cleanup
			let mut num3 = Wrapping(i as u8);
			num3 = num3 & Wrapping(15);
			num3 = num3 + Wrapping(0x55);
			num3 = num3 ^ Wrapping(i as u8) * Wrapping(11);
			num3 = num3 ^ Wrapping(num);
			num3 = num3 ^ Wrapping(170);
			let Wrapping(x) = num3;
			num = x;
		}
	}

	fn read_schema<T: Read>(source: &mut T,
				column_count: u32,
				expected_len: i32,
				enc: &Encoding) -> Result<ShnSchema> {
		let mut columns = Vec::with_capacity(column_count as usize);
		let mut len = 2; // because of that intrinsic row.
		// This one seems to be intrinsic. I don't actually think it holds any valuable data or
		// anything of relevance at all, to be honest. However it is there. weird.
		columns.push(ShnColumn {
			name: "__ID__".to_string(),
			data_type: ShnDataType::UnsignedShort,
			data_length: 2,
		});
		for _ in 0..column_count {
			let mut buf = vec![0; 48];
			try!(source.read(&mut buf[..]).map_err(|_| ShnError::InvalidFile));
			let name = try!(enc.decode(&buf[..], DecoderTrap::Strict).map_err(|_| ShnError::InvalidEncoding));
			let name = name.trim_matches('\u{0}').to_string();
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
			println!("length does not equal expected length! {} != {}", len, expected_len);
			return Err(ShnError::InvalidSchema)
		} else {
			Ok(ShnSchema {
				columns: columns,
			})
		}
	}
}