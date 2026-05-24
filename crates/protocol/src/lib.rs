include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
pub use protocol::*;

use std::io::{self, Read, Write};

/// Serialize a protobuf message and write it to a stream with a 4-byte big-endian length prefix.
///
/// This is required because TCP is a byte stream; protobuf messages have no inherent framing.
pub fn write_message<W, M>(writer: &mut W, msg: &M) -> io::Result<()>
where
    W: Write,
    M: protobuf::Message,
{
    let bytes = msg
        .write_to_bytes()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let len = bytes.len() as u32;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(&bytes)?;
    Ok(())
}

/// Read a 4-byte big-endian length prefix from a stream, then read and parse the protobuf message.
pub fn read_message<R, M>(reader: &mut R) -> io::Result<M>
where
    R: Read,
    M: protobuf::Message,
{
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;

    M::parse_from_bytes(&buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub const MAX_FILE_CHUNK_SIZE: usize = 64 * 1024;
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;