use super::shn::{ 
    Endianess, 
    Result,
    ShnFile,
    ShnRow,
    ShnColumn,
    ShnCell,
    ShnError, 
    decrypt
};
use ::std::io::{ Write, Cursor };

use ::byteorder::WriteBytesExt; 
use ::encoding::{Encoding, EncoderTrap };

// TODO: I might want to move this to a trait instead.
pub struct ShnWriter;

impl ShnWriter {
    pub fn write_to<T>(file: &ShnFile, enc: &Encoding, writer: &mut T) 
                       -> Result<()> 
                       where T: Write + WriteBytesExt {
        // let's decompose our file for now
        let crypt_header = &file.crypt_header;
        let header = file.header;
        let schema = file.schema.clone();
        let data = &file.data;
        try!(writer.write_all(&crypt_header[..])
             .map_err(|_| ShnError::InvalidFile));

        // TODO: Write data length.
        let mut buf_wrt = Cursor::new(Vec::<u8>::new());
        // TODO: Add error enum for this kinda stuff!
        try!(buf_wrt.write_u32::<Endianess>(header)
             .map_err(|_| ShnError::InvalidFile));
        try!(buf_wrt.write_u32::<Endianess>(data.len() as u32)
             .map_err(|_| ShnError::InvalidFile));
        try!(buf_wrt.write_i32::<Endianess>(schema.default_len)
             .map_err(|_| ShnError::InvalidFile));
        try!(buf_wrt.write_u32::<Endianess>(schema.columns.len() as u32)
             .map_err(|_| ShnError::InvalidFile));

        try!(ShnWriter::write_schema(file, enc, &mut buf_wrt));
        try!(ShnWriter::write_rows(file, enc, &mut buf_wrt));

        let mut buf = buf_wrt.into_inner();
        decrypt(&mut buf[..]);

        try!(writer.write_all(&buf[..])
             .map_err(|_| ShnError::InvalidFile));
        Ok(())
    }

    fn write_schema<T>(file: &ShnFile, enc: &Encoding, writer: &mut T)
                       -> Result<()>
                       where T: Write + WriteBytesExt {
        let schema = file.schema.clone();
        /* We want to skip the first item as it is only a pseudo-column that
         * does not in fact appear within the specification of the file. Still
         * no idea as to why it is there. see ShnReader::read_schema in shn.rs
         * for more information
         */
        let mut iter = schema.columns.iter();
        iter.next();
        for column in iter {
            // TODO: Move max length into a constant
            let mut buf = Vec::with_capacity(48);
            try!(enc.encode_to(&column.name, EncoderTrap::Strict, &mut buf)
                 .map_err(|_| ShnError::InvalidEncoding));
            // Fill in the rest of the 48 bytes with 0's.
            for _ in 48..buf.len() { buf.push(0); };
            let ctype = column.data_type.to_id();
            let clen = column.data_length;
            // TODO: Better enums!
            try!(writer.write_all(&buf[..])
                 .map_err(|_| ShnError::InvalidFile));
            try!(writer.write_u32::<Endianess>(ctype)
                 .map_err(|_| ShnError::InvalidFile));
            try!(writer.write_i32::<Endianess>(clen)
                 .map_err(|_| ShnError::InvalidFile));
        }
        Ok(())
    }

    fn write_rows<T>(file: &ShnFile, enc: &Encoding, writer: &mut T)
                     -> Result<()>
                     where T: Write + WriteBytesExt {
        for row in file.data.iter() {
            try!(ShnWriter::write_row(row, enc, writer));
        }
        Ok(())
    }

    fn write_row<T>(row: &ShnRow, enc: &Encoding, writer: &mut T)
                    -> Result<()>
                    where T: Write + WriteBytesExt {
        for i in 0..row.schema.columns.len() {
            let cell: &ShnCell = row.data.get(i).unwrap();
            let column: &ShnColumn = row.schema.columns.get(i).unwrap();
            let data_len = column.data_length;
            try!(ShnWriter::write_cell(cell, data_len, enc, writer));
        }
        Ok(())
    }

    fn write_cell<T>(cell: &ShnCell, 
                     data_length: i32, 
                     enc: &Encoding, 
                     writer: &mut T) 
                     -> Result<()>
                     where T: Write + WriteBytesExt {
        match cell {
            &ShnCell::Byte(b) => 
                try!(writer.write_u8(b)
                     .map_err(|_| ShnError::InvalidFile)),
            &ShnCell::SignedByte(b) => 
                try!(writer.write_i8(b)
                     .map_err(|_| ShnError::InvalidFile)),
            &ShnCell::UnsignedShort(s) => 
                try!(writer.write_u16::<Endianess>(s)
                     .map_err(|_| ShnError::InvalidFile)),
            &ShnCell::SignedShort(s) => 
                try!(writer.write_i16::<Endianess>(s)
                     .map_err(|_| ShnError::InvalidFile)),
            &ShnCell::UnsignedInteger(i) => 
                try!(writer.write_u32::<Endianess>(i)
                     .map_err(|_| ShnError::InvalidFile)),
            &ShnCell::SignedInteger(i) => 
                try!(writer.write_i32::<Endianess>(i)
                     .map_err(|_| ShnError::InvalidFile)),
            &ShnCell::SingleFloatingPoint(f) => 
                try!(writer.write_f32::<Endianess>(f)
                     .map_err(|_| ShnError::InvalidFile)),
            &ShnCell::StringFixedLen(ref st) => {
                let mut buf = Vec::with_capacity(data_length as usize);
                try!(enc.encode_to(&st, EncoderTrap::Strict, &mut buf)
                     .map_err(|_| ShnError::InvalidEncoding));
                // Fill remaining bytes with 0's.
                for _ in 48..buf.len() { buf.push(0); }
                try!(writer.write_all(&buf[..])
                     .map_err(|_| ShnError::InvalidFile));
            },
            &ShnCell::StringZeroTerminated(ref st) => {
                let mut buf = Vec::new();
                try!(enc.encode_to(&st, EncoderTrap::Strict, &mut buf)
                     .map_err(|_| ShnError::InvalidEncoding));
                if buf[buf.len() - 1] != 0 { buf.push(0); }
                try!(writer.write_all(&buf[..])
                     .map_err(|_| ShnError::InvalidFile));
            }
        }
        Ok(())
    }
}
