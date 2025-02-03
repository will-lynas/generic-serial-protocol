use std::io::{self, Read, Write};

#[derive(Debug, PartialEq, Clone)]
pub struct TestMessage {
    data: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Test(TestMessage),
}

impl Message {
    fn to_bytes(self) -> Vec<u8> {
        match self {
            Message::Test(test_msg) => test_msg.data,
        }
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        Message::Test(TestMessage { data: bytes })
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

        let test_message = Message::Test(TestMessage {
            data: vec![0x48, 0x65, 0x6C, 0x6C, 0x6F],
        });
        let expected = test_message.clone();
        sender.send(test_message).unwrap();
        let received = receiver.receive().unwrap();
        assert_eq!(expected, received);
    }
}
