use std::io::{self, Read, Write};

mod message;
mod message_types;

pub use message::Message;

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
    const START_BYTE: u8 = 0x58;
    const ESCAPE_BYTE: u8 = 0x42;
    const XOR_BYTE: u8 = 0x69;

    pub fn new(connection: T) -> Self {
        Self { connection }
    }

    fn needs_escaping(byte: u8) -> bool {
        byte == Self::START_BYTE || byte == Self::ESCAPE_BYTE
    }

    fn write_escaped_byte(&mut self, byte: u8) -> io::Result<()> {
        if Self::needs_escaping(byte) {
            self.connection.write_all(&[Self::ESCAPE_BYTE])?;
            self.connection.write_all(&[byte ^ Self::XOR_BYTE])?;
        } else {
            self.connection.write_all(&[byte])?;
        }
        Ok(())
    }

    fn read_escaped_byte(&mut self) -> io::Result<u8> {
        let mut byte = [0u8; 1];
        self.connection.read_exact(&mut byte)?;

        if byte[0] == Self::ESCAPE_BYTE {
            let mut next_byte = [0u8; 1];
            self.connection.read_exact(&mut next_byte)?;
            Ok(next_byte[0] ^ Self::XOR_BYTE)
        } else {
            Ok(byte[0])
        }
    }

    fn write_escaped_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        for &byte in bytes {
            self.write_escaped_byte(byte)?;
        }
        Ok(())
    }

    fn read_escaped_bytes(&mut self, length: usize) -> io::Result<Vec<u8>> {
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

        self.connection.write_all(&[Self::START_BYTE])?;
        self.write_escaped_bytes(&length_bytes)?;
        self.write_escaped_bytes(&message_type_bytes)?;
        self.write_escaped_bytes(&data)?;
        self.connection.flush()?;
        Ok(())
    }

    pub fn receive(&mut self) -> io::Result<Message> {
        loop {
            let mut byte = [0u8; 1];
            self.connection.read_exact(&mut byte)?;
            if byte[0] == Self::START_BYTE {
                break;
            }
        }

        // Read and unescape length
        let length_bytes = self.read_escaped_bytes(2)?;
        let length = u16::from_le_bytes([length_bytes[0], length_bytes[1]]) as usize;

        // Read and unescape the message type and data
        let buffer = self.read_escaped_bytes(length)?;

        let message_type = u16::from_le_bytes([buffer[0], buffer[1]]);
        let data = buffer[2..].to_vec();
        Ok(Message::from_bytes(message_type, data))
    }
}

#[cfg(test)]
mod tests;
