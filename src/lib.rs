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
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Bytes(messages::Bytes),
    Num(messages::Num),
    MyString(messages::MyString),
}

impl Message {
    fn message_type(&self) -> u8 {
        match self {
            Message::Bytes(_) => 0,
            Message::Num(_) => 1,
            Message::MyString(_) => 2,
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.message_type());

        match self {
            Message::Bytes(msg) => bytes.extend(msg.data),
            Message::Num(msg) => bytes.push(msg.num),
            Message::MyString(msg) => bytes.extend(msg.string.as_bytes()),
        }

        bytes
    }

    fn from_bytes(message_type: u8, data: Vec<u8>) -> Self {
        match message_type {
            0 => Message::Bytes(messages::Bytes { data }),
            1 => Message::Num(messages::Num { num: data[0] }),
            2 => Message::MyString(messages::MyString {
                string: String::from_utf8(data).unwrap(),
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
    pub fn new(connection: T) -> Self {
        Self { connection }
    }

    pub fn send(&mut self, message: Message) -> io::Result<()> {
        let data = message.to_bytes();
        self.connection.write_all(&data)?;
        self.connection.flush()?;
        Ok(())
    }

    pub fn receive(&mut self) -> io::Result<Message> {
        let mut buffer = vec![0; 1024];
        let n = self.connection.read(&mut buffer)?;
        buffer.truncate(n);
        let message_type = buffer[0];
        let data = buffer[1..].to_vec();
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
        ];

        for message in test_messages {
            sender.send(message.clone()).unwrap();
            let received = receiver.receive().unwrap();
            assert_eq!(message, received);
        }
    }
}
