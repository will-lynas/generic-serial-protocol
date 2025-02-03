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

    pub fn new(connection: T) -> Self {
        Self { connection }
    }

    pub fn send(&mut self, message: Message) -> io::Result<()> {
        let message_type_bytes = message.message_type().to_le_bytes();
        let data = message.to_bytes();
        let length = (message_type_bytes.len() + data.len()) as u16;

        self.connection.write_all(&[Self::START_BYTE])?;
        self.connection.write_all(&length.to_le_bytes())?;
        self.connection.write_all(&message_type_bytes)?;
        self.connection.write_all(&data)?;
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

        let mut length_bytes = [0u8; 2];
        self.connection.read_exact(&mut length_bytes)?;
        let length = u16::from_le_bytes(length_bytes) as usize;

        let mut buffer = vec![0u8; length];
        self.connection.read_exact(&mut buffer)?;

        let message_type = u16::from_le_bytes([buffer[0], buffer[1]]);
        let data = buffer[2..].to_vec();
        Ok(Message::from_bytes(message_type, data))
    }
}

#[cfg(test)]
mod tests;
