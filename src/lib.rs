#[macro_use]
pub mod buffer {
    use std::io::Error;
    use std::io::{Read, Seek, SeekFrom, Write};

    /// Specifies the position in a stream to use for seeking.
    #[derive(PartialEq)]
    pub enum SeekOrigin {
        /// Specifies the beginning of a stream.
        Begin,
        /// Specifies the current position within a stream.
        Current,
        /// Specifies the end of a stream.
        End,
    }

    /// Endianness refers to the order of bytes (or sometimes bits) within a binary representation of a number.
    #[derive(PartialEq)]
    pub enum Endianness {
        /// The least significant byte (LSB) value, 0Dh, is at the lowest address.
        /// The other bytes follow in increasing order of significance.
        /// This is akin to right-to-left reading in hexadecimal order.
        Little,
        /// The most significant byte (MSB) value, 0Ah, is at the lowest address.
        /// The other bytes follow in decreasing order of significance.
        /// This is akin to left-to-right reading in hexadecimal order.
        Big,
    }

    /// Writes primitive types in binary to a stream and supports writing strings in a specific encoding.
    pub struct BufferWriter<W: Write> {
        pub writer: W,
    }

    impl<W: Write> BufferWriter<W>
    where
        W: Seek + Read + Write,
    {
        /// Creates a new BufferWriter instance
        pub fn new(writer: W) -> Self {
            BufferWriter { writer: writer }
        }
        /// Gets the position within the current stream.
        pub fn position(&mut self) -> Result<u64, BufferError> {
            self.seek(0, SeekOrigin::Current)
        }
        /// Gets the length in bytes of the stream.
        pub fn len(&mut self) -> Result<u64, BufferError> {
            let old_pos = self.position()?;
            let len = self.seek(0, SeekOrigin::End)?;
            if old_pos != len {
                self.seek(old_pos as i64, SeekOrigin::Begin)?;
            }
            Ok(len)
        }
        pub fn to_vec(&mut self) -> Result<Vec<u8>, BufferError> {
            let mut out: Vec<u8> = vec![];
            self.seek(0, SeekOrigin::Begin)?;
            self.writer.read_to_end(&mut out).unwrap();
            Ok(out)
        }
        pub fn seek(&mut self, position: i64, origin: SeekOrigin) -> Result<u64, BufferError> {
            match origin {
                SeekOrigin::Begin => self.writer.seek(SeekFrom::Start(position as u64)),
                SeekOrigin::Current => self.writer.seek(SeekFrom::Current(position)),
                SeekOrigin::End => self.writer.seek(SeekFrom::End(position)),
            }
            .map(|o| o)
            .map_err(|_e| BufferError::IndexOutOfRange { index: position })
        }

        /// Writes a four-byte unsigned integer to the current stream
        /// and advances the stream position by four bytes.
        pub fn write_u32(&mut self, value: u32) -> Result<u64, BufferError> {
            let data = &[
                (value >> 0) as u8,
                (value >> 8) as u8,
                (value >> 16) as u8,
                (value >> 24) as u8,
            ];
            self.writer
                .write(data)
                .map(|o| o as u64)
                .map_err(|_e| BufferError::IOFailure)
        }

        /// Writes an eight-byte unsigned integer to the current stream
        /// and advances the stream position by eight bytes.
        pub fn write_u64(&mut self, value: u64) -> Result<u64, BufferError> {
            let data = &[
                (value >> 0) as u8,
                (value >> 8) as u8,
                (value >> 16) as u8,
                (value >> 24) as u8,
                (value >> 32) as u8,
                (value >> 40) as u8,
                (value >> 48) as u8,
                (value >> 56) as u8,
            ];
            self.writer
                .write(data)
                .map(|o| o as u64)
                .map_err(|_e| BufferError::IOFailure)
        }

        /// Writes a four-byte signed integer to the current stream
        /// and advances the stream position by four bytes.
        pub fn write_i32(&mut self, value: i32) -> Result<u64, BufferError> {
            let data = &[
                (value >> 0) as u8,
                (value >> 8) as u8,
                (value >> 16) as u8,
                (value >> 24) as u8,
            ];
            self.writer
                .write(data)
                .map(|o| o as u64)
                .map_err(|_e| BufferError::IOFailure)
        }

        /// Writes a two-byte unsigned integer to the current stream
        /// and advances the stream position by two bytes.
        pub fn write_u16(&mut self, value: u16) -> Result<u64, BufferError> {
            let data = &[(value >> 0) as u8, (value >> 8) as u8];
            self.writer
                .write(data)
                .map(|o| o as u64)
                .map_err(|_e| BufferError::IOFailure)
        }

        /// Writes an unsigned byte to the current stream
        /// and advances the stream position by one byte.
        pub fn write_u8(&mut self, value: u8) -> Result<u64, BufferError> {
            self.writer
                .write(&[value])
                .map(|o| o as u64)
                .map_err(|_e| BufferError::IOFailure)
        }

        /// Write out an int 7 bits at a time. The high bit of the byte,
        /// when on, tells reader to continue reading more bytes.
        pub fn write_7bit_int(&mut self, value: i32) -> Result<(), BufferError> {
            let mut v = value as u32;
            while v >= 0x80 {
                self.write_u8((v | 0x80) as u8)?;
                v >>= 7;
            }
            self.write_u8(v as u8)?;
            Ok(())
        }

        /// Writes a length-prefixed string to this stream in UTF8-encoding
        /// and advances the current position of the stream in accordance with the encoding
        /// used and the specific characters being written to the stream.
        pub fn write_string(&mut self, value: String) -> Result<u64, BufferError> {
            let bytes = value.as_bytes();
            self.write_7bit_int(bytes.len() as i32)?;
            self.writer
                .write(bytes)
                .map(|o| o as u64)
                .map_err(|_e| BufferError::IOFailure)
        }

        /// Writes a section of a bytes to the current stream, and advances the current position of the stream
        pub fn write_bytes(&mut self, value: &Vec<u8>) -> Result<u64, BufferError> {
            self.writer
                .write(value)
                .map(|o| o as u64)
                .map_err(|_e| BufferError::IOFailure)
        }
    }

