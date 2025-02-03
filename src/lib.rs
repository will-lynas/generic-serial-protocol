use std::io::{self, Read, Write};

mod messages {
    #[derive(Debug, PartialEq, Clone)]
    pub struct Bytes {
        pub data: Vec<u8>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Num {
        pub num: u8,
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
    Num(messages::Num),
    MyString(messages::MyString),
    Multi(messages::Multi),
    NoOp(messages::NoOp),
}

impl Message {
    fn message_type(&self) -> u16 {
        match self {
            Message::Bytes(_) => 0,
            Message::Num(_) => 1,
            Message::MyString(_) => 2,
            Message::Multi(_) => 3,
            Message::NoOp(_) => 4,
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let message_type_bytes = self.message_type().to_be_bytes();
        bytes.extend_from_slice(&message_type_bytes);

        match self {
            Message::Bytes(msg) => bytes.extend(msg.data),
            Message::Num(msg) => bytes.push(msg.num),
            Message::MyString(msg) => bytes.extend(msg.string.as_bytes()),
            Message::Multi(msg) => {
                bytes.push(msg.num);
                bytes.extend(msg.string.as_bytes());
            }
            Message::NoOp(_) => {}
        }

        bytes
    }

    fn from_bytes(message_type: u16, data: Vec<u8>) -> Self {
        match message_type {
            0 => Message::Bytes(messages::Bytes { data }),
            1 => Message::Num(messages::Num { num: data[0] }),
            2 => Message::MyString(messages::MyString {
                string: String::from_utf8(data).unwrap(),
            }),
            3 => Message::Multi(messages::Multi {
                num: data[0],
                string: String::from_utf8(data[1..].to_vec()).unwrap(),
            }),
            4 => Message::NoOp(messages::NoOp {}),
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
        let data = message.to_bytes();
        let length = (data.len() as u16).to_be_bytes();

        self.connection.write_all(&[Self::START_BYTE])?;
        self.connection.write_all(&length)?;
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
        let length = u16::from_be_bytes(length_bytes) as usize;

        let mut buffer = vec![0u8; length];
        self.connection.read_exact(&mut buffer)?;

        let message_type = u16::from_be_bytes([buffer[0], buffer[1]]);
        let data = buffer[2..].to_vec();
        Ok(Message::from_bytes(message_type, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    #[test]
    fn test_send_receive() {
        let (stream1, stream2) = UnixStream::pair().unwrap();
        let mut sender = SerialManager::new(stream1);
        let mut receiver = SerialManager::new(stream2);

        let test_messages = vec![
            Message::Bytes(messages::Bytes {
                data: vec![0x48, 0x65, 0x6C, 0x6C, 0x6F],
            }),
            Message::Num(messages::Num { num: 0x57 }),
            Message::MyString(messages::MyString {
                string: "Hello, world!".to_string(),
            }),
            Message::Multi(messages::Multi {
                num: 0x57,
                string: "Hello, world!".to_string(),
            }),
            Message::NoOp(messages::NoOp {}),
        ];

        for message in test_messages {
            sender.send(message.clone()).unwrap();
            let received = receiver.receive().unwrap();
            assert_eq!(message, received);
        }
    }
}
