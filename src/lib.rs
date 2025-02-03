use std::io::{self, Read, Write};

mod messages {
    #[derive(Debug, PartialEq, Clone)]
    pub struct Bytes {
        pub data: Vec<u8>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct U8 {
        pub num: u8,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct U16 {
        pub num: u16,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct MyString {
        pub string: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Multi {
        pub num: u8,
        pub string: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct NoOp {}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Bytes(messages::Bytes),
    U8(messages::U8),
    MyString(messages::MyString),
    Multi(messages::Multi),
    NoOp(messages::NoOp),
    U16(messages::U16),
}

impl Message {
    fn message_type(&self) -> u16 {
        match self {
            Message::Bytes(_) => 0,
            Message::U8(_) => 1,
            Message::MyString(_) => 2,
            Message::Multi(_) => 3,
            Message::NoOp(_) => 4,
            Message::U16(_) => 5,
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            Message::Bytes(msg) => bytes.extend(msg.data),
            Message::U8(msg) => bytes.push(msg.num),
            Message::MyString(msg) => bytes.extend(msg.string.as_bytes()),
            Message::Multi(msg) => {
                bytes.push(msg.num);
                bytes.extend(msg.string.as_bytes());
            }
            Message::NoOp(_) => {}
            Message::U16(msg) => bytes.extend(msg.num.to_le_bytes()),
        }

        bytes
    }

    fn from_bytes(message_type: u16, data: Vec<u8>) -> Self {
        match message_type {
            0 => Message::Bytes(messages::Bytes { data }),
            1 => Message::U8(messages::U8 { num: data[0] }),
            2 => Message::MyString(messages::MyString {
                string: String::from_utf8(data).unwrap(),
            }),
            3 => Message::Multi(messages::Multi {
                num: data[0],
                string: String::from_utf8(data[1..].to_vec()).unwrap(),
            }),
            4 => Message::NoOp(messages::NoOp {}),
            5 => Message::U16(messages::U16 {
                num: u16::from_le_bytes([data[0], data[1]]),
            }),
            _ => panic!("Invalid message type: {}", message_type),
        }
    }
}

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
    const START_BYTE: u8 = 0xFF;

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
mod tests {
    use super::*;
    use std::{os::unix::net::UnixStream, time::Duration};

    #[test]
    fn test_send_raw_bytes() {
        let test_cases = vec![
            (
                Message::NoOp(messages::NoOp {}),
                vec![
                    0xFF, // Start byte
                    0x02, 0x00, // Length (2 bytes for message type)
                    0x04, 0x00, // Message type (4)
                ],
            ),
            (
                Message::U8(messages::U8 { num: 0x57 }),
                vec![
                    0xFF, // Start byte
                    0x03, 0x00, // Length (2 bytes for message type + 1 byte data)
                    0x01, 0x00, // Message type (1)
                    0x57, // The u8 value
                ],
            ),
            (
                Message::Bytes(messages::Bytes {
                    data: vec![1, 2, 3, 4, 5],
                }),
                vec![
                    0xFF, // Start byte
                    0x07, 0x00, // Length (2 bytes for message type + 5 bytes data)
                    0x00, 0x00, // Message type (0)
                    1, 2, 3, 4, 5, // The bytes
                ],
            ),
        ];

        for (message, expected_bytes) in test_cases {
            let (stream1, mut stream2) = UnixStream::pair().unwrap();
            let mut sender = SerialManager::new(stream1);
            sender.send(message).unwrap();

            let mut received_bytes = vec![0u8; expected_bytes.len()];
            stream2.read_exact(&mut received_bytes).unwrap();
            assert_eq!(received_bytes, expected_bytes);

            // Try to read one more byte to verify nothing else was sent
            stream2
                .set_read_timeout(Some(Duration::from_millis(10)))
                .unwrap();
            let mut extra_byte = [0u8; 1];
            assert!(stream2.read_exact(&mut extra_byte).is_err());
        }
    }

    #[test]
    fn test_send_receive() {
        let (stream1, stream2) = UnixStream::pair().unwrap();
        let mut sender = SerialManager::new(stream1);
        let mut receiver = SerialManager::new(stream2);

        let test_messages = vec![
            Message::Bytes(messages::Bytes {
                data: vec![0x48, 0x65, 0x6C, 0x6C, 0x6F],
            }),
            Message::U8(messages::U8 { num: 0x57 }),
            Message::MyString(messages::MyString {
                string: "Hello, world!".to_string(),
            }),
            Message::Multi(messages::Multi {
                num: 0x57,
                string: "Hello, world!".to_string(),
            }),
            Message::NoOp(messages::NoOp {}),
            Message::U16(messages::U16 { num: 0x57 }),
        ];

        for message in test_messages {
            sender.send(message.clone()).unwrap();
            let received = receiver.receive().unwrap();
            assert_eq!(message, received);
        }
    }
}
