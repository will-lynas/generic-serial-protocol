use std::io::{self, Read, Write};

mod messages {
    #[derive(Debug, PartialEq, Clone)]
    pub struct Test {
        pub data: Vec<u8>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Test2 {
        pub data: Vec<u8>,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Test(messages::Test),
    Test2(messages::Test2),
}

impl Message {
    fn message_type(&self) -> u8 {
        match self {
            Message::Test(_) => 0,
            Message::Test2(_) => 1,
        }
    }

    fn from_type(message_type: u8, data: Vec<u8>) -> Self {
        match message_type {
            0 => Message::Test(messages::Test { data }),
            1 => Message::Test2(messages::Test2 { data }),
            _ => panic!("Invalid message type: {}", message_type),
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.message_type());

        match self {
            Message::Test(msg) => bytes.extend(msg.data),
            Message::Test2(msg) => bytes.extend(msg.data),
        }

        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        let message_type = bytes[0];
        let data = bytes[1..].to_vec();
        Self::from_type(message_type, data)
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
        Ok(Message::from_bytes(buffer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    #[test]
    fn test_serial_communication() {
        let (stream1, stream2) = UnixStream::pair().unwrap();

        let mut sender = SerialManager::new(stream1);
        let mut receiver = SerialManager::new(stream2);

        let test_message = Message::Test(messages::Test {
            data: vec![0x48, 0x65, 0x6C, 0x6C, 0x6F],
        });
        let expected = test_message.clone();
        sender.send(test_message).unwrap();
        let received = receiver.receive().unwrap();
        assert_eq!(expected, received);
    }

    #[test]
    fn test_serial_communication_test2() {
        let (stream1, stream2) = UnixStream::pair().unwrap();

        let mut sender = SerialManager::new(stream1);
        let mut receiver = SerialManager::new(stream2);

        let test_message = Message::Test2(messages::Test2 {
            data: vec![0x57, 0x6F, 0x72, 0x6C, 0x64], // "World" in hex
        });
        let expected = test_message.clone();
        sender.send(test_message).unwrap();
        let received = receiver.receive().unwrap();
        assert_eq!(expected, received);
    }
}
