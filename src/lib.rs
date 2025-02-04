use std::io::{self, Read, Write};

mod errors;
mod message;
mod message_types;

pub use errors::{DecodeError, ReadError, ReceiveError};
pub use message::Message;

const START_BYTE: u8 = 0x58;
const ESCAPE_BYTE: u8 = 0x42;
const XOR_BYTE: u8 = 0x69;

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

    fn read_byte(&mut self) -> Result<u8, ReadError> {
        let mut byte = [0u8; 1];
        self.connection.read_exact(&mut byte)?;

        if byte[0] == START_BYTE {
            return Err(ReadError::NewMessage);
        }
        Ok(byte[0])
    }

    fn read_escaped_byte(&mut self) -> Result<u8, ReadError> {
        let byte = self.read_byte()?;

        if byte == ESCAPE_BYTE {
            let next_byte = self.read_byte()?;
            Ok(next_byte ^ XOR_BYTE)
        } else {
            Ok(byte)
        }
    }

    fn write_escaped_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        for &byte in bytes {
            self.write_escaped_byte(byte)?;
        }
        Ok(())
    }

    fn read_escaped_bytes(&mut self, length: usize) -> Result<Vec<u8>, ReadError> {
        let mut result = Vec::with_capacity(length);
        for _ in 0..length {
            result.push(self.read_escaped_byte()?);
        }
        Ok(result)
    }

    pub fn send(&mut self, message: Message) -> io::Result<()> {
        let message_type_bytes = message.message_type().to_le_bytes();
        let data = message.to_bytes();
        let length = (message_type_bytes.len() + data.len()) as u16;
        let length_bytes = length.to_le_bytes();

        self.connection.write_all(&[START_BYTE])?;
        self.write_escaped_bytes(&length_bytes)?;
        self.write_escaped_bytes(&message_type_bytes)?;
        self.write_escaped_bytes(&data)?;
        self.connection.flush()?;
        Ok(())
    }

    fn wait_for_start_byte(&mut self) -> io::Result<()> {
        loop {
            match self.read_byte() {
                Ok(_) => (),
                Err(ReadError::NewMessage) => return Ok(()),
                Err(ReadError::Io(e)) => return Err(e),
            }
        }
    }

    pub fn receive(&mut self) -> Result<Message, ReceiveError> {
        self.wait_for_start_byte()?;

        // Try reading messages until we succeed
        loop {
            match self.read_message() {
                Ok(message) => return Ok(message),
                Err(ReceiveError::Read(ReadError::NewMessage)) => (), // Got another START_BYTE, try reading new message
                Err(e) => return Err(e),
            }
        }
    }

    fn read_u16(&mut self) -> Result<u16, ReadError> {
        let bytes = self.read_escaped_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_message(&mut self) -> Result<Message, ReceiveError> {
        let length = self.read_u16()? as usize;
        let message_type = self.read_u16()?;
        let data = self.read_escaped_bytes(length - 2)?;
        Ok(Message::from_bytes(message_type, data)?)
    }
}

#[cfg(test)]
mod tests;
