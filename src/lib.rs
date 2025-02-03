use std::io::{self, Read, Write};

mod messages {
    #[derive(Debug, PartialEq, Clone)]
    pub struct Bytes {
        pub data: Vec<u8>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Test2 {
        pub data: Vec<u8>,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Bytes(messages::Bytes),
    Test2(messages::Test2),
}

impl Message {
    fn message_type(&self) -> u8 {
        match self {
            Message::Bytes(_) => 0,
            Message::Test2(_) => 1,
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.message_type());

        match self {
            Message::Bytes(msg) => bytes.extend(msg.data),
            Message::Test2(msg) => bytes.extend(msg.data),
        }

        bytes
    }

    fn from_bytes(message_type: u8, data: Vec<u8>) -> Self {
        match message_type {
            0 => Message::Bytes(messages::Bytes { data }),
            1 => Message::Test2(messages::Test2 { data }),
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
            Message::Test2(messages::Test2 {
                data: vec![0x57, 0x6F, 0x72, 0x6C, 0x64],
            }),
        ];

        for message in test_messages {
            sender.send(message.clone()).unwrap();
            let received = receiver.receive().unwrap();
            assert_eq!(message, received);
        }
    }
}
