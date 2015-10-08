use std::io::{Write, Result, };

pub struct BufferedWriter {
	buf:			Vec<u8>,
}

impl BufferedWriter {
	pub fn new() -> Self {
		BufferedWriter {
			buf:		Vec::new(),
		}
	}

	pub fn with_capacity(capacity: usize) -> Self {
		BufferedWriter {
			buf:		Vec::with_capacity(capacity),
		}
	}

	pub fn into_buffer(self) -> Vec<u8> {
		self.buf
	}
}

impl Write for BufferedWriter {
	fn write(&mut self, data: &[u8]) -> Result<usize> {
		for &byte in data {
			self.buf.push(byte);
		}

		Ok(data.len())
	}

	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::io::Write;
	use super::BufferedWriter;

	#[test]
	fn test_empty() {
		let buf = BufferedWriter::new();
		let buf = buf.into_buffer();
		assert!(buf[..] == []);
	}

	#[test]
	fn test_no_capacity_single_data() {
		let mut buf = BufferedWriter::new();
		buf.write(&[1, 2, 3]).ok();
		let buf = buf.into_buffer();
		assert!(buf[..] == [1, 2, 3]);
	}

	#[test]
	fn test_no_capacity_multiple_data() {
		let mut buf = BufferedWriter::new();
		buf.write(&[1, 2]).ok();
		buf.write(&[3, 4]).ok();
		let buf = buf.into_buffer();
		assert!(buf[..] == [1, 2, 3, 4]);
	}

	#[test]
	fn test_capacity_empty() {
		let buf = BufferedWriter::with_capacity(0);
		let buf = buf.into_buffer();
		assert!(buf[..] == []);
	}

	#[test]
	fn test_capacity_single_data() {
		let mut buf = BufferedWriter::with_capacity(3);
		buf.write(&[1, 2, 3]).ok();
		let buf = buf.into_buffer();
		assert!(buf[..] == [1, 2, 3]);
	}

	#[test]
	fn test_capacity_multiple_data() {
		let mut buf = BufferedWriter::with_capacity(4);
		buf.write(&[1, 2]).ok();
		buf.write(&[3, 4]).ok();
		let buf = buf.into_buffer();
		assert!(buf[..] == [1, 2, 3, 4]);
	}
}