This crate provides convenience methods for reading and writing data to binary buffers. It supports writing primitive types, as well as Strings and Vectors to in-memory streams and files.

If you want to write data to a buffer you do it like so:
```rust
use crate::buffer::{BufferReader, BufferWriter, SeekOrigin};
use std::io::Cursor;
let mut buffer = BufferWriter::new(Cursor::new(Vec::new()));
buffer.write_u32(9001)?;
buffer.write_u32(9002).unwrap()?;
buffer.write_string("Hello World!")?;
```

You can then return the full buffer as a vector:
```rust
let data = buffer.to_vec()?;
```

Reading buffers is just as simple.
```rust
let mut reader = BufferReader::new(File::open("test.bin")?);
let magic = reader.read_u32()?;
let body = reader.read_string()?;
```