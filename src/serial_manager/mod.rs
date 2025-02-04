use crate::errors::{MaybeResyncError, ReceiveError};
use crate::message::Message;
use std::io::{self, Read, Write};

const START_BYTE: u8 = 0x58;
const ESCAPE_BYTE: u8 = 0x42;
const XOR_BYTE: u8 = 0x69;

/// An implementation of a custom serial protocol.
///
/// Message Format:
/// +------------+------------------+--------------------+------------------+
/// | Start Byte | Length (2 bytes) | Msg Type (2 bytes) |      Data        |
/// |    0x58    |     LE u16       |     LE u16         | Variable length  |
/// +------------+------------------+--------------------+------------------+
///
/// Every message starts with a start byte (0x58). This is the ground truth for the start of a message,
/// and any other start byte will trigger a resync to the next packet.
///
/// To encode a literal 0x58 within the packet (length field, msg type, or data) it must be *escaped*.
/// This is done by replacing the 0x58 with the escape byte (0x42) followed by the original byte XORed with
/// 0x69. This poses an additional problem with sending a literal 0x42. So 0x42 must itself be escaped
/// in the same way.
///
/// For example:
/// - 0x58 becomes [0x42, 0x31] (0x58 ^ 0x69 = 0x31)
/// - 0x42 becomes [0x42, 0x2B] (0x42 ^ 0x69 = 0x2B)
///
/// The length field is the size of the data field plus two bytes for the message type.
/// It is the length *before* escaping, so that the actual number of bytes transmitted may be greater than
/// this number.
///
/// All multi-byte fields are transmitted in little-endian format.
pub struct SerialManager<T>
where
    T: Read + Write,
{
    connection: T,
}

impl<T> SerialManager<T>
where
    T: Read + Write,
{
    pub fn new(connection: T) -> Self {
        Self { connection }
    }

    /// Sends a message over the serial connection
    pub fn send(&mut self, message: Message) -> io::Result<()> {
        let message_type_bytes = message.message_type().to_le_bytes();
        let data = message.to_bytes();
        #[allow(clippy::cast_possible_truncation)]
        let length = (message_type_bytes.len() + data.len()) as u16;
        let length_bytes = length.to_le_bytes();

        self.connection.write_all(&[START_BYTE])?;
        self.write_escaped_bytes(&length_bytes)?;
        self.write_escaped_bytes(&message_type_bytes)?;
        self.write_escaped_bytes(&data)?;
        self.connection.flush()?;
        Ok(())
    }

    /// Receives a message from the serial connection
    ///
    /// This function will block until a message is received.
    ///
    /// If the start byte is encountered mid-packet, the function will resync to the next packet.
    ///
    /// An error is returned if there is an IO error or if the message is malformed.
    pub fn receive(&mut self) -> Result<Message, ReceiveError> {
        self.wait_for_start_byte()?;

        loop {
            match self.read_message() {
                Ok(message) => return Ok(message),
                Err(MaybeResyncError::Resync) => (),
                Err(MaybeResyncError::Error(e)) => return Err(e),
            }
        }
    }

    fn needs_escaping(byte: u8) -> bool {
        byte == START_BYTE || byte == ESCAPE_BYTE
    }

    fn write_escaped_byte(&mut self, byte: u8) -> io::Result<()> {
        if Self::needs_escaping(byte) {
            self.connection.write_all(&[ESCAPE_BYTE])?;
            self.connection.write_all(&[byte ^ XOR_BYTE])?;
        } else {
            self.connection.write_all(&[byte])?;
        }
        Ok(())
    }

    fn write_escaped_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        for &byte in bytes {
            self.write_escaped_byte(byte)?;
        }
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, MaybeResyncError<io::Error>> {
        let mut byte = [0u8; 1];
        self.connection.read_exact(&mut byte)?;

        if byte[0] == START_BYTE {
            return Err(MaybeResyncError::Resync);
        }
        Ok(byte[0])
    }

    fn read_escaped_byte(&mut self) -> Result<u8, MaybeResyncError<io::Error>> {
        let byte = self.read_byte()?;

        if byte == ESCAPE_BYTE {
            let next_byte = self.read_byte()?;
            Ok(next_byte ^ XOR_BYTE)
        } else {
            Ok(byte)
        }
    }

    fn read_escaped_bytes(
        &mut self,
        length: usize,
    ) -> Result<Vec<u8>, MaybeResyncError<io::Error>> {
        let mut result = Vec::with_capacity(length);
        for _ in 0..length {
            result.push(self.read_escaped_byte()?);
        }
        Ok(result)
    }

    fn read_u16(&mut self) -> Result<u16, MaybeResyncError<io::Error>> {
        let bytes = self.read_escaped_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn wait_for_start_byte(&mut self) -> io::Result<()> {
        loop {
            match self.read_byte() {
                Ok(_) => (),
                Err(MaybeResyncError::Resync) => return Ok(()),
                Err(MaybeResyncError::Error(e)) => return Err(e),
            }
        }
    }

    fn read_message(&mut self) -> Result<Message, MaybeResyncError<ReceiveError>> {
        let length = self.read_u16()? as usize;
        let message_type = self.read_u16()?;
        let data = self.read_escaped_bytes(length - 2)?;
        Ok(Message::from_bytes(message_type, data)?)
    }
}

#[cfg(test)]
mod tests;