    /// Reads primitive data types as binary values in a specific encoding.
    pub struct BufferReader<R: Read> {
        pub reader: R,
    }

    impl<R: Read> BufferReader<R>
    where
        R: Seek + Read + Write,
    {
        /// Creates a new BufferReader
        pub fn new(reader: R) -> Self {
            BufferReader { reader: reader }
        }
        /// Gets the position within the current stream.
        pub fn position(&mut self) -> Result<u64, BufferError> {
            self.seek(0, SeekOrigin::Current)
        }
        /// Gets the length in bytes of the stream.
        pub fn len(&mut self) -> Result<u64, BufferError> {
            let old_pos = self.position()?;
            let len = self.seek(0, SeekOrigin::End)?;
            if old_pos != len {
                self.seek(old_pos as i64, SeekOrigin::Begin)?;
            }
            Ok(len)
        }
        pub fn seek(&mut self, position: i64, origin: SeekOrigin) -> Result<u64, BufferError> {
            match origin {
                SeekOrigin::Begin => self.reader.seek(SeekFrom::Start(position as u64)),
                SeekOrigin::Current => self.reader.seek(SeekFrom::Current(position)),
                SeekOrigin::End => self.reader.seek(SeekFrom::End(position)),
            }
            .map(|o| o as u64)
            .map_err(|_e| BufferError::IndexOutOfRange { index: position })
        }

        /// Reads in a 32-bit integer in compressed format.
        pub fn read_7bit_int(&mut self) -> Result<i32, BufferError> {
            let mut count: i32 = 0;
            let mut shift = 0;
            let mut b: u8 = 0;
            while {
                // Check for a corrupted stream.  Read a max of 5 bytes.
                // In a future version, add a DataFormatException.
                if shift == 5 * 7 {
                    // 5 bytes max per Int32, shift += 7
                    // too many bytes in what should have been a 7 bit encoded i32.
                    return Err(BufferError::IOFailure);
                }
                // read_u8 handles end of stream cases for us.
                b = self.read_u8()?;
                count |= ((b & 0x7F) as i32) << shift;
                shift += 7;
                (b & 0x80) != 0
            } {}
            Ok(count)
        }
        /// Reads a null-terminated string from the buffer
        pub fn read_string(&mut self) -> Result<String, BufferError> {
            let string_length = self.read_7bit_int()?;
            if string_length < 0 {
                return Err(BufferError::IOFailure);
            }
            if string_length == 0 {
                return Ok(String::default());
            }
            let chars = self.read_bytes(string_length as u64)?;
            String::from_utf8(chars)
                .map(|o| o)
                .map_err(|_e| BufferError::IOFailure)
        }

        /// Reads a 4-byte unsigned integer from the current vector
        /// and advances the position of the cursor by four bytes.
        pub fn read_u32(&mut self) -> Result<u32, BufferError> {
            let size = std::mem::size_of::<u32>() as u64;
            if self.position()? + size > self.len()? {
                return Err(BufferError::EndOfStream);
            }
            let mut buffer = [0u8; 4];
            self.reader
                .read_exact(&mut buffer)
                .map_err(|e| BufferError::ReadFailure { error: e })
                .map(|_b| {
                    ((buffer[0] as u32) << 0)
                        | ((buffer[1] as u32) << 8)
                        | ((buffer[2] as u32) << 16)
                        | ((buffer[3] as u32) << 24)
                })
        }

        /// Reads a 8-byte unsigned integer from the current vector
        /// and advances the position of the cursor by eight bytes.
        pub fn read_u64(&mut self) -> Result<u64, BufferError> {
            let size = std::mem::size_of::<u64>() as u64;
            if self.position()? + size > self.len()? {
                return Err(BufferError::EndOfStream);
            }
            let mut buffer = vec![0u8; 8];
            self.reader
                .read_exact(&mut buffer)
                .map_err(|e| BufferError::ReadFailure { error: e })
                .map(|_b| {
                    let lo = (buffer[0] as u32)
                        | (buffer[1] as u32) << 8
                        | (buffer[2] as u32) << 16
                        | (buffer[3] as u32) << 24;
                    let hi = (buffer[4] as u32)
                        | (buffer[5] as u32) << 8
                        | (buffer[6] as u32) << 16
                        | (buffer[7] as u32) << 24;

                    (hi as u64) << 32 | lo as u64
                })
        }

        /// Reads a 4-byte signed integer from the current vector
        /// and advances the current position of the cursor by four bytes.
        pub fn read_i32(&mut self) -> Result<i32, BufferError> {
            let size = std::mem::size_of::<i32>() as u64;
            if self.position()? + size > self.len()? {
                return Err(BufferError::EndOfStream);
            }
            let mut buffer = [0u8; 4];
            self.reader
                .read_exact(&mut buffer)
                .map_err(|e| BufferError::ReadFailure { error: e })
                .map(|_b| {
                    ((buffer[0] as i32) << 0)
                        | ((buffer[1] as i32) << 8)
                        | ((buffer[2] as i32) << 16)
                        | ((buffer[3] as i32) << 24)
                })
        }

        /// Reads a 2-byte unsigned integer from the current vector using little-endian encoding
        /// and advances the position of the cursor by two bytes.
        pub fn read_u16(&mut self) -> Result<u16, BufferError> {
            let size = std::mem::size_of::<u16>() as u64;
            if self.position()? + size > self.len()? {
                return Err(BufferError::EndOfStream);
            }
            let mut buffer = [0u8; 2];
            self.reader
                .read_exact(&mut buffer)
                .map_err(|e| BufferError::ReadFailure { error: e })
                .map(|_b| (buffer[0] as u16) | (buffer[1] as u16))
        }

        /// Reads the next byte from the current vector
        /// and advances the current position of the cursor by one byte.
        pub fn read_u8(&mut self) -> Result<u8, BufferError> {
            let size = std::mem::size_of::<u8>() as u64;
            if self.position()? + size > self.len()? {
                return Err(BufferError::EndOfStream);
            }
            let mut buffer = [0u8; 1];
            self.reader
                .read_exact(&mut buffer)
                .map_err(|e| BufferError::ReadFailure { error: e })
                .map(|_b| buffer[0])
        }

        /// Reads the specified number of bytes from the current stream
        /// into a byte array and advances the current position by that number of bytes.
        pub fn read_bytes(&mut self, count: u64) -> Result<Vec<u8>, BufferError> {
            if self.position()? + count > self.len()? {
                return Err(BufferError::EndOfStream);
            }
            let mut buffer = vec![0u8; count as usize];
            self.reader
                .read_exact(&mut buffer)
                .map_err(|e| BufferError::ReadFailure { error: e })
                .map(|_b| buffer)
        }

        /// Reads the specified number of bytes at a pointer from the current stream
        /// into a byte array without advancing the current position.
        pub fn read_bytes_at(&mut self, offset: u64, count: u64) -> Result<Vec<u8>, BufferError> {
            if offset + count > self.len()? {
                return Err(BufferError::EndOfStream);
            }
            let current_pos = self.position()?;
            self.seek(offset as i64, SeekOrigin::Begin)?;
            let buffer = self.read_bytes(count)?;
            self.seek(current_pos as i64, SeekOrigin::Begin)?;
            Ok(buffer)
        }
    }

    #[derive(Debug)]
    pub enum BufferError {
        IndexOutOfRange { index: i64 },
        EndOfStream,
        ReadFailure { error: Error },
        IOFailure,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::buffer::{BufferReader, BufferWriter, SeekOrigin};
        use std::io::Cursor;
        let mut buffer = BufferWriter::new(Cursor::new(Vec::new()));
        buffer.write_u32(9001).unwrap();
        buffer.write_u32(9002).unwrap();
        buffer.write_string("Hello World!".to_string()).unwrap();
        buffer.seek(0, SeekOrigin::Begin).unwrap();
        buffer.write_u32(9003).unwrap();
        let data = buffer.to_vec().unwrap();
        let mut reader = BufferReader::new(Cursor::new(data));
        assert_eq!(9003, reader.read_u32().unwrap());
        assert_eq!(9002, reader.read_u32().unwrap());
        assert_eq!("Hello World!", reader.read_string().unwrap());
    }
}
